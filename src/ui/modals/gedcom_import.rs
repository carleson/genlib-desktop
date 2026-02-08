//! Modal för GEDCOM-import

use std::path::PathBuf;

use egui::{self, RichText};

use crate::db::Database;
use crate::gedcom::{GedcomData, GedcomImporter, GedcomParser, ImportPreview, ImportResult};
use crate::ui::{
    state::AppState,
    theme::{Colors, Icons},
};

/// Importsteg
#[derive(Debug, Clone, PartialEq)]
enum ImportStep {
    /// Välj fil
    SelectFile,
    /// Förhandsgranskning
    Preview,
    /// Importerar
    Importing,
    /// Klar
    Done,
}

/// Modal för GEDCOM-import
pub struct GedcomImportModal {
    /// Aktuellt steg
    step: ImportStep,
    /// Vald fil
    selected_file: Option<PathBuf>,
    /// Parsad GEDCOM-data
    gedcom_data: Option<GedcomData>,
    /// Förhandsgranskning
    preview: Option<ImportPreview>,
    /// Importresultat
    result: Option<ImportResult>,
    /// Felmeddelande
    error: Option<String>,
}

impl Default for GedcomImportModal {
    fn default() -> Self {
        Self::new()
    }
}

impl GedcomImportModal {
    pub fn new() -> Self {
        Self {
            step: ImportStep::SelectFile,
            selected_file: None,
            gedcom_data: None,
            preview: None,
            result: None,
            error: None,
        }
    }

    /// Återställ modal
    pub fn reset(&mut self) {
        self.step = ImportStep::SelectFile;
        self.selected_file = None;
        self.gedcom_data = None;
        self.preview = None;
        self.result = None;
        self.error = None;
    }

