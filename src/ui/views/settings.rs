use egui::{self, RichText};

use crate::db::Database;
use crate::models::config::{
    default_shortcuts, AppSettings, ShortcutAction, ShortcutMap,
};
use crate::ui::{
    shortcuts::capture_shortcut,
    state::AppState,
    theme::{Colors, Icons},
    View,
};

pub struct SettingsView {
    media_path: String,
    backup_path: String,
    needs_refresh: bool,
    status_message: Option<String>,
    // Genvägar
    shortcuts: ShortcutMap,
    shortcuts_loaded: bool,
    capturing_action: Option<ShortcutAction>,
    conflict_warning: Option<String>,
}

impl SettingsView {
    pub fn new() -> Self {
        Self {
            media_path: String::new(),
            backup_path: String::new(),
            needs_refresh: true,
            status_message: None,
            shortcuts: default_shortcuts(),
            shortcuts_loaded: false,
            capturing_action: None,
            conflict_warning: None,
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut AppState,
        db: &Database,
        app_settings: &AppSettings,
    ) {
        if self.needs_refresh {
            self.refresh_config(db);
            self.needs_refresh = false;
        }

        // Ladda genvägar från app_settings första gången
        if !self.shortcuts_loaded {
            self.shortcuts = app_settings.shortcuts.clone();
            self.shortcuts_loaded = true;
        }

        // Hantera tangentfångst
        state.capturing_shortcut = self.capturing_action.is_some();

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

                    // Tangentbordsgenvägar
                    self.show_shortcuts_section(ui, state);

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

                    // Katalognamnformat (read-only)
                    egui::Frame::none()
                        .fill(ui.visuals().extreme_bg_color)
                        .rounding(8.0)
                        .inner_margin(16.0)
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label(RichText::new("Katalognamnformat").strong());
                            ui.add_space(8.0);

                            if let Ok(config) = db.config().get() {
                                ui.label(format!("Format: {}", config.dir_name_format.label()));
                                ui.label(
                                    RichText::new(format!("Exempel: {}", config.dir_name_format.example()))
                                        .small()
                                        .color(Colors::TEXT_MUTED),
                                );
                            }
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new("Ändras vid setup av nytt projekt.")
                                    .small()
                                    .color(Colors::TEXT_MUTED),
                            );
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

    fn show_shortcuts_section(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        // Hantera tangentfångst
        if self.capturing_action.is_some() {
            self.handle_capture(ui.ctx(), state);
        }

        egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .rounding(8.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Tangentbordsgenvägar").strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("Återställ alla").clicked() {
                            self.shortcuts = default_shortcuts();
                            self.conflict_warning = None;
                            state.shortcuts_to_apply = Some(self.shortcuts.clone());
                            state.show_status("Genvägar återställda till standard", crate::ui::StatusType::Success);
                        }
                    });
                });

                ui.add_space(8.0);

                // Konfliktvarning
                if let Some(ref warning) = self.conflict_warning {
                    ui.label(RichText::new(warning).color(Colors::WARNING).small());
                    ui.add_space(4.0);
                }

                egui::Grid::new("shortcuts_grid")
                    .num_columns(4)
                    .spacing([16.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        // Header
                        ui.label(RichText::new("Åtgärd").strong().small());
                        ui.label(RichText::new("Genväg").strong().small());
                        ui.label(""); // Ändra
                        ui.label(""); // Återställ
                        ui.end_row();

                        for &action in ShortcutAction::ALL {
                            ui.label(action.label());

                            let is_capturing = self.capturing_action == Some(action);

                            if is_capturing {
                                ui.label(
                                    RichText::new("Tryck genväg...")
                                        .color(Colors::INFO)
                                        .italics(),
                                );
                                if ui.small_button("Avbryt").clicked() {
                                    self.capturing_action = None;
                                }
                            } else {
                                let display = self
                                    .shortcuts
                                    .get(&action)
                                    .map(|s| s.display())
                                    .unwrap_or_else(|| "—".to_string());
                                ui.label(RichText::new(&display).monospace());
                                if ui.small_button("Ändra").clicked() {
                                    self.capturing_action = Some(action);
                                    self.conflict_warning = None;
                                }
                            }

                            // Återställ-knapp
                            let defaults = default_shortcuts();
                            let is_default = self.shortcuts.get(&action) == defaults.get(&action);
                            if is_default {
                                ui.label("");
                            } else if ui.small_button("Standard").clicked() {
                                if let Some(default_shortcut) = defaults.get(&action) {
                                    self.shortcuts.insert(action, default_shortcut.clone());
                                    state.shortcuts_to_apply = Some(self.shortcuts.clone());
                                }
                            }

                            ui.end_row();
                        }
                    });

                ui.add_space(8.0);
                ui.label(
                    RichText::new(if cfg!(target_os = "macos") {
                        "Tips: Cmd fungerar som Ctrl på macOS"
                    } else {
                        "Tips: Håll Ctrl, Alt eller Shift och tryck en tangent"
                    })
                    .small()
                    .color(Colors::TEXT_MUTED),
                );
            });
    }

    fn handle_capture(&mut self, ctx: &egui::Context, state: &mut AppState) {
        let Some(capturing) = self.capturing_action else {
            return;
        };

        let Some(captured) = capture_shortcut(ctx) else {
            return;
        };

        // Escape utan modifierare avbryter fångst
        if captured.key == egui::Key::Escape
            && !captured.modifiers.ctrl_or_cmd
            && !captured.modifiers.shift
            && !captured.modifiers.alt
        {
            self.capturing_action = None;
            return;
        }

        // Kolla efter konflikter
        let conflict = self
            .shortcuts
            .iter()
            .find(|(a, s)| **a != capturing && **s == captured)
            .map(|(a, _)| *a);

        if let Some(conflicting) = conflict {
            self.conflict_warning = Some(format!(
                "Varning: {} använder redan {}",
                conflicting.label(),
                captured.display()
            ));
        } else {
            self.conflict_warning = None;
        }

        self.shortcuts.insert(capturing, captured);
        self.capturing_action = None;
        // Spara direkt
        state.shortcuts_to_apply = Some(self.shortcuts.clone());
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

        let existing_format = db.config().get().map(|c| c.dir_name_format).unwrap_or_default();

        let config = SystemConfig {
            id: 1,
            media_directory_path: PathBuf::from(&self.media_path),
            backup_directory_path: PathBuf::from(&self.backup_path),
            dir_name_format: existing_format,
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
