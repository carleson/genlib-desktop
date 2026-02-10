//! Vy för backup och restore

use std::path::PathBuf;

use egui::{self, RichText};

use crate::db::Database;
use crate::services::{BackupInfo, BackupService, RestorePreview, RestoreService};
use crate::ui::{
    state::AppState,
    theme::{Colors, Icons},
};

/// Aktuell operation
#[derive(Debug, Clone, PartialEq)]
enum BackupOperation {
    None,
    Creating,
    Restoring,
    Previewing,
}

/// Backup-vy
pub struct BackupView {
    /// Lista med backuper
    backups: Vec<BackupInfo>,
    /// Vald backup för restore
    selected_backup: Option<PathBuf>,
    /// Förhandsgranskning av restore
    restore_preview: Option<RestorePreview>,
    /// Aktuell operation
    operation: BackupOperation,
    /// Felmeddelande
    error: Option<String>,
    /// Bekräfta restore
    confirm_restore: bool,
    /// Restore-alternativ: återställ databas
    restore_db: bool,
    /// Restore-alternativ: återställ media
    restore_media: bool,
    /// Behöver uppdateras
    needs_refresh: bool,
}

impl Default for BackupView {
    fn default() -> Self {
        Self::new()
    }
}

impl BackupView {
    pub fn new() -> Self {
        Self {
            backups: Vec::new(),
            selected_backup: None,
            restore_preview: None,
            operation: BackupOperation::None,
            error: None,
            confirm_restore: false,
            restore_db: true,
            restore_media: true,
            needs_refresh: true,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) {
        if self.needs_refresh {
            self.refresh_backups(db);
            self.needs_refresh = false;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading(format!("{} Backup & Restore", Icons::FOLDER));
            });

            ui.add_space(16.0);

            // Skapa backup-sektion
            self.show_create_backup_section(ui, state, db);

            ui.add_space(24.0);

            // Befintliga backuper
            self.show_backup_list(ui, state, db);

            // Restore-dialog
            if self.selected_backup.is_some() && !self.confirm_restore {
                self.show_restore_preview(ui, state, db);
            }

            // Bekräfta restore
            if self.confirm_restore {
                self.show_restore_confirm(ui, state, db);
            }
        });
    }

    fn show_create_backup_section(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) {
        ui.heading("Skapa backup");
        ui.add_space(8.0);

        ui.label("Skapa en backup av databasen och alla dokument.");
        ui.add_space(8.0);

        // Felmeddelande
        if let Some(ref error) = self.error {
            ui.label(RichText::new(error).color(Colors::ERROR));
            ui.add_space(8.0);
        }

        match self.operation {
            BackupOperation::Creating => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Skapar backup...");
                });
            }
            _ => {
                if ui
                    .button(RichText::new(format!("{} Skapa backup", Icons::SAVE)).strong())
                    .clicked()
                {
                    self.operation = BackupOperation::Creating;
                    self.error = None;

                    let backup_service = BackupService::new(db);
                    match backup_service.create_backup() {
                        Ok(result) => {
                            state.show_success(&format!(
                                "Backup skapad! {} filer, {}",
                                result.file_count,
                                result.size_display()
                            ));
                            self.needs_refresh = true;
                        }
                        Err(e) => {
                            self.error = Some(format!("Kunde inte skapa backup: {}", e));
                        }
                    }
                    self.operation = BackupOperation::None;
                }
            }
        }
    }

    fn show_backup_list(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) {
        ui.heading("Befintliga backuper");
        ui.add_space(8.0);

        if self.backups.is_empty() {
            ui.label(RichText::new("Inga backuper hittades.").color(Colors::TEXT_MUTED));
            return;
        }

        egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .rounding(4.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for backup in &self.backups.clone() {
                            ui.horizontal(|ui| {
                                ui.label(Icons::DOCUMENT);
                                ui.label(&backup.filename);

                                ui.label(
                                    RichText::new(backup.size_display())
                                        .small()
                                        .color(Colors::TEXT_MUTED),
                                );

                                if let Some(ref date) = backup.date {
                                    ui.label(
                                        RichText::new(date).small().color(Colors::TEXT_SECONDARY),
                                    );
                                }

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        // Radera-knapp
                                        if ui
                                            .small_button(
                                                RichText::new(Icons::DELETE).color(Colors::ERROR),
                                            )
                                            .on_hover_text("Radera backup")
                                            .clicked()
                                        {
                                            let backup_service = BackupService::new(db);
                                            match backup_service.delete_backup(&backup.path) {
                                                Ok(_) => {
                                                    state.show_success("Backup raderad");
                                                    self.needs_refresh = true;
                                                }
                                                Err(e) => {
                                                    self.error =
                                                        Some(format!("Kunde inte radera: {}", e));
                                                }
                                            }
                                        }

                                        // Återställ-knapp
                                        if ui
                                            .small_button("Återställ")
                                            .on_hover_text("Återställ från denna backup")
                                            .clicked()
                                        {
                                            self.selected_backup = Some(backup.path.clone());

                                            // Förhandsgranskning
                                            let restore_service = RestoreService::new(db);
                                            match restore_service.preview(&backup.path) {
                                                Ok(preview) => {
                                                    self.restore_preview = Some(preview);
                                                }
                                                Err(e) => {
                                                    self.error = Some(format!(
                                                        "Kunde inte läsa backup: {}",
                                                        e
                                                    ));
                                                    self.selected_backup = None;
                                                }
                                            }
                                            self.operation = BackupOperation::None;
                                        }
                                    },
                                );
                            });
                            ui.add_space(4.0);
                        }
                    });
            });
    }

    fn show_restore_preview(&mut self, ui: &mut egui::Ui, _state: &mut AppState, _db: &Database) {
        ui.add_space(16.0);
        ui.separator();
        ui.add_space(16.0);

        ui.heading("Återställ backup");

        if let Some(ref preview) = self.restore_preview {
            ui.add_space(8.0);

            egui::Frame::none()
                .fill(ui.visuals().extreme_bg_color)
                .rounding(4.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    egui::Grid::new("restore_preview")
                        .num_columns(2)
                        .spacing([16.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Filer:");
                            ui.label(format!("{}", preview.file_count));
                            ui.end_row();

                            ui.label("Total storlek:");
                            ui.label(preview.size_display());
                            ui.end_row();

                            ui.label("Innehåller databas:");
                            ui.label(if preview.has_database { "Ja" } else { "Nej" });
                            ui.end_row();

                            ui.label("Innehåller media:");
                            ui.label(if preview.has_media { "Ja" } else { "Nej" });
                            ui.end_row();
                        });
                });

            ui.add_space(12.0);

            // Alternativ
            ui.label(RichText::new("Vad ska återställas?").strong());
            ui.add_space(4.0);

            ui.add_enabled_ui(preview.has_database, |ui| {
                ui.checkbox(&mut self.restore_db, "Databas");
            });

            ui.add_enabled_ui(preview.has_media, |ui| {
                ui.checkbox(&mut self.restore_media, "Media/Dokument");
            });

            ui.add_space(12.0);

            ui.horizontal(|ui| {
                if ui.button("Avbryt").clicked() {
                    self.selected_backup = None;
                    self.restore_preview = None;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let can_restore = self.restore_db || self.restore_media;

                    ui.add_enabled_ui(can_restore, |ui| {
                        if ui
                            .button(RichText::new("Återställ").color(Colors::WARNING))
                            .clicked()
                        {
                            self.confirm_restore = true;
                        }
                    });
                });
            });
        }
    }

    fn show_restore_confirm(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) {
        ui.add_space(16.0);

        egui::Frame::none()
            .fill(Colors::WARNING_BG)
            .rounding(4.0)
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(format!("{} Varning!", Icons::DELETE))
                        .strong()
                        .color(Colors::WARNING),
                );
                ui.add_space(8.0);

                ui.label("Detta kommer att ersätta befintlig data med backup-data.");
                ui.label("Denna åtgärd kan inte ångras.");

                if self.restore_db {
                    ui.label(
                        RichText::new("• Databasen kommer att ersättas")
                            .color(Colors::TEXT_SECONDARY),
                    );
                }
                if self.restore_media {
                    ui.label(
                        RichText::new("• Media-filer kan skrivas över")
                            .color(Colors::TEXT_SECONDARY),
                    );
                }

                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    if ui.button("Avbryt").clicked() {
                        self.confirm_restore = false;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(RichText::new("Bekräfta återställning").color(Colors::ERROR))
                            .clicked()
                        {
                            self.do_restore(state, db);
                        }
                    });
                });
            });
    }

    fn do_restore(&mut self, state: &mut AppState, db: &Database) {
        if let Some(ref path) = self.selected_backup.clone() {
            self.operation = BackupOperation::Restoring;

            let restore_service = RestoreService::new(db);
            match restore_service.restore(path, self.restore_db, self.restore_media) {
                Ok(result) => {
                    let mut msg = format!("{} filer återställda", result.files_restored);
                    if result.database_restored {
                        msg.push_str(". Databasen återställd - starta om applikationen för full effekt.");
                    }
                    state.show_success(&msg);
                }
                Err(e) => {
                    self.error = Some(format!("Återställning misslyckades: {}", e));
                }
            }

            self.operation = BackupOperation::None;
            self.confirm_restore = false;
            self.selected_backup = None;
            self.restore_preview = None;
            self.needs_refresh = true;
        }
    }

    fn refresh_backups(&mut self, db: &Database) {
        let backup_service = BackupService::new(db);
        self.backups = backup_service.list_backups().unwrap_or_default();
    }

    pub fn mark_needs_refresh(&mut self) {
        self.needs_refresh = true;
    }
}