    /// Visa modalen. Returnerar true om den ska stängas.
    pub fn show(&mut self, ctx: &egui::Context, state: &mut AppState, db: &Database) -> bool {
        let mut should_close = false;

        egui::Window::new(format!("{} GEDCOM-import", Icons::DOCUMENT))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(500.0);

                match self.step {
                    ImportStep::SelectFile => {
                        should_close = self.show_select_file(ui, db);
                    }
                    ImportStep::Preview => {
                        should_close = self.show_preview(ui, state, db);
                    }
                    ImportStep::Importing => {
                        self.show_importing(ui);
                    }
                    ImportStep::Done => {
                        should_close = self.show_done(ui, state);
                    }
                }
            });

        should_close
    }

    fn show_select_file(&mut self, ui: &mut egui::Ui, db: &Database) -> bool {
        ui.heading("Välj GEDCOM-fil");
        ui.add_space(8.0);

        ui.label("Välj en GEDCOM-fil (.ged) att importera:");
        ui.add_space(16.0);

        // Visa vald fil
        if let Some(ref path) = self.selected_file {
            ui.horizontal(|ui| {
                ui.label(Icons::DOCUMENT);
                ui.label(path.file_name().unwrap_or_default().to_string_lossy().to_string());
            });
            ui.add_space(8.0);
        }

        // Felmeddelande
        if let Some(ref error) = self.error {
            ui.label(RichText::new(error).color(Colors::ERROR));
            ui.add_space(8.0);
        }

        // Knappar
        ui.horizontal(|ui| {
            if ui.button(format!("{} Välj fil...", Icons::FOLDER)).clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("GEDCOM", &["ged", "GED"])
                    .pick_file()
                {
                    self.selected_file = Some(path.clone());
                    self.error = None;

                    // Försök parsa filen
                    match GedcomParser::parse_file(&path) {
                        Ok(data) => {
                            let importer = GedcomImporter::new(db);
                            let preview = importer.preview(&data);
                            self.preview = Some(preview);
                            self.gedcom_data = Some(data);
                            self.step = ImportStep::Preview;
                        }
                        Err(e) => {
                            self.error = Some(format!("Kunde inte läsa filen: {}", e));
                        }
                    }
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Avbryt").clicked() {
                    self.reset();
                    return;
                }
            });
        });

        false
    }

    fn show_preview(&mut self, ui: &mut egui::Ui, _state: &mut AppState, db: &Database) -> bool {
        let mut should_close = false;

        ui.heading("Förhandsgranskning");
        ui.add_space(8.0);

        if let Some(ref preview) = self.preview {
            // Statistik
            egui::Frame::none()
                .fill(ui.visuals().extreme_bg_color)
                .rounding(4.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    egui::Grid::new("preview_stats")
                        .num_columns(2)
                        .spacing([16.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Individer i fil:");
                            ui.label(RichText::new(format!("{}", preview.total_individuals)).strong());
                            ui.end_row();

                            ui.label("Familjer i fil:");
                            ui.label(RichText::new(format!("{}", preview.total_families)).strong());
                            ui.end_row();

                            ui.label("Nya personer:");
                            ui.label(
                                RichText::new(format!("{}", preview.new_persons))
                                    .color(Colors::SUCCESS),
                            );
                            ui.end_row();

                            ui.label("Befintliga (överhoppas):");
                            ui.label(
                                RichText::new(format!("{}", preview.existing_persons))
                                    .color(Colors::TEXT_MUTED),
                            );
                            ui.end_row();

                            ui.label("Uppskattade relationer:");
                            ui.label(format!("{}", preview.estimated_relations));
                            ui.end_row();
                        });
                });

            ui.add_space(12.0);

            // Exempel på personer
            if !preview.sample_persons.is_empty() {
                ui.label(RichText::new("Exempel på personer:").strong());
                ui.add_space(4.0);

                for person in &preview.sample_persons {
                    ui.horizontal(|ui| {
                        ui.label(Icons::PERSON);
                        ui.label(&person.name);

                        if let Some(ref birth) = person.birth_year {
                            ui.label(
                                RichText::new(format!("f. {}", birth))
                                    .small()
                                    .color(Colors::TEXT_MUTED),
                            );
                        }

                        if let Some(ref death) = person.death_year {
                            ui.label(
                                RichText::new(format!("d. {}", death))
                                    .small()
                                    .color(Colors::TEXT_MUTED),
                            );
                        }
                    });
                }
            }
        }

        // Felmeddelande
        if let Some(ref error) = self.error {
            ui.add_space(8.0);
            ui.label(RichText::new(error).color(Colors::ERROR));
        }

        ui.add_space(16.0);

        // Knappar
        ui.horizontal(|ui| {
            if ui.button(format!("{} Tillbaka", Icons::ARROW_LEFT)).clicked() {
                self.step = ImportStep::SelectFile;
                self.error = None;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Avbryt").clicked() {
                    self.reset();
                    should_close = true;
                    return;
                }

                let can_import = self.preview.as_ref().map(|p| p.new_persons > 0).unwrap_or(false);

                ui.add_enabled_ui(can_import, |ui| {
                    if ui
                        .button(RichText::new(format!("{} Importera", Icons::SAVE)).strong())
                        .clicked()
                    {
                        self.step = ImportStep::Importing;

                        // Utför importen
                        if let Some(ref data) = self.gedcom_data {
                            let importer = GedcomImporter::new(db);
                            match importer.import_data(data) {
                                Ok(result) => {
                                    self.result = Some(result);
                                    self.step = ImportStep::Done;
                                }
                                Err(e) => {
                                    self.error = Some(format!("Import misslyckades: {}", e));
                                    self.step = ImportStep::Preview;
                                }
                            }
                        }
                    }
                });

                if !can_import {
                    ui.label(
                        RichText::new("Inga nya personer att importera")
                            .small()
                            .color(Colors::TEXT_MUTED),
                    );
                }
            });
        });

        should_close
    }

    fn show_importing(&mut self, ui: &mut egui::Ui) {
        ui.heading("Importerar...");
        ui.add_space(16.0);

        ui.horizontal(|ui| {
            ui.spinner();
            ui.label("Importerar data, vänta...");
        });
    }

    fn show_done(&mut self, ui: &mut egui::Ui, state: &mut AppState) -> bool {
        let mut should_close = false;

        ui.heading(format!("{} Import klar!", Icons::CHECK));
        ui.add_space(16.0);

        if let Some(ref result) = self.result {
            egui::Frame::none()
                .fill(ui.visuals().extreme_bg_color)
                .rounding(4.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    egui::Grid::new("result_stats")
                        .num_columns(2)
                        .spacing([16.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Personer importerade:");
                            ui.label(
                                RichText::new(format!("{}", result.persons_imported))
                                    .strong()
                                    .color(Colors::SUCCESS),
                            );
                            ui.end_row();

                            ui.label("Relationer skapade:");
                            ui.label(
                                RichText::new(format!("{}", result.relations_imported))
                                    .strong()
                                    .color(Colors::SUCCESS),
                            );
                            ui.end_row();

                            if result.skipped > 0 {
                                ui.label("Överhoppade:");
                                ui.label(
                                    RichText::new(format!("{}", result.skipped))
                                        .color(Colors::TEXT_MUTED),
                                );
                                ui.end_row();
                            }
                        });
                });

            // Visa varningar
            if !result.warnings.is_empty() {
                ui.add_space(8.0);
                ui.collapsing(
                    RichText::new(format!("Varningar ({})", result.warnings.len()))
                        .color(Colors::WARNING),
                    |ui| {
                        for warning in &result.warnings {
                            ui.label(RichText::new(warning).small().color(Colors::WARNING));
                        }
                    },
                );
            }
        }

        ui.add_space(16.0);

        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Stäng").clicked() {
                    state.show_success("GEDCOM-import slutförd!");
                    self.reset();
                    should_close = true;
                }
            });
        });

        should_close
    }
}
