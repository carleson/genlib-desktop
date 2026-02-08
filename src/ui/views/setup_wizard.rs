//! Setup wizard för första start

use std::path::PathBuf;

use egui::{self, RichText};

use crate::db::Database;
use crate::models::SystemConfig;
use crate::ui::{
    state::AppState,
    theme::{Colors, Icons},
    View,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WizardStep {
    Media,
    Backup,
    Done,
}

pub struct SetupWizardView {
    step: WizardStep,
    media_path: String,
    backup_path: String,
    needs_refresh: bool,
    error_message: Option<String>,
}

impl Default for SetupWizardView {
    fn default() -> Self {
        Self::new()
    }
}

impl SetupWizardView {
    pub fn new() -> Self {
        Self {
            step: WizardStep::Media,
            media_path: String::new(),
            backup_path: String::new(),
            needs_refresh: true,
            error_message: None,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) {
        if self.needs_refresh {
            self.refresh_from_config(db);
        }

        ui.vertical(|ui| {
            ui.heading(format!("{} Setup Wizard", Icons::SETTINGS));
            ui.add_space(12.0);

            ui.label(RichText::new(self.step_title()).strong());
            ui.add_space(8.0);

            if let Some(ref error) = self.error_message {
                ui.label(RichText::new(error).color(Colors::ERROR));
                ui.add_space(8.0);
            }

            match self.step {
                WizardStep::Media => self.show_media_step(ui),
                WizardStep::Backup => self.show_backup_step(ui),
                WizardStep::Done => self.show_done_step(ui, state, db),
            }

            ui.add_space(12.0);
            self.show_step_controls(ui, state, db);
        });
    }

    fn show_media_step(&mut self, ui: &mut egui::Ui) {
        ui.label("Välj en katalog där media (bilder, dokument) ska lagras.");
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.media_path).desired_width(360.0));
            if ui.button("Välj...").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.media_path = path.display().to_string();
                }
            }
        });
    }

    fn show_backup_step(&mut self, ui: &mut egui::Ui) {
        ui.label("Välj en katalog för backuper.");
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.backup_path).desired_width(360.0));
            if ui.button("Välj...").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.backup_path = path.display().to_string();
                }
            }
        });
    }

    fn show_done_step(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) {
        ui.label("Klart! Du kan nu börja använda Genlib.");
        ui.add_space(8.0);

        ui.label(format!("Media: {}", self.media_path));
        ui.label(format!("Backup: {}", self.backup_path));

        ui.add_space(12.0);
        ui.label(RichText::new("Valfritt").strong());
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            if ui.button(format!("{} Importera GEDCOM", Icons::IMPORT)).clicked() {
                if self.save_config(db).is_ok() {
                    state.show_gedcom_import = true;
                }
            }

            if ui.button(format!("{} Återställ backup", Icons::BACKUP)).clicked() {
                if self.save_config(db).is_ok() {
                    state.navigate(View::Backup);
                }
            }
        });
    }

    fn show_step_controls(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) {
        ui.horizontal(|ui| {
            if self.step != WizardStep::Media {
                if ui.button(format!("{} Tillbaka", Icons::ARROW_LEFT)).clicked() {
                    self.error_message = None;
                    self.step = match self.step {
                        WizardStep::Backup => WizardStep::Media,
                        WizardStep::Done => WizardStep::Backup,
                        WizardStep::Media => WizardStep::Media,
                    };
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                match self.step {
                    WizardStep::Media => {
                        if ui
                            .button(format!("{} Nästa", Icons::ARROW_RIGHT))
                            .clicked()
                        {
                            if self.media_path.trim().is_empty() {
                                self.error_message =
                                    Some("Välj en media-katalog först.".to_string());
                            } else {
                                self.error_message = None;
                                self.step = WizardStep::Backup;
                            }
                        }
                    }
                    WizardStep::Backup => {
                        if ui
                            .button(format!("{} Slutför", Icons::SAVE))
                            .clicked()
                        {
                            if self.backup_path.trim().is_empty() {
                                self.error_message =
                                    Some("Välj en backup-katalog först.".to_string());
                            } else if self.save_config(db).is_ok() {
                                self.error_message = None;
                                self.step = WizardStep::Done;
                                state.show_success("Setup klar");
                            }
                        }
                    }
                    WizardStep::Done => {
                        if ui.button("Gå till Dashboard").clicked() {
                            if self.save_config(db).is_ok() {
                                state.navigate(View::Dashboard);
                            }
                        }
                    }
                }
            });
        });
    }

    fn refresh_from_config(&mut self, db: &Database) {
        if let Ok(config) = db.config().get() {
            self.media_path = config.media_directory_path.display().to_string();
            self.backup_path = config.backup_directory_path.display().to_string();
        }
        self.needs_refresh = false;
    }

    fn save_config(&mut self, db: &Database) -> anyhow::Result<()> {
        let config = SystemConfig {
            id: 1,
            media_directory_path: PathBuf::from(self.media_path.trim()),
            backup_directory_path: PathBuf::from(self.backup_path.trim()),
            created_at: None,
            updated_at: None,
        };

        if let Err(e) = db.config().save(&config) {
            self.error_message = Some(format!("Kunde inte spara: {}", e));
            return Err(e);
        }

        if let Err(e) = config.ensure_directories() {
            self.error_message = Some(format!("Kunde inte skapa kataloger: {}", e));
            return Err(e.into());
        }

        Ok(())
    }

    fn step_title(&self) -> &'static str {
        match self.step {
            WizardStep::Media => "Steg 1: Media-katalog",
            WizardStep::Backup => "Steg 2: Backup-katalog",
            WizardStep::Done => "Steg 3: Klar",
        }
    }
}
