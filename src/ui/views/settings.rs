use egui::{self, RichText};

use crate::db::Database;
use crate::ui::{
    state::AppState,
    theme::{Colors, Icons},
    View,
};

pub struct SettingsView {
    media_path: String,
    backup_path: String,
    needs_refresh: bool,
    status_message: Option<String>,
}

impl SettingsView {
    pub fn new() -> Self {
        Self {
            media_path: String::new(),
            backup_path: String::new(),
            needs_refresh: true,
            status_message: None,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) {
        if self.needs_refresh {
            self.refresh_config(db);
            self.needs_refresh = false;
        }

        let available_width = ui.available_width();
        let section_width = available_width * 0.8;
        let margin = (available_width - section_width) / 2.0;

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading(format!("{} Inställningar", Icons::SETTINGS));
            ui.add_space(16.0);

            ui.horizontal(|ui| {
                ui.add_space(margin);
                ui.vertical(|ui| {
                    ui.set_width(section_width);

                    // Utseende
                    egui::Frame::none()
                        .fill(ui.visuals().extreme_bg_color)
                        .rounding(8.0)
                        .inner_margin(16.0)
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label(RichText::new("Utseende").strong());
                            ui.add_space(8.0);

                            ui.horizontal(|ui| {
                                ui.label("Mörkt läge:");
                                if ui.checkbox(&mut state.dark_mode, "").changed() {
                                    // Tema ändras i huvudappen
                                }
                            });
                        });

                    ui.add_space(16.0);

                    // Lagringsplatser
                    egui::Frame::none()
                        .fill(ui.visuals().extreme_bg_color)
                        .rounding(8.0)
                        .inner_margin(16.0)
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label(RichText::new("Lagringsplatser").strong());
                            ui.add_space(8.0);

                            ui.label("Media-katalog:");
                            ui.horizontal(|ui| {
                                let w = ui.available_width() - 60.0;
                                ui.add(egui::TextEdit::singleline(&mut self.media_path).desired_width(w));
                                if ui.button("Välj...").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                        self.media_path = path.display().to_string();
                                    }
                                }
                            });

                            ui.add_space(4.0);
                            ui.label("Backup-katalog:");
                            ui.horizontal(|ui| {
                                let w = ui.available_width() - 60.0;
                                ui.add(egui::TextEdit::singleline(&mut self.backup_path).desired_width(w));
                                if ui.button("Välj...").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                        self.backup_path = path.display().to_string();
                                    }
                                }
                            });

                            ui.add_space(8.0);

                            if ui.button("Spara ändringar").clicked() {
                                self.save_config(db);
                            }

                            if let Some(ref msg) = self.status_message {
                                ui.add_space(8.0);
                                ui.label(RichText::new(msg).color(Colors::SUCCESS));
                            }
                        });

                    ui.add_space(16.0);

                    // Dokumentmallar
                    egui::Frame::none()
                        .fill(ui.visuals().extreme_bg_color)
                        .rounding(8.0)
                        .inner_margin(16.0)
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label(RichText::new("Dokumentmallar").strong());
                            ui.add_space(8.0);

                            ui.label("Hantera dokumenttyper med fördefinierade namn och platser.");
                            if ui.button(format!("{} Dokumentmallar", Icons::DOCUMENT)).clicked() {
                                state.navigate(View::DocumentTemplates);
                            }
                        });

                    ui.add_space(16.0);

                    // Checklistmallar
                    egui::Frame::none()
                        .fill(ui.visuals().extreme_bg_color)
                        .rounding(8.0)
                        .inner_margin(16.0)
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label(RichText::new("Checklistor").strong());
                            ui.add_space(8.0);

                            ui.label("Hantera checklistmallar och applicera dem på personer.");
                            if ui.button(format!("{} Mallar", Icons::CHECK)).clicked() {
                                state.navigate(View::ChecklistTemplates);
                            }
                        });

                    ui.add_space(16.0);

                    // Rapporter & Export
                    egui::Frame::none()
                        .fill(ui.visuals().extreme_bg_color)
                        .rounding(8.0)
                        .inner_margin(16.0)
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label(RichText::new("Rapporter & Export").strong());
                            ui.add_space(8.0);

                            ui.label("Exportera data till JSON eller CSV-format.");
                            if ui.button(format!("{} Rapporter", Icons::DOCUMENT)).clicked() {
                                state.navigate(View::Reports);
                            }
                        });

                    ui.add_space(16.0);

                    // Backup & Återställning
                    egui::Frame::none()
                        .fill(ui.visuals().extreme_bg_color)
                        .rounding(8.0)
                        .inner_margin(16.0)
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label(RichText::new("Backup & Återställning").strong());
                            ui.add_space(8.0);

                            ui.label("Skapa och återställ säkerhetskopior av din databas.");
                            if ui.button(format!("{} Öppna Backup & Återställning", Icons::BACKUP)).clicked() {
                                state.navigate(View::Backup);
                            }
                        });

                    ui.add_space(16.0);

                    // Arkivera projekt
                    egui::Frame::none()
                        .fill(ui.visuals().extreme_bg_color)
                        .rounding(8.0)
                        .inner_margin(16.0)
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label(RichText::new("Arkivera projekt").strong());
                            ui.add_space(8.0);

                            ui.label("Arkivera alla data till en ZIP-fil och börja om med ett nytt projekt.");
                            ui.add_space(4.0);
                            if ui.button(format!("{} Arkivera projekt", Icons::FOLDER)).clicked() {
                                state.show_archive_modal = true;
                            }
                        });

                    ui.add_space(16.0);

                    // Om applikationen
                    egui::Frame::none()
                        .fill(ui.visuals().extreme_bg_color)
                        .rounding(8.0)
                        .inner_margin(16.0)
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label(RichText::new("Om Genlib").strong());
                            ui.add_space(8.0);

                            ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));
                            ui.label("Ett dokumenthanteringssystem för släktforskning");
                            ui.add_space(4.0);
                            ui.hyperlink_to("GitHub", "https://github.com/carleson/genlib-desktop");
                        });
                });
            });
        });
    }

    fn refresh_config(&mut self, db: &Database) {
        if let Ok(config) = db.config().get() {
            self.media_path = config.media_directory_path.display().to_string();
            self.backup_path = config.backup_directory_path.display().to_string();
        }
    }

    fn save_config(&mut self, db: &Database) {
        use crate::models::SystemConfig;
        use std::path::PathBuf;

        let config = SystemConfig {
            id: 1,
            media_directory_path: PathBuf::from(&self.media_path),
            backup_directory_path: PathBuf::from(&self.backup_path),
            created_at: None,
            updated_at: None,
        };

        match db.config().save(&config) {
            Ok(_) => {
                self.status_message = Some("Inställningar sparade!".to_string());
                // Skapa kataloger
                let _ = config.ensure_directories();
            }
            Err(e) => {
                self.status_message = Some(format!("Fel: {}", e));
            }
        }
    }
}
