use egui::{self, Color32, RichText};

use crate::db::Database;
use crate::ui::{state::AppState, theme::{Colors, Icons}, View};

pub struct DashboardView {
    // Cachad statistik
    person_count: i64,
    document_count: i64,
    image_count: i64,
    total_size: i64,
    tasks_completed: i64,
    tasks_total: i64,
    needs_refresh: bool,
}

impl DashboardView {
    pub fn new() -> Self {
        Self {
            person_count: 0,
            document_count: 0,
            image_count: 0,
            total_size: 0,
            tasks_completed: 0,
            tasks_total: 0,
            needs_refresh: true,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) {
        if self.needs_refresh {
            self.refresh_stats(db);
            self.needs_refresh = false;
        }

        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading(format!("{} Dashboard", Icons::DASHBOARD));
            });

            ui.add_space(16.0);

            // Statistikkort
            ui.horizontal(|ui| {
                self.stat_card(ui, Icons::PEOPLE, "Personer", &self.person_count.to_string(), Colors::PRIMARY);
                ui.add_space(8.0);
                self.stat_card(ui, Icons::DOCUMENT, "Dokument", &self.document_count.to_string(), Colors::SUCCESS);
                ui.add_space(8.0);
                self.stat_card(ui, Icons::IMAGE, "Bilder", &self.image_count.to_string(), Colors::PARENT);
                ui.add_space(8.0);
                self.stat_card(ui, Icons::FOLDER, "Lagring", &format_size(self.total_size), Colors::WARNING);
                ui.add_space(8.0);
                let tasks_remaining = self.tasks_total - self.tasks_completed;
                let tasks_label = format!("{} / {}", tasks_remaining, self.tasks_total);
                self.stat_card(ui, Icons::CHECK, "Uppgifter kvar", &tasks_label, Colors::INFO);
            });

            ui.add_space(24.0);

            // Snabbåtgärder
            ui.heading("Snabbåtgärder");
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button(format!("{} Ny person", Icons::ADD)).clicked() {
                    state.open_new_person_form();
                }

                if ui.button(format!("{} Visa alla personer", Icons::PEOPLE)).clicked() {
                    state.navigate(View::PersonList);
                }

                if ui.button(format!("{} Importera GEDCOM", Icons::DOCUMENT)).clicked() {
                    state.show_gedcom_import = true;
                }

                if ui.button(format!("{} Backup", Icons::FOLDER)).clicked() {
                    state.navigate(View::Backup);
                }

                if ui.button(format!("{} Inställningar", Icons::SETTINGS)).clicked() {
                    state.navigate(View::Settings);
                }
            });

            ui.add_space(24.0);

            // Senaste aktivitet
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading("Senaste personer");
                    ui.add_space(8.0);
                    self.show_recent_persons(ui, state, db);
                });

                ui.add_space(32.0);

                ui.vertical(|ui| {
                    ui.heading("Senaste uppgifter");
                    ui.add_space(8.0);
                    self.show_recent_tasks(ui, state, db);
                });

                ui.add_space(32.0);

                ui.vertical(|ui| {
                    ui.heading("Senaste dokument");
                    ui.add_space(8.0);
                    self.show_recent_documents(ui, state, db);
                });
            });
        });
    }

    fn stat_card(&self, ui: &mut egui::Ui, icon: &str, label: &str, value: &str, color: Color32) {
        egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .rounding(8.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.set_min_width(150.0);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(icon).size(24.0));
                        ui.label(RichText::new(label).color(Colors::TEXT_SECONDARY));
                    });
                    ui.add_space(8.0);
                    ui.label(RichText::new(value).size(28.0).strong().color(color));
                });
            });
    }

    fn show_recent_persons(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) {
        let persons = db.persons().find_all().unwrap_or_default();

        // Visa de 5 senaste
        let recent: Vec<_> = persons.into_iter().take(5).collect();

        if recent.is_empty() {
            ui.label(RichText::new("Inga personer ännu. Skapa din första person!").color(Colors::TEXT_SECONDARY));
            return;
        }

        for person in recent {
            ui.horizontal(|ui| {
                ui.label(Icons::PERSON);
                if ui.link(&person.full_name()).clicked() {
                    if let Some(id) = person.id {
                        state.navigate_to_person(id);
                    }
                }

                let years = person.years_display();
                if !years.is_empty() {
                    ui.label(RichText::new(years).small().color(Colors::TEXT_MUTED));
                }
            });
        }
    }

    fn show_recent_tasks(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) {
        let recent = db.checklists().find_recent(5).unwrap_or_default();

        if recent.is_empty() {
            ui.label(RichText::new("Inga öppna uppgifter.").color(Colors::TEXT_SECONDARY));
            return;
        }

        for (item, person_name) in recent {
            ui.horizontal(|ui| {
                ui.label(Icons::CHECK);
                if ui.link(&item.title).clicked() {
                    state.navigate_to_person(item.person_id);
                }
                if let Some(name) = person_name {
                    ui.label(RichText::new(format!("({})", name.trim())).small().color(Colors::TEXT_MUTED));
                }
            });
        }
    }

    fn show_recent_documents(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) {
        let recent = db.documents().find_recent(5).unwrap_or_default();

        if recent.is_empty() {
            ui.label(RichText::new("Inga dokument ännu.").color(Colors::TEXT_SECONDARY));
            return;
        }

        for (doc, person_name) in recent {
            ui.horizontal(|ui| {
                ui.label(Icons::DOCUMENT);
                if ui.link(&doc.filename).clicked() {
                    state.navigate_to_person(doc.person_id);
                }
                if let Some(name) = person_name {
                    ui.label(RichText::new(format!("({})", name.trim())).small().color(Colors::TEXT_MUTED));
                }
            });
        }
    }

    fn refresh_stats(&mut self, db: &Database) {
        self.person_count = db.persons().count().unwrap_or(0);
        self.document_count = db.documents().count().unwrap_or(0);
        self.image_count = db.documents().count_images().unwrap_or(0);
        self.total_size = db.documents().total_file_size().unwrap_or(0);
        let (completed, total) = db.checklists().get_global_progress().unwrap_or((0, 0));
        self.tasks_completed = completed;
        self.tasks_total = total;
    }

    pub fn mark_needs_refresh(&mut self) {
        self.needs_refresh = true;
    }
}

fn format_size(bytes: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = KB * 1024;
    const GB: i64 = MB * 1024;

    match bytes {
        b if b >= GB => format!("{:.1} GB", b as f64 / GB as f64),
        b if b >= MB => format!("{:.1} MB", b as f64 / MB as f64),
        b if b >= KB => format!("{:.1} KB", b as f64 / KB as f64),
        b => format!("{} B", b),
    }
}
