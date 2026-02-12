use anyhow::{anyhow, Result};
use rusqlite::{params, Connection, Row};
use std::sync::{Arc, Mutex};

use crate::models::{Document, DocumentType};

pub struct DocumentRepository {
    conn: Arc<Mutex<Connection>>,
}

impl DocumentRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    // === DocumentType ===

    /// Hämta alla dokumenttyper
    pub fn get_all_types(&self) -> Result<Vec<DocumentType>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, target_directory, default_filename, description
             FROM document_types ORDER BY name"
        )?;

        let types = stmt
            .query_map([], |row| {
                Ok(DocumentType {
                    id: row.get(0).ok(),
                    name: row.get(1)?,
                    target_directory: row.get(2)?,
                    default_filename: row.get(3).ok(),
                    description: row.get(4).ok(),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(types)
    }

    /// Hämta dokumenttyp via ID
    pub fn get_type_by_id(&self, id: i64) -> Result<Option<DocumentType>> {
        let conn = self.conn.lock().unwrap();
        let result = conn
            .query_row(
                "SELECT id, name, target_directory, default_filename, description
                 FROM document_types WHERE id = ?",
                [id],
                |row| {
                    Ok(DocumentType {
                        id: row.get(0).ok(),
                        name: row.get(1)?,
                        target_directory: row.get(2)?,
                        default_filename: row.get(3).ok(),
                        description: row.get(4).ok(),
                    })
                },
            )
            .ok();

        Ok(result)
    }

    /// Hitta dokumenttyp baserat på sökväg
    pub fn find_type_by_path(&self, relative_path: &str) -> Result<Option<DocumentType>> {
        let types = self.get_all_types()?;

        for doc_type in types {
            if relative_path.starts_with(&doc_type.target_directory) {
                return Ok(Some(doc_type));
            }
        }

        Ok(None)
    }

    /// Skapa ny dokumenttyp
    pub fn create_type(&self, doc_type: &DocumentType) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO document_types (name, target_directory, default_filename, description)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                doc_type.name,
                doc_type.target_directory,
                doc_type.default_filename,
                doc_type.description,
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Uppdatera dokumenttyp
    pub fn update_type(&self, doc_type: &DocumentType) -> Result<()> {
        let id = doc_type.id.ok_or_else(|| anyhow!("Dokumenttyp har inget ID"))?;

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE document_types SET
                name = ?1, target_directory = ?2, default_filename = ?3, description = ?4
             WHERE id = ?5",
            params![
                doc_type.name,
                doc_type.target_directory,
                doc_type.default_filename,
                doc_type.description,
                id,
            ],
        )?;

        Ok(())
    }

    /// Ta bort dokumenttyp
    pub fn delete_type(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM document_types WHERE id = ?", [id])?;
        Ok(())
    }

    // === Document ===

    /// Hämta alla dokument för en person
    pub fn find_by_person(&self, person_id: i64) -> Result<Vec<Document>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, person_id, document_type_id, filename, relative_path,
                    file_size, file_type, tags, file_modified_at, created_at, updated_at
             FROM documents
             WHERE person_id = ?
             ORDER BY document_type_id, filename"
        )?;

        let docs = stmt
            .query_map([person_id], |row| Ok(Self::row_to_document(row)))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(docs)
    }

    /// Hämta dokument via ID
    pub fn find_by_id(&self, id: i64) -> Result<Option<Document>> {
        let conn = self.conn.lock().unwrap();
        let result = conn
            .query_row(
                "SELECT id, person_id, document_type_id, filename, relative_path,
                        file_size, file_type, tags, file_modified_at, created_at, updated_at
                 FROM documents WHERE id = ?",
                [id],
                |row| Ok(Self::row_to_document(row)),
            )
            .ok();

        Ok(result)
    }

    /// Hitta dokument baserat på person och relativ sökväg
    pub fn find_by_path(&self, person_id: i64, relative_path: &str) -> Result<Option<Document>> {
        let conn = self.conn.lock().unwrap();
        let result = conn
            .query_row(
                "SELECT id, person_id, document_type_id, filename, relative_path,
                        file_size, file_type, tags, file_modified_at, created_at, updated_at
                 FROM documents WHERE person_id = ? AND relative_path = ?",
                params![person_id, relative_path],
                |row| Ok(Self::row_to_document(row)),
            )
            .ok();

        Ok(result)
    }

    /// Skapa dokument
    pub fn create(&self, doc: &mut Document) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO documents (person_id, document_type_id, filename, relative_path,
                                    file_size, file_type, tags, file_modified_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                doc.person_id,
                doc.document_type_id,
                doc.filename,
                doc.relative_path,
                doc.file_size,
                doc.file_type,
                doc.tags,
                doc.file_modified_at.map(|d| d.to_string()),
            ],
        )?;

        let id = conn.last_insert_rowid();
        doc.id = Some(id);

        Ok(id)
    }

    /// Uppdatera dokument
    pub fn update(&self, doc: &Document) -> Result<()> {
        let id = doc.id.ok_or_else(|| anyhow!("Dokument har inget ID"))?;

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE documents SET
                document_type_id = ?1, filename = ?2, relative_path = ?3,
                file_size = ?4, file_type = ?5, tags = ?6, file_modified_at = ?7,
                updated_at = datetime('now')
             WHERE id = ?8",
            params![
                doc.document_type_id,
                doc.filename,
                doc.relative_path,
                doc.file_size,
                doc.file_type,
                doc.tags,
                doc.file_modified_at.map(|d| d.to_string()),
                id,
            ],
        )?;

        Ok(())
    }

    /// Ta bort dokument
    pub fn delete(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM documents WHERE id = ?", [id])?;
        Ok(())
    }

    /// Ta bort alla dokument för en person
    pub fn delete_by_person(&self, person_id: i64) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM documents WHERE person_id = ?", [person_id])?;
        Ok(rows)
    }

    /// Räkna dokument per person
    pub fn count_by_person(&self, person_id: i64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM documents WHERE person_id = ?",
            [person_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Räkna totalt antal dokument
    pub fn count(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Räkna antal bilder
    pub fn count_images(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM documents WHERE file_type IN ('jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp')",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Hämta senaste dokument (med personnamn)
    pub fn find_recent(&self, limit: usize) -> Result<Vec<(Document, Option<String>)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT d.id, d.person_id, d.document_type_id, d.filename, d.relative_path,
                    d.file_size, d.file_type, d.tags, d.file_modified_at, d.created_at, d.updated_at,
                    COALESCE(p.firstname || ' ', '') || COALESCE(p.surname, '')
             FROM documents d
             LEFT JOIN persons p ON d.person_id = p.id
             ORDER BY d.created_at DESC
             LIMIT ?"
        )?;

        let results = stmt
            .query_map([limit as i64], |row| {
                let doc = Self::row_to_document(row);
                let person_name: Option<String> = row.get(11).ok();
                Ok((doc, person_name))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(results)
    }

    /// Beräkna total filstorlek
    pub fn total_file_size(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let size: i64 = conn.query_row(
            "SELECT COALESCE(SUM(file_size), 0) FROM documents",
            [],
            |row| row.get(0),
        )?;
        Ok(size)
    }

    /// Hämta dokument grupperade per typ för en person
    pub fn find_by_person_grouped(&self, person_id: i64) -> Result<Vec<(Option<DocumentType>, Vec<Document>)>> {
        let docs = self.find_by_person(person_id)?;
        let types = self.get_all_types()?;

        let mut grouped: Vec<(Option<DocumentType>, Vec<Document>)> = Vec::new();

        // Gruppera dokument per typ
        for doc_type in types {
            let type_docs: Vec<Document> = docs
                .iter()
                .filter(|d| d.document_type_id == doc_type.id)
                .cloned()
                .collect();

            if !type_docs.is_empty() {
                grouped.push((Some(doc_type), type_docs));
            }
        }

        // Dokument utan typ
        let untyped: Vec<Document> = docs
            .iter()
            .filter(|d| d.document_type_id.is_none())
            .cloned()
            .collect();

        if !untyped.is_empty() {
            grouped.push((None, untyped));
        }

        Ok(grouped)
    }

    fn row_to_document(row: &Row) -> Document {
        Document {
            id: row.get(0).ok(),
            person_id: row.get(1).unwrap_or(0),
            document_type_id: row.get(2).ok(),
            filename: row.get(3).unwrap_or_default(),
            relative_path: row.get(4).unwrap_or_default(),
            file_size: row.get(5).unwrap_or(0),
            file_type: row.get(6).ok(),
            tags: row.get(7).ok(),
            file_modified_at: row
                .get::<_, Option<String>>(8)
                .ok()
                .flatten()
                .and_then(|s| chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").ok()),
            created_at: row.get(9).ok(),
            updated_at: row.get(10).ok(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::models::Person;

    fn setup_db() -> Database {
        Database::open_in_memory().unwrap()
    }

    #[test]
    fn test_document_types() {
        let db = setup_db();
        let repo = db.documents();

        let types = repo.get_all_types().unwrap();
        assert!(!types.is_empty());

        // Standardtyper ska finnas
        let names: Vec<&str> = types.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"Personbevis"));
    }

    #[test]
    fn test_create_document() {
        let db = setup_db();

        // Skapa person först
        let mut person = Person::new(Some("Test".into()), None, "test".into());
        db.persons().create(&mut person).unwrap();

        // Skapa dokument
        let mut doc = Document::new(
            person.id.unwrap(),
            "test.pdf".into(),
            "dokument/test.pdf".into(),
        );
        doc.file_size = 1024;

        let id = db.documents().create(&mut doc).unwrap();
        assert!(id > 0);

        let found = db.documents().find_by_id(id).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().filename, "test.pdf");
    }
}
