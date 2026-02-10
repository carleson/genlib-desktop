//! Checklist-panel för att visa och hantera checklistobjekt

use egui::{self, RichText};

use crate::db::Database;
use crate::models::{ChecklistCategory, ChecklistPriority, PersonChecklistItem};
use crate::ui::{
    state::AppState,
    theme::{Colors, Icons},
};

/// Checklist-panel som visas i persondetaljvyn
pub struct ChecklistPanel {
    /// Cached items
    items: Vec<PersonChecklistItem>,
    /// Progress (completed, total)
    progress: (i64, i64),
    /// Behöver refresh
    needs_refresh: bool,
    /// Person ID
    person_id: Option<i64>,
    /// Visa formulär för nytt objekt
    show_add_form: bool,
    /// Formulärdata för nytt objekt
    new_item_title: String,
    new_item_category: ChecklistCategory,
    new_item_priority: ChecklistPriority,
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
            new_item_title: String::new(),
            new_item_category: ChecklistCategory::default(),
            new_item_priority: ChecklistPriority::default(),
            editing_item_id: None,
            edit_title: String::new(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database, person_id: i64) {
        // Refresh om nödvändigt eller person ändrats
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

                        // Progress bar
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
                        if ui
                            .small_button(format!("{}", Icons::ADD))
                            .on_hover_text("Lägg till objekt")
                            .clicked()
                        {
                            self.show_add_form = true;
                        }
                    });
                });

                ui.add_space(8.0);

                // Formulär för nytt objekt
                if self.show_add_form {
                    self.show_add_item_form(ui, state, db, person_id);
                    ui.add_space(8.0);
                }

                // Lista med checklistobjekt
                if self.items.is_empty() {
                    ui.label(
                        RichText::new("Inga checklistobjekt ännu")
                            .color(Colors::TEXT_MUTED),
                    );
                } else {
                    self.show_items(ui, state, db);
                }
            });
    }

    fn show_add_item_form(
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
                ui.horizontal(|ui| {
                    ui.label("Ny uppgift:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_item_title)
                            .hint_text("Titel...")
                            .desired_width(150.0),
                    );

                    // Kategori
                    egui::ComboBox::from_id_salt("new_category")
                        .selected_text(self.new_item_category.display_name())
                        .width(80.0)
                        .show_ui(ui, |ui| {
                            for cat in ChecklistCategory::all() {
                                ui.selectable_value(
                                    &mut self.new_item_category,
                                    *cat,
                                    cat.display_name(),
                                );
                            }
                        });

                    // Prioritet
                    egui::ComboBox::from_id_salt("new_priority")
                        .selected_text(self.new_item_priority.display_name())
                        .width(70.0)
                        .show_ui(ui, |ui| {
                            for prio in ChecklistPriority::all() {
                                ui.selectable_value(
                                    &mut self.new_item_priority,
                                    *prio,
                                    prio.display_name(),
                                );
                            }
                        });

                    if ui.small_button(Icons::SAVE).clicked() && !self.new_item_title.is_empty() {
                        let mut item = PersonChecklistItem::new(person_id, self.new_item_title.clone());
                        item.category = self.new_item_category;
                        item.priority = self.new_item_priority;

                        if db.checklists().create(&mut item).is_ok() {
                            state.show_success("Uppgift tillagd");
                            self.new_item_title.clear();
                            self.show_add_form = false;
                            self.needs_refresh = true;
                        }
                    }

                    if ui.small_button(Icons::CROSS).clicked() {
                        self.show_add_form = false;
                        self.new_item_title.clear();
                    }
                });
            });
    }

    fn show_items(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) {
        // Gruppera per kategori
        let mut current_category: Option<ChecklistCategory> = None;

        for item in self.items.clone() {
            // Visa kategoriheader
            if current_category != Some(item.category) {
                current_category = Some(item.category);
                ui.add_space(4.0);
                ui.label(
                    RichText::new(item.category.display_name())
                        .small()
                        .strong()
                        .color(Colors::TEXT_SECONDARY),
                );
            }

            ui.horizontal(|ui| {
                // Checkbox
                let mut is_completed = item.is_completed;
                if ui.checkbox(&mut is_completed, "").changed() {
                    if let Some(id) = item.id {
                        if db.checklists().toggle_completed(id).is_ok() {
                            self.needs_refresh = true;
                        }
                    }
                }

                // Prioritetsindikator
                let prio_color = match item.priority {
                    ChecklistPriority::Critical => Colors::ERROR,
                    ChecklistPriority::High => Colors::WARNING,
                    ChecklistPriority::Medium => Colors::INFO,
                    ChecklistPriority::Low => Colors::TEXT_MUTED,
                };
                ui.label(RichText::new("●").small().color(prio_color));

                // Titel (redigera om aktivt)
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
                    // Titel med strikethrough om completed
                    let title_text = if item.is_completed {
                        RichText::new(&item.title)
                            .strikethrough()
                            .color(Colors::TEXT_MUTED)
                    } else {
                        RichText::new(&item.title)
                    };
                    ui.label(title_text);

                    // Knappar (visa vid hover)
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Delete
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

                        // Edit
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

        // Sortera: incomplete först, sedan efter kategori och prioritet
        self.items.sort_by(|a, b| {
            a.is_completed
                .cmp(&b.is_completed)
                .then_with(|| a.category.cmp(&b.category))
                .then_with(|| b.priority.cmp(&a.priority))
        });

        self.progress = db.checklists().get_progress(person_id).unwrap_or((0, 0));
        self.needs_refresh = false;
    }

    pub fn mark_needs_refresh(&mut self) {
        self.needs_refresh = true;
    }
}
