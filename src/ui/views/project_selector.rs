//! Projektväljar-vy — helskärm för att välja, skapa och hantera projekt

use eframe::egui;

use crate::projects::{Project, ProjectAction, ProjectRegistry};
use crate::utils::path::display_path;

/// Formulärdata för nytt projekt
#[derive(Default)]
struct NewProjectForm {
    name: String,
    description: String,
    directory: String,
    error: Option<String>,
}

impl NewProjectForm {
    fn clear(&mut self) {
        *self = Self::default();
    }

    fn update_directory_suggestion(&mut self) {
        if !self.name.is_empty() {
            let suggested = ProjectRegistry::suggested_dir(&self.name);
            self.directory = suggested.display().to_string();
        }
    }
}

/// Helskärmsvy för val och hantering av projekt
pub struct ProjectSelectorView {
    show_new_form: bool,
    new_form: NewProjectForm,
    renaming_id: Option<String>,
    rename_buffer: String,
    confirm_delete_id: Option<String>,
}

impl ProjectSelectorView {
    pub fn new() -> Self {
        Self {
            show_new_form: false,
            new_form: NewProjectForm::default(),
            renaming_id: None,
            rename_buffer: String::new(),
            confirm_delete_id: None,
        }
    }

    /// Återställ vy-tillståndet (anropas när vyn visas från topbaren)
    pub fn reset(&mut self) {
        self.show_new_form = false;
        self.new_form.clear();
        self.renaming_id = None;
        self.rename_buffer.clear();
        self.confirm_delete_id = None;
    }

