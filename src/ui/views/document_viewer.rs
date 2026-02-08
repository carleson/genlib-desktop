//! Dokumentvisningsvy
//!
//! Visar och redigerar dokument (bilder, text, PDF-metadata).

use egui::{self, RichText, TextureHandle};

use crate::db::Database;
use crate::models::{Document, DocumentType, Person};
use crate::ui::{
    state::{AppState, ConfirmAction},
    theme::{Colors, Icons},
    View,
};
use crate::utils::file_ops;

/// Tillstånd för dokumentvisning
pub struct DocumentViewerView {
    /// Aktuellt dokument
    document: Option<Document>,
    /// ID på aktuellt laddat dokument
    loaded_document_id: Option<i64>,
    /// Dokumenttyp (cachad)
    document_type: Option<DocumentType>,
    /// Person som dokumentet tillhör
    person: Option<Person>,
    /// Textinnehåll (för textfiler)
    text_content: Option<String>,
    /// Om texten har ändrats
    text_modified: bool,
    /// Laddad bildtextur
    image_texture: Option<TextureHandle>,
    /// Felmeddelande
    error_message: Option<String>,
    /// Behöver laddas om
    needs_refresh: bool,
    /// Redigerar metadata
    editing_metadata: bool,
    /// Metadata som redigeras
    edit_tags: String,
}

impl DocumentViewerView {
    pub fn new() -> Self {
        Self {
            document: None,
            loaded_document_id: None,
            document_type: None,
            person: None,
            text_content: None,
            text_modified: false,
            image_texture: None,
            error_message: None,
            needs_refresh: true,
            editing_metadata: false,
            edit_tags: String::new(),
        }
    }

    /// Ladda ett dokument för visning
    pub fn load_document(&mut self, document_id: i64, db: &Database) {
        self.needs_refresh = true;
        self.loaded_document_id = Some(document_id);

        // Hämta dokument
        match db.documents().find_by_id(document_id) {
            Ok(Some(doc)) => {
                // Hämta dokumenttyp
                if let Some(type_id) = doc.document_type_id {
                    self.document_type = db.documents().get_type_by_id(type_id).ok().flatten();
                }

                // Hämta person
                self.person = db.persons().find_by_id(doc.person_id).ok().flatten();

                // Ladda innehåll baserat på typ
                self.load_content(&doc, db);

                self.edit_tags = doc.tags.clone().unwrap_or_default();
                self.document = Some(doc);
                self.error_message = None;
            }
            Ok(None) => {
                self.error_message = Some("Dokument hittades inte".to_string());
                self.document = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Kunde inte ladda dokument: {}", e));
                self.document = None;
            }
        }

        self.needs_refresh = false;
    }

    fn load_content(&mut self, doc: &Document, db: &Database) {
        // Rensa tidigare innehåll
        self.text_content = None;
        self.text_modified = false;
        self.image_texture = None;

        // Hämta fullständig sökväg
        let Some(ref person) = self.person else {
            self.error_message = Some("Person saknas".to_string());
            return;
        };

        let config = match db.config().get() {
            Ok(c) => c,
            Err(e) => {
                self.error_message = Some(format!("Kunde inte hämta config: {}", e));
                return;
            }
        };

        let full_path = doc.full_path(&config.media_directory_path, &person.directory_name);

        tracing::debug!("Försöker ladda dokument från: {:?}", full_path);
        tracing::debug!("  media_root: {:?}", config.media_directory_path);
        tracing::debug!("  person_dir: {}", person.directory_name);
        tracing::debug!("  relative_path: {}", doc.relative_path);
        tracing::debug!("  file_type: {:?}", doc.file_type);

        if !full_path.exists() {
            self.error_message = Some(format!("Filen finns inte: {:?}", full_path));
            return;
        }

        // Ladda baserat på filtyp
        if doc.is_text() {
            match file_ops::read_text_file(&full_path) {
                Ok(content) => {
                    tracing::debug!("Laddade textfil med {} tecken", content.len());
                    self.text_content = Some(content);
                }
                Err(e) => {
                    self.error_message = Some(format!("Kunde inte läsa textfil: {}", e));
                }
            }
        }
        // Bildladdning hanteras separat i show() för att ha tillgång till egui context
    }

