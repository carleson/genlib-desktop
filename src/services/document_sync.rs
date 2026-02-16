//! Dokumentsynkronisering
//!
//! Synkroniserar filsystemet med databasen för en person.
//! - Lägger till nya filer som hittas
//! - Tar bort poster för filer som inte längre finns
//! - Uppdaterar metadata för ändrade filer

use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;

use crate::db::Database;
use crate::models::{Document, DocumentType, Person};
use crate::utils::file_ops;

/// Resultat av synkronisering
#[derive(Debug, Default)]
pub struct SyncResult {
    /// Antal nya dokument tillagda
    pub added: usize,
    /// Antal dokument uppdaterade (metadata)
    pub updated: usize,
    /// Antal dokument borttagna (fil fanns inte längre)
    pub removed: usize,
    /// Antal filer som ignorerades (ingen matchande dokumenttyp)
    pub ignored: usize,
    /// Eventuella varningar
    pub warnings: Vec<String>,
}

impl SyncResult {
    pub fn summary(&self) -> String {
        format!(
            "{} tillagda, {} uppdaterade, {} borttagna, {} ignorerade",
            self.added, self.updated, self.removed, self.ignored
        )
    }

    pub fn has_changes(&self) -> bool {
        self.added > 0 || self.updated > 0 || self.removed > 0
    }
}

/// Tjänst för dokumentsynkronisering
pub struct DocumentSyncService<'a> {
    db: &'a Database,
}