    /// Visa projektväljar-skärmen. Returnerar åtgärd om användaren interagerar.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        registry: &ProjectRegistry,
    ) -> Option<ProjectAction> {
        // Om inga projekt finns, visa direkt formuläret för nytt projekt
        if registry.projects.is_empty() && !self.show_new_form {
            self.show_new_form = true;
            self.new_form.clear();
        }

        let mut action = None;

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(40.0);

            // Rubrik
            ui.vertical_centered(|ui| {
                ui.heading(egui::RichText::new("Genlib").size(32.0).strong());
                ui.label(
                    egui::RichText::new("Välj genealogiprojekt")
                        .size(18.0)
                        .weak(),
                );
            });

            ui.add_space(30.0);
            ui.separator();
            ui.add_space(20.0);

            // Projektlista
            if !registry.projects.is_empty() {
                let available_width = ui.available_width();
                let card_width = (available_width - 40.0).min(700.0);

                for project in &registry.projects {
                    ui.vertical_centered(|ui| {
                        ui.set_max_width(card_width);
                        if let Some(a) = self.show_project_card(ui, project, registry) {
                            action = Some(a);
                        }
                    });
                    ui.add_space(8.0);
                }

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(20.0);
            }

            // Ny-projekt-knapp eller formulär
            ui.vertical_centered(|ui| {
                let available_width = ui.available_width();
                let form_width = (available_width - 40.0).min(700.0);
                ui.set_max_width(form_width);

                if !self.show_new_form {
                    if ui
                        .button(egui::RichText::new("➕ Nytt projekt").size(15.0))
                        .clicked()
                    {
                        self.show_new_form = true;
                        self.new_form.clear();
                    }
                } else {
                    if let Some(a) = self.show_new_form(ui, registry.projects.is_empty()) {
                        action = Some(a);
                    }
                }
            });

            ui.add_space(40.0);
        });

        action
    }

    fn show_project_card(
        &mut self,
        ui: &mut egui::Ui,
        project: &Project,
        registry: &ProjectRegistry,
    ) -> Option<ProjectAction> {
        let mut action = None;

        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                // Vänster: info
                ui.vertical(|ui| {
                    ui.set_min_width(200.0);

                    // Rubrik / rename-läge
                    if self.renaming_id.as_deref() == Some(&project.id) {
                        ui.horizontal(|ui| {
                            let resp = ui.text_edit_singleline(&mut self.rename_buffer);
                            if resp.lost_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                            {
                                let new_name = self.rename_buffer.trim().to_string();
                                if !new_name.is_empty() {
                                    action = Some(ProjectAction::Rename(
                                        project.id.clone(),
                                        new_name,
                                    ));
                                }
                                self.renaming_id = None;
                            }
                            if ui.button("✔").clicked() {
                                let new_name = self.rename_buffer.trim().to_string();
                                if !new_name.is_empty() {
                                    action = Some(ProjectAction::Rename(
                                        project.id.clone(),
                                        new_name,
                                    ));
                                }
                                self.renaming_id = None;
                            }
                            if ui.button("✖").clicked() {
                                self.renaming_id = None;
                            }
                        });
                    } else {
                        let name_text = if project.is_default {
                            format!("⭐ {}", project.name)
                        } else {
                            project.name.clone()
                        };
                        ui.strong(name_text);
                    }

                    if !project.description.is_empty() {
                        ui.label(
                            egui::RichText::new(&project.description).weak().small(),
                        );
                    }
                    ui.label(
                        egui::RichText::new(display_path(&project.directory))
                            .weak()
                            .small()
                            .monospace(),
                    );
                    ui.label(
                        egui::RichText::new(format!("Skapad: {}", &project.created_at))
                            .weak()
                            .small(),
                    );
                });

                // Höger: knappar
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Radera
                    if self.confirm_delete_id.as_deref() == Some(&project.id) {
                        ui.colored_label(
                            egui::Color32::from_rgb(220, 80, 80),
                            "Bekräfta radering:",
                        );
                        if ui
                            .button(egui::RichText::new("Ja, ta bort").color(
                                egui::Color32::from_rgb(220, 80, 80),
                            ))
                            .clicked()
                        {
                            action = Some(ProjectAction::Delete(project.id.clone()));
                            self.confirm_delete_id = None;
                        }
                        if ui.button("Avbryt").clicked() {
                            self.confirm_delete_id = None;
                        }
                    } else {
                        // Öppna
                        if ui
                            .button(egui::RichText::new("📂 Öppna").strong())
                            .clicked()
                        {
                            action = Some(ProjectAction::Open(project.id.clone()));
                        }

                        // Byt namn
                        if ui.button("✏").on_hover_text("Byt namn").clicked() {
                            self.renaming_id = Some(project.id.clone());
                            self.rename_buffer = project.name.clone();
                        }

                        // Ta bort (ej tillåtet om bara ett projekt)
                        if registry.projects.len() > 1 {
                            if ui
                                .button("🗑")
                                .on_hover_text("Ta bort från listan (filer lämnas kvar)")
                                .clicked()
                            {
                                self.confirm_delete_id = Some(project.id.clone());
                            }
                        }

                        // Standard
                        if !project.is_default {
                            if ui
                                .button("⭐")
                                .on_hover_text("Sätt som standardprojekt")
                                .clicked()
                            {
                                action = Some(ProjectAction::SetDefault(project.id.clone()));
                            }
                        }
                    }
                });
            });
        });

        action
    }

    fn show_new_form(
        &mut self,
        ui: &mut egui::Ui,
        is_first_project: bool,
    ) -> Option<ProjectAction> {
        let mut action = None;

        let title = if is_first_project {
            "Skapa ditt första projekt"
        } else {
            "Nytt projekt"
        };

        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.strong(title);
            ui.add_space(8.0);

            ui.label("Namn:");
            let name_resp = ui.text_edit_singleline(&mut self.new_form.name);
            if name_resp.changed() && self.new_form.directory.is_empty() {
                self.new_form.update_directory_suggestion();
            }
            if name_resp.changed() {
                // Uppdatera förslag när namnet ändras och katalog inte är manuellt angiven
                if !self.new_form.name.is_empty() {
                    let suggested = ProjectRegistry::suggested_dir(&self.new_form.name)
                        .display()
                        .to_string();
                    self.new_form.directory = suggested;
                }
            }

            ui.add_space(4.0);
            ui.label("Beskrivning (valfri):");
            ui.text_edit_singleline(&mut self.new_form.description);

            ui.add_space(4.0);
            ui.label("Projektmapp:");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.new_form.directory);
                if ui.button("…").on_hover_text("Välj mapp").clicked() {
                    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                        let name_safe = if self.new_form.name.is_empty() {
                            "nytt_projekt".to_string()
                        } else {
                            self.new_form.name.to_lowercase().replace(' ', "_")
                        };
                        self.new_form.directory =
                            dir.join(&name_safe).display().to_string();
                    }
                }
            });

            if let Some(ref err) = self.new_form.error {
                ui.colored_label(egui::Color32::from_rgb(220, 80, 80), err);
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let can_create = !self.new_form.name.trim().is_empty()
                    && !self.new_form.directory.trim().is_empty();

                ui.add_enabled_ui(can_create, |ui| {
                    if ui
                        .button(egui::RichText::new("✔ Skapa projekt").strong())
                        .clicked()
                    {
                        let name = self.new_form.name.trim().to_string();
                        let description = self.new_form.description.trim().to_string();
                        let directory =
                            std::path::PathBuf::from(self.new_form.directory.trim());

                        action = Some(ProjectAction::CreateNew {
                            name,
                            description,
                            directory,
                        });
                        self.show_new_form = false;
                        self.new_form.clear();
                    }
                });

                if !is_first_project && ui.button("Avbryt").clicked() {
                    self.show_new_form = false;
                    self.new_form.clear();
                }
            });
        });

        action
    }
}