    /// Visa dokumentet
    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) {
        // Ladda dokumentet om det är ett nytt eller om vi behöver refresh
        if let Some(doc_id) = state.selected_document_id {
            if self.loaded_document_id != Some(doc_id) || self.needs_refresh {
                self.load_document(doc_id, db);
            }
        }

        // Header med tillbaka-knapp
        ui.horizontal(|ui| {
            if ui.button(format!("{} Tillbaka", Icons::ARROW_LEFT)).clicked() {
                state.navigate(View::PersonDetail);
            }

            ui.separator();

            if let Some(ref doc) = self.document {
                ui.heading(&doc.filename);
            } else {
                ui.heading("Dokument");
            }
        });

        ui.add_space(8.0);

        // Felmeddelande
        if let Some(ref error) = self.error_message {
            ui.label(RichText::new(error).color(Colors::ERROR));
            return;
        }

        let Some(doc) = self.document.clone() else {
            ui.label("Inget dokument laddat");
            return;
        };

        // Layout med metadata-panel och innehåll
        ui.columns(2, |columns| {
            // Vänster kolumn - Metadata
            self.show_metadata_panel(&mut columns[0], &doc, state, db);

            // Höger kolumn - Innehåll
            self.show_content_panel(&mut columns[1], &doc, db);
        });
    }

    fn show_metadata_panel(
        &mut self,
        ui: &mut egui::Ui,
        doc: &Document,
        state: &mut AppState,
        db: &Database,
    ) {
        egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .rounding(8.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.heading("Information");
                ui.add_space(8.0);

                egui::Grid::new("doc_metadata_grid")
                    .num_columns(2)
                    .spacing([16.0, 8.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Filnamn:").color(Colors::TEXT_SECONDARY));
                        ui.label(&doc.filename);
                        ui.end_row();

                        if let Some(ref doc_type) = self.document_type {
                            ui.label(RichText::new("Typ:").color(Colors::TEXT_SECONDARY));
                            ui.label(&doc_type.name);
                            ui.end_row();
                        }

                        ui.label(RichText::new("Storlek:").color(Colors::TEXT_SECONDARY));
                        ui.label(doc.file_size_display());
                        ui.end_row();

                        if let Some(ref file_type) = doc.file_type {
                            ui.label(RichText::new("Format:").color(Colors::TEXT_SECONDARY));
                            ui.label(file_type.to_uppercase());
                            ui.end_row();
                        }

                        if let Some(ref person) = self.person {
                            ui.label(RichText::new("Person:").color(Colors::TEXT_SECONDARY));
                            if ui.link(person.full_name()).clicked() {
                                state.navigate_to_person(person.id.unwrap_or(0));
                            }
                            ui.end_row();
                        }

                        ui.label(RichText::new("Sökväg:").color(Colors::TEXT_SECONDARY));
                        ui.label(RichText::new(&doc.relative_path).small());
                        ui.end_row();
                    });

                ui.add_space(16.0);

                // Taggar
                ui.label(RichText::new("Taggar:").color(Colors::TEXT_SECONDARY));
                if self.editing_metadata {
                    ui.text_edit_singleline(&mut self.edit_tags);
                } else {
                    let tags_display = doc.tags.as_deref().unwrap_or("(inga taggar)");
                    ui.label(tags_display);
                }

                ui.add_space(16.0);

                // Åtgärdsknappar
                ui.horizontal(|ui| {
                    if self.editing_metadata {
                        if ui.button(format!("{} Spara", Icons::SAVE)).clicked() {
                            self.save_metadata(db);
                        }
                        if ui.button("Avbryt").clicked() {
                            self.editing_metadata = false;
                            if let Some(ref doc) = self.document {
                                self.edit_tags = doc.tags.clone().unwrap_or_default();
                            }
                        }
                    } else {
                        if ui.button(format!("{} Redigera", Icons::EDIT)).clicked() {
                            self.editing_metadata = true;
                        }
                    }
                });

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if ui.button(format!("{} Öppna extern", Icons::EXPORT)).clicked() {
                        self.open_in_external_app(db);
                    }

                    if ui
                        .button(RichText::new(format!("{} Radera", Icons::DELETE)).color(Colors::ERROR))
                        .clicked()
                    {
                        if let Some(id) = doc.id {
                            state.show_confirm(
                                &format!("Vill du verkligen radera {}?", doc.filename),
                                ConfirmAction::DeleteDocument(id),
                            );
                        }
                    }
                });
            });
    }

    fn show_content_panel(&mut self, ui: &mut egui::Ui, doc: &Document, db: &Database) {
        egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .rounding(8.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.heading("Innehåll");
                ui.add_space(8.0);

                // Debug: visa filtyp
                let file_type_str = doc.file_type.as_deref().unwrap_or("(ingen)");
                ui.label(RichText::new(format!("Filtyp: {}", file_type_str)).small().color(Colors::TEXT_MUTED));

                if doc.is_text() {
                    self.show_text_content(ui, db);
                } else if doc.is_image() {
                    self.show_image_content(ui, doc, db);
                } else if doc.is_pdf() {
                    self.show_pdf_placeholder(ui);
                } else {
                    ui.label("Förhandsgranskning stöds inte för denna filtyp.");
                    ui.add_space(8.0);
                    ui.label("Använd 'Öppna extern' för att visa filen.");
                }
            });
    }

    fn show_text_content(&mut self, ui: &mut egui::Ui, db: &Database) {
        if let Some(ref mut content) = self.text_content {
            // Visa antal tecken
            ui.label(RichText::new(format!("{} tecken", content.len())).small().color(Colors::TEXT_MUTED));
            ui.add_space(4.0);

            // Redigerbar textyta
            let response = egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(content)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .desired_rows(20),
                    )
                });

            if response.inner.changed() {
                self.text_modified = true;
            }

            ui.add_space(8.0);

            // Spara-knapp om modifierad
            if self.text_modified {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Osparade ändringar").color(Colors::WARNING));
                    if ui.button(format!("{} Spara", Icons::SAVE)).clicked() {
                        self.save_text_content(db);
                    }
                });
            }
        } else {
            ui.label(RichText::new("Textinnehåll kunde inte laddas").color(Colors::ERROR));
            ui.add_space(4.0);
            if let Some(ref error) = self.error_message {
                ui.label(RichText::new(error).small().color(Colors::TEXT_MUTED));
            }
        }
    }

    fn show_image_content(&mut self, ui: &mut egui::Ui, doc: &Document, db: &Database) {
        // Försök ladda bild om inte redan laddad
        if self.image_texture.is_none() {
            if let Some(ref person) = self.person {
                if let Ok(config) = db.config().get() {
                    let full_path = doc.full_path(&config.media_directory_path, &person.directory_name);

                    if let Ok(image_data) = std::fs::read(&full_path) {
                        if let Ok(image) = image::load_from_memory(&image_data) {
                            let size = [image.width() as _, image.height() as _];
                            let image_buffer = image.to_rgba8();
                            let pixels = image_buffer.as_flat_samples();

                            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                size,
                                pixels.as_slice(),
                            );

                            self.image_texture = Some(ui.ctx().load_texture(
                                "document_image",
                                color_image,
                                egui::TextureOptions::LINEAR,
                            ));
                        }
                    }
                }
            }
        }

        // Visa bild
        if let Some(ref texture) = self.image_texture {
            let available_size = ui.available_size();
            let texture_size = texture.size_vec2();

            // Skala ner om bilden är för stor
            let scale = (available_size.x / texture_size.x)
                .min(available_size.y / texture_size.y)
                .min(1.0);

            let display_size = texture_size * scale;

            egui::ScrollArea::both().show(ui, |ui| {
                ui.image((texture.id(), display_size));
            });
        } else {
            ui.label("Kunde inte ladda bild");
        }
    }

    fn show_pdf_placeholder(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.label(RichText::new(Icons::DOCUMENT).size(64.0));
            ui.add_space(16.0);
            ui.label("PDF-förhandsgranskning stöds inte inbyggt.");
            ui.add_space(8.0);
            ui.label("Klicka på 'Öppna extern' för att visa i PDF-läsare.");
        });
    }

    fn save_metadata(&mut self, db: &Database) {
        if let Some(ref mut doc) = self.document {
            let tags = if self.edit_tags.trim().is_empty() {
                None
            } else {
                Some(self.edit_tags.clone())
            };

            doc.tags = tags;

            if let Err(e) = db.documents().update(doc) {
                self.error_message = Some(format!("Kunde inte spara: {}", e));
            } else {
                self.editing_metadata = false;
            }
        }
    }

    fn save_text_content(&mut self, db: &Database) {
        let Some(ref doc) = self.document else {
            return;
        };
        let Some(ref person) = self.person else {
            return;
        };
        let Some(ref content) = self.text_content else {
            return;
        };

        let config = match db.config().get() {
            Ok(c) => c,
            Err(e) => {
                self.error_message = Some(format!("Kunde inte hämta config: {}", e));
                return;
            }
        };

        let full_path = doc.full_path(&config.media_directory_path, &person.directory_name);

        match file_ops::write_text_file(&full_path, content) {
            Ok(_) => {
                self.text_modified = false;
                tracing::info!("Textfil sparad: {:?}", full_path);
            }
            Err(e) => {
                self.error_message = Some(format!("Kunde inte spara: {}", e));
            }
        }
    }

    fn open_in_external_app(&self, db: &Database) {
        let Some(ref doc) = self.document else {
            return;
        };
        let Some(ref person) = self.person else {
            return;
        };

        let config = match db.config().get() {
            Ok(c) => c,
            Err(_) => return,
        };

        let full_path = doc.full_path(&config.media_directory_path, &person.directory_name);

        // Öppna med systemets standardprogram
        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open")
                .arg(&full_path)
                .spawn();
        }

        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open")
                .arg(&full_path)
                .spawn();
        }

        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("explorer")
                .arg(&full_path)
                .spawn();
        }
    }

    pub fn mark_needs_refresh(&mut self) {
        self.needs_refresh = true;
        self.loaded_document_id = None;
    }
}