impl<'a> DocumentSyncService<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Synkronisera dokument för en person
    pub fn sync_person(&self, person: &Person) -> Result<SyncResult> {
        let person_id = person.id.ok_or_else(|| anyhow::anyhow!("Person saknar ID"))?;

        let config = self.db.config().get()?;
        let person_dir = config.persons_directory().join(&person.directory_name);

        if !person_dir.exists() {
            // Skapa katalogen om den inte finns
            file_ops::ensure_directory(&person_dir)?;
            return Ok(SyncResult::default());
        }

        let mut result = SyncResult::default();

        // Hämta alla dokumenttyper
        let doc_types = self.db.documents().get_all_types()?;

        // Hämta befintliga dokument från databas
        let existing_docs = self.db.documents().find_by_person(person_id)?;
        let existing_paths: HashSet<String> = existing_docs
            .iter()
            .map(|d| d.relative_path.clone())
            .collect();

        // Skanna filsystemet
        let files = file_ops::scan_directory_relative(&person_dir)?;
        let file_paths: HashSet<String> = files.iter().map(|(_, rel)| rel.clone()).collect();

        // 1. Hitta nya filer (finns i filsystem men inte i databas)
        for (full_path, relative_path) in &files {
            if !existing_paths.contains(relative_path) {
                // Ny fil - försök matcha mot dokumenttyp
                match self.match_document_type(relative_path, &doc_types) {
                    Some(doc_type) => {
                        // Skapa dokument
                        if let Err(e) = self.create_document(person_id, full_path, relative_path, &doc_type) {
                            result.warnings.push(format!(
                                "Kunde inte lägga till {}: {}",
                                relative_path, e
                            ));
                        } else {
                            result.added += 1;
                            tracing::info!("Lade till dokument: {}", relative_path);
                        }
                    }
                    None => {
                        // Ingen matchande dokumenttyp - ignorera eller lägg till som okategoriserad
                        result.ignored += 1;
                        tracing::debug!("Ignorerade fil (ingen typ): {}", relative_path);
                    }
                }
            }
        }

        // 2. Hitta borttagna filer (finns i databas men inte i filsystem)
        for doc in &existing_docs {
            if !file_paths.contains(&doc.relative_path) {
                // Fil finns inte längre - ta bort från databas
                if let Some(id) = doc.id {
                    if let Err(e) = self.db.documents().delete(id) {
                        result.warnings.push(format!(
                            "Kunde inte ta bort {}: {}",
                            doc.relative_path, e
                        ));
                    } else {
                        result.removed += 1;
                        tracing::info!("Tog bort dokument: {}", doc.relative_path);
                    }
                }
            }
        }

        // 3. Uppdatera metadata för befintliga filer
        for doc in &existing_docs {
            if file_paths.contains(&doc.relative_path) {
                let full_path = person_dir.join(&doc.relative_path);

                if let Ok(true) = self.needs_metadata_update(doc, &full_path) {
                    if let Err(e) = self.update_document_metadata(doc, &full_path) {
                        result.warnings.push(format!(
                            "Kunde inte uppdatera {}: {}",
                            doc.relative_path, e
                        ));
                    } else {
                        result.updated += 1;
                        tracing::info!("Uppdaterade dokument: {}", doc.relative_path);
                    }
                }
            }
        }

        Ok(result)
    }

    /// Matcha en fil mot en dokumenttyp baserat på sökväg
    fn match_document_type(&self, relative_path: &str, doc_types: &[DocumentType]) -> Option<DocumentType> {
        // Normalisera sökvägen
        let path_lower = relative_path.to_lowercase();

        // Försök matcha mot target_directory
        for doc_type in doc_types {
            let target_lower = doc_type.target_directory.to_lowercase();

            // Matcha om sökvägen börjar med target_directory
            if path_lower.starts_with(&target_lower) {
                return Some(doc_type.clone());
            }

            // Alternativt: matcha på katalognamn någonstans i sökvägen
            let parts: Vec<&str> = relative_path.split('/').collect();
            if parts.len() > 1 {
                let parent_dir = parts[..parts.len() - 1].join("/").to_lowercase();
                if parent_dir == target_lower || parent_dir.ends_with(&format!("/{}", target_lower)) {
                    return Some(doc_type.clone());
                }
            }
        }

        // Fallback: matcha baserat på filändelse till generiska typer
        let extension = Path::new(relative_path)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        match extension.as_deref() {
            Some("jpg" | "jpeg" | "png" | "gif" | "webp") => {
                // Hitta bildtyp
                doc_types.iter()
                    .find(|t| t.target_directory.contains("bild") || t.target_directory.contains("image"))
                    .cloned()
            }
            Some("pdf") => {
                // Hitta dokumenttyp
                doc_types.iter()
                    .find(|t| t.target_directory.contains("dokument"))
                    .cloned()
            }
            Some("txt" | "md") => {
                // Hitta anteckningstyp
                doc_types.iter()
                    .find(|t| t.target_directory.contains("anteck") || t.target_directory.contains("note"))
                    .cloned()
            }
            _ => None,
        }
    }

    /// Skapa ett nytt dokument i databasen
    fn create_document(
        &self,
        person_id: i64,
        full_path: &Path,
        relative_path: &str,
        doc_type: &DocumentType,
    ) -> Result<()> {
        let filename = full_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let file_size = file_ops::get_file_size(full_path)? as i64;
        let file_type = file_ops::get_file_extension(full_path);
        let file_modified = file_ops::get_modified_time(full_path).ok();

        let mut document = Document {
            id: None,
            person_id,
            document_type_id: doc_type.id,
            filename,
            relative_path: relative_path.to_string(),
            file_size,
            file_type,
            file_modified_at: file_modified,
            created_at: None,
            updated_at: None,
        };

        self.db.documents().create(&mut document)?;

        Ok(())
    }

    /// Kontrollera om ett dokument behöver uppdateras
    fn needs_metadata_update(&self, doc: &Document, full_path: &Path) -> Result<bool> {
        let current_size = file_ops::get_file_size(full_path)? as i64;

        // Uppdatera om filstorleken har ändrats
        if current_size != doc.file_size {
            return Ok(true);
        }

        // Uppdatera om modifieringstiden har ändrats
        if let Ok(current_modified) = file_ops::get_modified_time(full_path) {
            if let Some(doc_modified) = doc.file_modified_at {
                if current_modified != doc_modified {
                    return Ok(true);
                }
            } else {
                // Dokument saknar modifieringstid, uppdatera
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Uppdatera metadata för ett befintligt dokument
    fn update_document_metadata(&self, doc: &Document, full_path: &Path) -> Result<()> {
        let file_size = file_ops::get_file_size(full_path)? as i64;
        let file_modified = file_ops::get_modified_time(full_path).ok();

        let mut updated_doc = doc.clone();
        updated_doc.file_size = file_size;
        updated_doc.file_modified_at = file_modified;

        self.db.documents().update(&updated_doc)?;

        Ok(())
    }

    /// Synkronisera alla personer (för batch-operationer)
    pub fn sync_all(&self) -> Result<Vec<(String, SyncResult)>> {
        let persons = self.db.persons().find_all()?;
        let mut results = Vec::new();

        for person in persons {
            let result = self.sync_person(&person)?;
            results.push((person.full_name(), result));
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // Tester kräver en mer komplex setup med databas och filsystem
    // Läggs till vid behov
}
