//! Dokumentmallsvy för att hantera dokumenttyper
//!
//! Visar och redigerar dokumenttyper med fördefinierade namn och platser.

use egui::{self, RichText};

use crate::db::Database;
use crate::models::DocumentType;
use crate::ui::{
    state::AppState,
    theme::{Colors, Icons},
    View,
};

pub struct DocumentTemplatesView {
    /// Cachade dokumenttyper
    document_types: Vec<DocumentType>,
    /// Behöver refresh
    needs_refresh: bool,
    /// Redigerar dokumenttyp (None = ny, Some(id) = redigera)
    editing_type_id: Option<i64>,
    /// Formulärdata
    form_name: String,
    form_target_directory: String,
    form_default_filename: String,
    form_description: String,
    /// Visar formulär
    show_form: bool,
    /// Felmeddelande
    error_message: Option<String>,
}

impl Default for DocumentTemplatesView {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentTemplatesView {
    pub fn new() -> Self {
        Self {
            document_types: Vec::new(),
            needs_refresh: true,
            editing_type_id: None,
            form_name: String::new(),
            form_target_directory: String::new(),
            form_default_filename: String::new(),
            form_description: String::new(),
            show_form: false,
            error_message: None,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) {
        // Refresh om nödvändigt
        if self.needs_refresh {
            self.refresh(db);
            self.needs_refresh = false;
        }

        // Header
        ui.horizontal(|ui| {
            if ui.button(format!("{} Tillbaka", Icons::ARROW_LEFT)).clicked() {
                state.navigate(View::Settings);
            }
            ui.separator();
            ui.heading(format!("{} Dokumentmallar", Icons::DOCUMENT));
        });

        ui.add_space(8.0);
        ui.label("Hantera dokumenttyper som kan användas vid uppladdning. Varje dokumenttyp har en målkatalog och ett fördefinierat filnamn.");

        ui.add_space(16.0);

        // Formulär för ny/redigera dokumenttyp
        if self.show_form {
            self.show_edit_form(ui, db);
            ui.add_space(16.0);
        }

        // Lista med dokumenttyper
        self.show_document_types_list(ui, db);
    }

    fn show_edit_form(&mut self, ui: &mut egui::Ui, db: &Database) {
        let is_new = self.editing_type_id.is_none();
        let title = if is_new { "Ny dokumenttyp" } else { "Redigera dokumenttyp" };

        egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .rounding(8.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.heading(title);
                ui.add_space(8.0);

                egui::Grid::new("doc_type_form_grid")
                    .num_columns(2)
                    .spacing([16.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Namn:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.form_name)
                                .hint_text("t.ex. Personbevis")
                                .desired_width(250.0)
                        );
                        ui.end_row();

                        ui.label("Målkatalog:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.form_target_directory)
                                .hint_text("t.ex. dokument/personbevis")
                                .desired_width(250.0)
                        );
                        ui.end_row();

                        ui.label("Fördefinierat filnamn:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.form_default_filename)
                                .hint_text("t.ex. personbevis.pdf (valfritt)")
                                .desired_width(250.0)
                        );
                        ui.end_row();

                        ui.label("Beskrivning:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.form_description)
                                .hint_text("Valfri beskrivning")
                                .desired_width(250.0)
                        );
                        ui.end_row();
                    });

                if let Some(ref error) = self.error_message {
                    ui.add_space(8.0);
                    ui.label(RichText::new(error).color(Colors::ERROR));
                }

                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    if ui.button("Avbryt").clicked() {
                        self.close_form();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let button_text = if is_new {
                            format!("{} Skapa", Icons::ADD)
                        } else {
                            format!("{} Spara", Icons::SAVE)
                        };

                        if ui.button(button_text).clicked() {
                            self.save_document_type(db);
                        }
                    });
                });
            });
    }

    fn show_document_types_list(&mut self, ui: &mut egui::Ui, db: &Database) {
        egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .rounding(8.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Dokumenttyper");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(format!("{} Ny dokumenttyp", Icons::ADD)).clicked() {
                            self.open_new_form();
                        }
                    });
                });

                ui.add_space(8.0);

                if self.document_types.is_empty() {
                    ui.label(RichText::new("Inga dokumenttyper definierade").color(Colors::TEXT_MUTED));
                    return;
                }

                // Tabell-header
                egui::Grid::new("doc_types_header")
                    .num_columns(5)
                    .spacing([16.0, 4.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Namn").strong());
                        ui.label(RichText::new("Målkatalog").strong());
                        ui.label(RichText::new("Fördefinierat filnamn").strong());
                        ui.label(RichText::new("Beskrivning").strong());
                        ui.label(""); // Actions
                        ui.end_row();
                    });

                ui.separator();

                // Dokumenttyper
                egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                    let types_data: Vec<_> = self.document_types.iter().map(|dt| {
                        (
                            dt.id,
                            dt.name.clone(),
                            dt.target_directory.clone(),
                            dt.default_filename.clone().unwrap_or_default(),
                            dt.description.clone().unwrap_or_default(),
                        )
                    }).collect();

                    for (id, name, target_dir, default_file, description) in types_data {
                        ui.horizontal(|ui| {
                            ui.set_min_width(ui.available_width());

                            egui::Grid::new(format!("doc_type_row_{:?}", id))
                                .num_columns(5)
                                .spacing([16.0, 4.0])
                                .show(ui, |ui| {
                                    ui.label(&name);
                                    ui.label(RichText::new(&target_dir).small());
                                    ui.label(RichText::new(&default_file).small().color(Colors::TEXT_MUTED));
                                    ui.label(RichText::new(&description).small().color(Colors::TEXT_MUTED));

                                    ui.horizontal(|ui| {
                                        if ui.small_button(Icons::EDIT).on_hover_text("Redigera").clicked() {
                                            if let Some(dt_id) = id {
                                                self.open_edit_form(dt_id, db);
                                            }
                                        }
                                        if ui.small_button(RichText::new(Icons::DELETE).color(Colors::ERROR))
                                            .on_hover_text("Ta bort")
                                            .clicked()
                                        {
                                            if let Some(dt_id) = id {
                                                self.delete_document_type(dt_id, db);
                                            }
                                        }
                                    });
                                });
                        });

                        ui.add_space(4.0);
                    }
                });
            });
    }

    fn open_new_form(&mut self) {
        self.editing_type_id = None;
        self.form_name.clear();
        self.form_target_directory.clear();
        self.form_default_filename.clear();
        self.form_description.clear();
        self.error_message = None;
        self.show_form = true;
    }

    fn open_edit_form(&mut self, type_id: i64, db: &Database) {
        if let Ok(Some(dt)) = db.documents().get_type_by_id(type_id) {
            self.editing_type_id = Some(type_id);
            self.form_name = dt.name;
            self.form_target_directory = dt.target_directory;
            self.form_default_filename = dt.default_filename.unwrap_or_default();
            self.form_description = dt.description.unwrap_or_default();
            self.error_message = None;
            self.show_form = true;
        }
    }

    fn close_form(&mut self) {
        self.show_form = false;
        self.editing_type_id = None;
        self.error_message = None;
    }

    fn save_document_type(&mut self, db: &Database) {
        // Validera
        if self.form_name.trim().is_empty() {
            self.error_message = Some("Namn krävs".to_string());
            return;
        }
        if self.form_target_directory.trim().is_empty() {
            self.error_message = Some("Målkatalog krävs".to_string());
            return;
        }

        let doc_type = DocumentType {
            id: self.editing_type_id,
            name: self.form_name.trim().to_string(),
            target_directory: self.form_target_directory.trim().to_string(),
            default_filename: if self.form_default_filename.trim().is_empty() {
                None
            } else {
                Some(self.form_default_filename.trim().to_string())
            },
            description: if self.form_description.trim().is_empty() {
                None
            } else {
                Some(self.form_description.trim().to_string())
            },
        };

        let result = if self.editing_type_id.is_some() {
            db.documents().update_type(&doc_type)
        } else {
            db.documents().create_type(&doc_type).map(|_| ())
        };

        match result {
            Ok(()) => {
                self.close_form();
                self.needs_refresh = true;
            }
            Err(e) => {
                self.error_message = Some(format!("Kunde inte spara: {}", e));
            }
        }
    }

    fn delete_document_type(&mut self, type_id: i64, db: &Database) {
        if let Err(e) = db.documents().delete_type(type_id) {
            self.error_message = Some(format!("Kunde inte ta bort: {}", e));
        } else {
            self.needs_refresh = true;
        }
    }

    fn refresh(&mut self, db: &Database) {
        self.document_types = db.documents().get_all_types().unwrap_or_default();
    }

    pub fn mark_needs_refresh(&mut self) {
        self.needs_refresh = true;
    }
}
