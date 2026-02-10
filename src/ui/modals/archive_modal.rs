//! Modal för att arkivera projekt

use egui::{self, RichText};

use crate::db::Database;
use crate::services::BackupService;
use crate::ui::{
    state::AppState,
    theme::{Colors, Icons},
    View,
};

pub struct ArchiveModal {
    confirm_text: String,
    error: Option<String>,
}

impl ArchiveModal {
    pub fn new() -> Self {
        Self {
            confirm_text: String::new(),
            error: None,
        }
    }

    /// Visar arkiveringsmodal. Returnerar true om arkiveringen genomfördes.
    pub fn show(&mut self, ctx: &egui::Context, state: &mut AppState, db: &Database) -> bool {
        if !state.show_archive_modal {
            return false;
        }

        let mut done = false;
        let mut close = false;

        egui::Window::new(format!("{} Arkivera projekt", Icons::FOLDER))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(400.0);

                // Varning
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(Icons::DELETE).size(32.0).color(Colors::WARNING));
                });

                ui.add_space(8.0);

                ui.label("Detta kommer att:");
                ui.add_space(4.0);
                ui.label(RichText::new("  1. Skapa ett arkiv (ZIP) av alla data").color(Colors::TEXT_SECONDARY));
                ui.label(RichText::new("  2. Rensa databasen helt").color(Colors::TEXT_SECONDARY));
                ui.label(RichText::new("  3. Ta bort alla mediefiler från disk").color(Colors::TEXT_SECONDARY));
                ui.label(RichText::new("  4. Visa setup-guiden för nytt projekt").color(Colors::TEXT_SECONDARY));

                ui.add_space(8.0);
                ui.label(
                    RichText::new("Denna åtgärd kan inte ångras!")
                        .strong()
                        .color(Colors::ERROR),
                );

                if let Some(ref error) = self.error {
                    ui.add_space(8.0);
                    ui.label(RichText::new(error).color(Colors::ERROR));
                }

                ui.add_space(12.0);

                // Bekräftelsefält
                ui.label("Skriv OK för att bekräfta:");
                ui.add(egui::TextEdit::singleline(&mut self.confirm_text).desired_width(ui.available_width()));

                ui.add_space(12.0);

                let can_archive = self.confirm_text.trim().eq_ignore_ascii_case("ok");

                ui.horizontal(|ui| {
                    if ui.button("Avbryt").clicked() {
                        close = true;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_enabled_ui(can_archive, |ui| {
                            if ui
                                .button(RichText::new("Arkivera").color(Colors::ERROR))
                                .clicked()
                            {
                                match self.execute_archive(state, db) {
                                    Ok(_) => {
                                        done = true;
                                        close = true;
                                    }
                                    Err(e) => {
                                        self.error = Some(format!("Arkivering misslyckades: {}", e));
                                    }
                                }
                            }
                        });
                    });
                });
            });

        if close {
            self.confirm_text.clear();
            self.error = None;
            state.show_archive_modal = false;
        }

        done
    }

    fn execute_archive(&self, state: &mut AppState, db: &Database) -> anyhow::Result<()> {
        // 1. Hämta config innan vi rensar
        let config = db.config().get()?;
        let media_dir = config.media_directory_path.clone();

        // 2. Skapa arkiv-ZIP
        let backup_service = BackupService::new(db);
        let result = backup_service.create_archive()?;

        // 3. Rensa alla datatabeller
        db.with_connection(|conn| {
            conn.execute_batch(
                "DELETE FROM person_checklist_items;
                 DELETE FROM documents;
                 DELETE FROM person_relationships;
                 DELETE FROM persons;
                 DELETE FROM templates;
                 DELETE FROM system_config;"
            )?;
            Ok(())
        })?;

        // 4. Ta bort media-katalogen från disk
        if media_dir.exists() {
            std::fs::remove_dir_all(&media_dir)?;
        }

        // 5. Navigera till setup-wizard
        state.navigate(View::SetupWizard);
        state.show_success(&format!(
            "Projekt arkiverat! {} filer sparade i {}",
            result.file_count,
            result.path.display()
        ));

        Ok(())
    }
}
