//! Modal för att lägga till dokument (import eller skapa text)

use egui::{self, RichText};
use std::path::PathBuf;

use crate::db::Database;
use crate::models::{Document, DocumentType, Person};
use crate::ui::{
    state::{AppState, DocumentUploadMode},
    theme::{Colors, Icons},
};
use crate::utils::file_ops;

/// Status för en fil i import
#[derive(Clone)]
struct ImportFileEntry {
    path: PathBuf,
    filename: String,
    status: ImportStatus,
}

#[derive(Clone, Copy, PartialEq)]
enum ImportStatus {
    Pending,
    Done,
    Failed,
}

/// Tillstånd för dokumentuppladdning
#[derive(Default)]
pub struct DocumentUploadModal {
    /// Valda filer att importera
    selected_files: Vec<ImportFileEntry>,
    /// Vald dokumenttyp
    selected_type_id: Option<i64>,
    /// Textinnehåll för nya textdokument
    text_content: String,
    /// Filnamn för textdokument
    text_filename: String,
    /// Om användaren har ändrat filnamnet manuellt
    filename_manually_edited: bool,
    /// Cachade dokumenttyper
    document_types: Vec<DocumentType>,
    /// Behöver refresha dokumenttyper
    needs_refresh: bool,
    /// Felmeddelande
    error_message: Option<String>,
    /// Uppladdning pågår
    uploading: bool,
    /// Antal filer som importerats
    import_success_count: usize,
    /// Antal filer som misslyckades
    import_failed_count: usize,
}

impl DocumentUploadModal {
    pub fn new() -> Self {
        Self {
            needs_refresh: true,
            ..Default::default()
        }
    }

    /// Återställ modal till ursprungstillstånd
    pub fn reset(&mut self) {
        self.selected_files.clear();
        self.selected_type_id = None;
        self.text_content.clear();
        self.text_filename.clear();
        self.filename_manually_edited = false;
        self.error_message = None;
        self.uploading = false;
        self.import_success_count = 0;
        self.import_failed_count = 0;
    }

