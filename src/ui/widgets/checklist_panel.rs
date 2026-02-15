//! Checklist-panel för att visa och hantera uppgifter

use egui::{self, RichText};

use crate::db::Database;
use crate::models::{ChecklistTemplateItem, PersonChecklistItem};
use crate::ui::{
    state::AppState,
    theme::{Colors, Icons},
};

/// Uppgiftspanel som visas i persondetaljvyn
pub struct ChecklistPanel {
    items: Vec<PersonChecklistItem>,
    progress: (i64, i64),
    needs_refresh: bool,
    person_id: Option<i64>,
    show_add_form: bool,
    /// Fördefinierade uppgifter (från inställningar)
    available_tasks: Vec<ChecklistTemplateItem>,
    /// Redigerar objekt
    editing_item_id: Option<i64>,
    edit_title: String,
}

impl Default for ChecklistPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl ChecklistPanel {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            progress: (0, 0),
            needs_refresh: true,
            person_id: None,
            show_add_form: false,
            available_tasks: Vec::new(),
            editing_item_id: None,
            edit_title: String::new(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database, person_id: i64) {
        if self.needs_refresh || self.person_id != Some(person_id) {
            self.refresh(db, person_id);
        }

        egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .rounding(8.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());

                // Header med progress
                ui.horizontal(|ui| {
                    ui.heading(format!("{} Uppgifter", Icons::CHECK));

                    let (completed, total) = self.progress;
                    if total > 0 {
                        let progress_text = format!("{}/{}", completed, total);
                        let progress_color = if completed == total {
                            Colors::SUCCESS
                        } else {
                            Colors::TEXT_MUTED
                        };
                        ui.label(RichText::new(progress_text).color(progress_color));

                        let progress = completed as f32 / total as f32;
                        let bar_width = 60.0;
                        let bar_height = 6.0;
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(bar_width, bar_height),
                            egui::Sense::hover(),
                        );
                        ui.painter().rect_filled(rect, 3.0, Colors::TEXT_MUTED);
                        let filled_rect = egui::Rect::from_min_size(
                            rect.min,
                            egui::vec2(bar_width * progress, bar_height),
                        );
                        ui.painter().rect_filled(filled_rect, 3.0, Colors::SUCCESS);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if !self.available_tasks.is_empty() {
                            if ui
                                .small_button(format!("{}", Icons::ADD))
                                .on_hover_text("Lägg till uppgift")
                                .clicked()
                            {
                                self.show_add_form = !self.show_add_form;
                            }
                        }
                    });
                });

                ui.add_space(8.0);

                // Visa tillgängliga uppgifter att lägga till
                if self.show_add_form {
                    self.show_add_tasks(ui, state, db, person_id);
                    ui.add_space(8.0);
                }

                // Lista med personens uppgifter
                if self.items.is_empty() {
                    ui.label(
                        RichText::new("Inga uppgifter ännu")
                            .color(Colors::TEXT_MUTED),
                    );
                } else {
                    self.show_items(ui, state, db);
                }
            });
    }

    fn show_add_tasks(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut AppState,
        db: &Database,
        person_id: i64,
    ) {
        egui::Frame::none()
            .fill(ui.visuals().faint_bg_color)
            .rounding(4.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.label(RichText::new("Välj uppgifter att lägga till:").small().strong());
                ui.add_space(4.0);

                // Vilka template_item_ids har personen redan?
                let existing_ids = db
                    .checklists()
                    .template_item_ids_for_person(person_id)
                    .unwrap_or_default();

                let mut any_available = false;
                for task in self.available_tasks.clone() {
                    let already_added = task.id.map_or(false, |id| existing_ids.contains(&id));
                    if already_added {
                        continue;
                    }
                    any_available = true;

                    ui.horizontal(|ui| {
                        if ui
                            .small_button(format!("{} {}", Icons::ADD, &task.title))
                            .clicked()
                        {
                            let mut item =
                                PersonChecklistItem::from_template(person_id, &task);
                            if db.checklists().create(&mut item).is_ok() {
                                state.show_success(&format!("\"{}\" tillagd", task.title));
                                self.needs_refresh = true;
                            }
                        }
                    });
                }

                if !any_available {
                    ui.label(
                        RichText::new("Alla uppgifter är redan tillagda.")
                            .small()
                            .color(Colors::TEXT_MUTED),
                    );
                }
            });
    }

    fn show_items(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) {
        for item in self.items.clone() {
            ui.horizontal(|ui| {
                let mut is_completed = item.is_completed;
                if ui.checkbox(&mut is_completed, "").changed() {
                    if let Some(id) = item.id {
                        if db.checklists().toggle_completed(id).is_ok() {
                            self.needs_refresh = true;
                        }
                    }
                }

                if self.editing_item_id == item.id {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.edit_title)
                            .desired_width(150.0),
                    );

                    if ui.small_button(Icons::SAVE).clicked() {
                        if let Some(id) = item.id {
                            if let Ok(Some(mut updated_item)) = db.checklists().find_by_id(id) {
                                updated_item.title = self.edit_title.clone();
                                if db.checklists().update(&updated_item).is_ok() {
                                    state.show_success("Uppgift uppdaterad");
                                    self.needs_refresh = true;
                                }
                            }
                        }
                        self.editing_item_id = None;
                    }

                    if ui.small_button(Icons::CROSS).clicked() {
                        self.editing_item_id = None;
                    }
                } else {
                    let title_text = if item.is_completed {
                        RichText::new(&item.title)
                            .strikethrough()
                            .color(Colors::TEXT_MUTED)
                    } else {
                        RichText::new(&item.title)
                    };
                    ui.label(title_text);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .small_button(RichText::new(Icons::DELETE).color(Colors::TEXT_MUTED))
                            .on_hover_text("Ta bort")
                            .clicked()
                        {
                            if let Some(id) = item.id {
                                if db.checklists().delete(id).is_ok() {
                                    state.show_success("Uppgift borttagen");
                                    self.needs_refresh = true;
                                }
                            }
                        }

                        if ui
                            .small_button(RichText::new(Icons::EDIT).color(Colors::TEXT_MUTED))
                            .on_hover_text("Redigera")
                            .clicked()
                        {
                            self.editing_item_id = item.id;
                            self.edit_title = item.title.clone();
                        }
                    });
                }
            });
        }
    }

    fn refresh(&mut self, db: &Database, person_id: i64) {
        self.person_id = Some(person_id);
        self.items = db.checklists().find_by_person(person_id).unwrap_or_default();

        self.items.sort_by(|a, b| {
            a.is_completed
                .cmp(&b.is_completed)
                .then_with(|| a.sort_order.cmp(&b.sort_order))
        });

        self.progress = db.checklists().get_progress(person_id).unwrap_or((0, 0));

        // Ladda fördefinierade uppgifter
        self.available_tasks = db.checklists().list_all_template_items().unwrap_or_default();

        self.needs_refresh = false;
    }

    pub fn mark_needs_refresh(&mut self) {
        self.needs_refresh = true;
    }
}