    /// Visa modalen. Returnerar true om den ska stängas.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        state: &mut AppState,
        db: &Database,
        person: &Person,
    ) -> bool {
        let mut should_close = false;
        let mode = state.document_upload_mode;

        // Ladda dokumenttyper om nödvändigt
        if self.needs_refresh {
            self.document_types = db.documents().get_all_types().unwrap_or_default();
            self.needs_refresh = false;
        }

        let title = match mode {
            DocumentUploadMode::Import => format!("{} Importera filer", Icons::IMPORT),
            DocumentUploadMode::Create => format!("{} Skapa dokument", Icons::ADD),
        };

        egui::Window::new(title)
            .collapsible(false)
            .resizable(true)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(500.0);

                ui.label(format!("Till: {}", person.full_name()));
                ui.add_space(8.0);

                // Välj dokumenttyp
                ui.horizontal(|ui| {
                    ui.label("Dokumenttyp:");

                    let selected_name = self.selected_type_id
                        .and_then(|id| self.document_types.iter().find(|t| t.id == Some(id)))
                        .map(|t| t.name.as_str())
                        .unwrap_or("Välj typ...");

                    egui::ComboBox::from_id_salt("doc_type_combo")
                        .selected_text(selected_name)
                        .show_ui(ui, |ui| {
                            for doc_type in &self.document_types {
                                if let Some(id) = doc_type.id {
                                    let is_selected = self.selected_type_id == Some(id);
                                    if ui.selectable_label(is_selected, &doc_type.name).clicked() {
                                        let type_changed = self.selected_type_id != Some(id);
                                        self.selected_type_id = Some(id);

                                        // Förifyll filnamn från mall om användaren inte har ändrat manuellt
                                        if type_changed && !self.filename_manually_edited {
                                            if let Some(ref default_name) = doc_type.default_filename {
                                                self.text_filename = default_name.clone();
                                            }
                                        }
                                    }
                                }
                            }
                        });
                });

                // Visa målkatalog
                if let Some(type_id) = self.selected_type_id {
                    if let Some(doc_type) = self.document_types.iter().find(|t| t.id == Some(type_id)) {
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Sparas till:").color(Colors::TEXT_SECONDARY));
                            ui.label(RichText::new(&doc_type.target_directory).small());
                        });
                    }
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                match mode {
                    DocumentUploadMode::Import => self.show_import_ui(ui),
                    DocumentUploadMode::Create => self.show_create_ui(ui),
                }

                // Felmeddelande
                if let Some(ref error) = self.error_message {
                    ui.add_space(8.0);
                    ui.label(RichText::new(error).color(Colors::ERROR));
                }

                ui.add_space(16.0);

                // Knappar
                ui.horizontal(|ui| {
                    if ui.button("Avbryt").clicked() {
                        self.reset();
                        should_close = true;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let template_has_default_filename = self.selected_type_id
                            .and_then(|id| self.document_types.iter().find(|t| t.id == Some(id)))
                            .and_then(|t| t.default_filename.as_ref())
                            .map(|s| !s.is_empty())
                            .unwrap_or(false);

                        let can_save = match mode {
                            DocumentUploadMode::Import => {
                                !self.selected_files.is_empty()
                                    && self.selected_type_id.is_some()
                                    && !self.uploading
                            }
                            DocumentUploadMode::Create => {
                                !self.text_content.is_empty()
                                    && self.selected_type_id.is_some()
                                    && !self.uploading
                                    && (!self.text_filename.is_empty() || template_has_default_filename)
                            }
                        };

                        let button_text = match mode {
                            DocumentUploadMode::Import => {
                                let count = self.selected_files.len();
                                if count == 1 {
                                    format!("{} Importera fil", Icons::SAVE)
                                } else {
                                    format!("{} Importera {} filer", Icons::SAVE, count)
                                }
                            }
                            DocumentUploadMode::Create => format!("{} Skapa", Icons::SAVE),
                        };

                        ui.add_enabled_ui(can_save, |ui| {
                            if ui.button(button_text).clicked() {
                                let result = match mode {
                                    DocumentUploadMode::Import => self.import_files(db, person),
                                    DocumentUploadMode::Create => self.create_text_document(db, person),
                                };

                                match result {
                                    Ok(_) => {
                                        let msg = match mode {
                                            DocumentUploadMode::Import => {
                                                if self.import_failed_count > 0 {
                                                    format!(
                                                        "{} filer importerade, {} misslyckades",
                                                        self.import_success_count,
                                                        self.import_failed_count
                                                    )
                                                } else if self.import_success_count == 1 {
                                                    "Fil importerad!".to_string()
                                                } else {
                                                    format!("{} filer importerade!", self.import_success_count)
                                                }
                                            }
                                            DocumentUploadMode::Create => "Dokument skapat!".to_string(),
                                        };
                                        state.show_success(&msg);
                                        self.reset();
                                        should_close = true;
                                    }
                                    Err(e) => {
                                        self.error_message = Some(e.to_string());
                                    }
                                }
                            }
                        });
                    });
                });
            });

        should_close
    }

    /// UI för att importera filer
    fn show_import_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if self.selected_files.is_empty() {
                ui.label(RichText::new("Inga filer valda").color(Colors::TEXT_MUTED));
            } else {
                ui.label(RichText::new(format!("{} fil(er) valda", self.selected_files.len())).strong());
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Välj filer...").clicked() {
                    if let Some(paths) = rfd::FileDialog::new()
                        .add_filter("Alla filer", &["*"])
                        .add_filter("Bilder", &["jpg", "jpeg", "png", "gif", "webp"])
                        .add_filter("Dokument", &["pdf", "txt", "md", "doc", "docx"])
                        .pick_files()
                    {
                        self.selected_files = paths
                            .into_iter()
                            .map(|path| {
                                let filename = path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("unknown")
                                    .to_string();
                                ImportFileEntry {
                                    path,
                                    filename,
                                    status: ImportStatus::Pending,
                                }
                            })
                            .collect();
                        self.error_message = None;
                    }
                }

                if !self.selected_files.is_empty() {
                    if ui.small_button("Rensa").clicked() {
                        self.selected_files.clear();
                    }
                }
            });
        });

        // Lista valda filer
        if !self.selected_files.is_empty() {
            ui.add_space(8.0);

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    let mut to_remove = Vec::new();

                    for (i, entry) in self.selected_files.iter().enumerate() {
                        ui.horizontal(|ui| {
                            // Status-ikon
                            let (status_icon, status_color) = match entry.status {
                                ImportStatus::Pending => ("○", Colors::TEXT_MUTED),
                                ImportStatus::Done => ("✓", Colors::SUCCESS),
                                ImportStatus::Failed => ("✗", Colors::ERROR),
                            };
                            ui.label(RichText::new(status_icon).color(status_color));
                            ui.label(&entry.filename);

                            if !self.uploading {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.small_button("✕").clicked() {
                                        to_remove.push(i);
                                    }
                                });
                            }
                        });
                    }

                    for i in to_remove.into_iter().rev() {
                        self.selected_files.remove(i);
                    }
                });
        }
    }

    /// UI för att skapa textdokument
    fn show_create_ui(&mut self, ui: &mut egui::Ui) {
        // Hämta mallens default filnamn
        let template_default_filename = self.selected_type_id
            .and_then(|id| self.document_types.iter().find(|t| t.id == Some(id)))
            .and_then(|t| t.default_filename.clone());

        // Filnamn
        ui.horizontal(|ui| {
            ui.label("Filnamn:");

            let hint = template_default_filename.as_ref()
                .map(|s| format!("Standard: {}", s))
                .unwrap_or_else(|| "t.ex. anteckningar.txt".to_string());

            let response = ui.add(
                egui::TextEdit::singleline(&mut self.text_filename)
                    .hint_text(&hint)
                    .desired_width(200.0)
            );

            if response.changed() && !self.text_filename.is_empty() {
                self.filename_manually_edited = true;
            }

            if self.text_filename.is_empty() {
                if let Some(ref default) = template_default_filename {
                    ui.label(RichText::new(format!("(använder: {})", default)).small().color(Colors::TEXT_MUTED));
                }
            } else if !self.text_filename.contains('.') {
                ui.label(RichText::new("(.txt läggs till automatiskt)").small().color(Colors::TEXT_MUTED));
            }
        });

        ui.add_space(8.0);

        // Textinnehåll
        ui.label("Innehåll:");
        ui.add_space(4.0);

        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.text_content)
                        .hint_text("Skriv text här...")
                        .desired_width(f32::INFINITY)
                        .desired_rows(10)
                        .font(egui::TextStyle::Monospace)
                );
            });

        ui.add_space(4.0);
        ui.label(
            RichText::new(format!("{} tecken", self.text_content.len()))
                .small()
                .color(Colors::TEXT_MUTED)
        );
    }

    /// Importera filer till personens katalog
    fn import_files(&mut self, db: &Database, person: &Person) -> anyhow::Result<()> {
        if self.selected_files.is_empty() {
            return Err(anyhow::anyhow!("Inga filer valda"));
        }

        let type_id = self.selected_type_id
            .ok_or_else(|| anyhow::anyhow!("Ingen dokumenttyp vald"))?;

        let doc_type = db.documents()
            .get_type_by_id(type_id)?
            .ok_or_else(|| anyhow::anyhow!("Dokumenttyp hittades inte"))?;

        let config = db.config().get()?;
        let person_dir = config.persons_directory().join(&person.directory_name);
        let target_dir = person_dir.join(&doc_type.target_directory);

        self.uploading = true;
        self.import_success_count = 0;
        self.import_failed_count = 0;

        for i in 0..self.selected_files.len() {
            let source_path = self.selected_files[i].path.clone();
            let filename = self.selected_files[i].filename.clone();

            let unique_name = file_ops::unique_filename(&target_dir, &filename);

            match file_ops::copy_file_to_directory(&source_path, &target_dir, &unique_name) {
                Ok(dest_path) => {
                    let file_size = file_ops::get_file_size(&dest_path).unwrap_or(0) as i64;
                    let file_type = file_ops::get_file_extension(&dest_path);
                    let file_modified = file_ops::get_modified_time(&dest_path).ok();

                    let relative_path = format!("{}/{}", doc_type.target_directory, unique_name);

                    let mut document = Document {
                        id: None,
                        person_id: person.id.unwrap_or(0),
                        document_type_id: Some(type_id),
                        filename: unique_name.clone(),
                        relative_path,
                        file_size,
                        file_type,
                        file_modified_at: file_modified,
                        created_at: None,
                        updated_at: None,
                    };

                    if db.documents().create(&mut document).is_ok() {
                        self.selected_files[i].status = ImportStatus::Done;
                        self.import_success_count += 1;
                        tracing::info!("Fil importerad: {:?}", dest_path);
                    } else {
                        self.selected_files[i].status = ImportStatus::Failed;
                        self.import_failed_count += 1;
                    }
                }
                Err(e) => {
                    tracing::error!("Kunde inte importera {}: {}", filename, e);
                    self.selected_files[i].status = ImportStatus::Failed;
                    self.import_failed_count += 1;
                }
            }
        }

        self.uploading = false;

        if self.import_success_count == 0 {
            return Err(anyhow::anyhow!("Alla importer misslyckades"));
        }

        Ok(())
    }

    /// Skapa textdokument från inmatat innehåll
    fn create_text_document(&mut self, db: &Database, person: &Person) -> anyhow::Result<()> {
        if self.text_content.is_empty() {
            return Err(anyhow::anyhow!("Inget textinnehåll"));
        }

        let type_id = self.selected_type_id
            .ok_or_else(|| anyhow::anyhow!("Ingen dokumenttyp vald"))?;

        let doc_type = db.documents()
            .get_type_by_id(type_id)?
            .ok_or_else(|| anyhow::anyhow!("Dokumenttyp hittades inte"))?;

        // Bestäm filnamn: använd användarens input, eller mallens default
        let base_filename = if !self.text_filename.is_empty() {
            self.text_filename.clone()
        } else if let Some(ref default_name) = doc_type.default_filename {
            default_name.clone()
        } else {
            return Err(anyhow::anyhow!("Inget filnamn angivet"));
        };

        // Lägg till .txt om det saknas
        let filename = if base_filename.contains('.') {
            base_filename
        } else {
            format!("{}.txt", base_filename)
        };

        let config = db.config().get()?;
        let person_dir = config.persons_directory().join(&person.directory_name);
        let target_dir = person_dir.join(&doc_type.target_directory);

        std::fs::create_dir_all(&target_dir)?;

        let unique_name = file_ops::unique_filename(&target_dir, &filename);
        let dest_path = target_dir.join(&unique_name);
        std::fs::write(&dest_path, &self.text_content)?;

        let file_size = self.text_content.len() as i64;
        let file_type = Some("txt".to_string());
        let file_modified = file_ops::get_modified_time(&dest_path).ok();
        let relative_path = format!("{}/{}", doc_type.target_directory, unique_name);

        let mut document = Document {
            id: None,
            person_id: person.id.ok_or_else(|| anyhow::anyhow!("Person saknar ID"))?,
            document_type_id: Some(type_id),
            filename: unique_name,
            relative_path,
            file_size,
            file_type,
            file_modified_at: file_modified,
            created_at: None,
            updated_at: None,
        };

        db.documents().create(&mut document)?;

        tracing::info!("Textdokument skapat: {:?}", dest_path);

        Ok(())
    }
}
