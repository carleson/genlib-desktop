//! Global checklistavy — sök och hantera checklistobjekt för alla personer

use chrono::NaiveDate;
use egui::{self, RichText};

use crate::db::checklist_repo::{ChecklistSearchFilter, ChecklistSearchResult};
use crate::db::Database;
use crate::models::ChecklistPriority;
use crate::ui::{
    state::AppState,
    theme::{Colors, Icons},
};

pub struct ChecklistSearchView {
    filter: ChecklistSearchFilter,
    birth_after_str: String,
    birth_before_str: String,
    death_after_str: String,
    death_before_str: String,
    results: Vec<ChecklistSearchResult>,
    needs_refresh: bool,
    show_completed: bool,
    show_advanced_filters: bool,
}

impl ChecklistSearchView {
    pub fn new() -> Self {
        Self {
            filter: ChecklistSearchFilter::default(),
            birth_after_str: String::new(),
            birth_before_str: String::new(),
            death_after_str: String::new(),
            death_before_str: String::new(),
            results: Vec::new(),
            needs_refresh: true,
            show_completed: false,
            show_advanced_filters: false,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) {
        if self.needs_refresh {
            self.refresh(db);
        }

        // Header
        ui.horizontal(|ui| {
            ui.heading(format!("{} Uppgifter", Icons::CHECK));
        });

        ui.add_space(8.0);

        // Sökfält och filter
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Sök person:");
            if ui
                .add(
                    egui::TextEdit::singleline(&mut self.filter.query)
                        .hint_text("Namn...")
                        .desired_width(200.0),
                )
                .changed()
            {
                changed = true;
            }

            if ui.button("Rensa").clicked() {
                self.filter.query.clear();
                changed = true;
            }

            ui.separator();

            if ui
                .checkbox(&mut self.show_completed, "Visa avbockade")
                .changed()
            {
                changed = true;
            }

            ui.separator();

            // Levande/Avlidna
            if ui.selectable_label(self.filter.filter_alive.is_none(), "Alla").clicked() {
                self.filter.filter_alive = None;
                changed = true;
            }
            if ui.selectable_label(self.filter.filter_alive == Some(true), "Levande").clicked() {
                self.filter.filter_alive = Some(true);
                changed = true;
            }
            if ui.selectable_label(self.filter.filter_alive == Some(false), "Avlidna").clicked() {
                self.filter.filter_alive = Some(false);
                changed = true;
            }

            ui.separator();

            // Toggle avancerade filter
            let advanced_label = if self.has_date_filters() {
                format!("{} Filter aktiva", Icons::FILTER)
            } else {
                format!("{} Fler filter", Icons::FILTER)
            };
            if ui.selectable_label(self.show_advanced_filters, advanced_label).clicked() {
                self.show_advanced_filters = !self.show_advanced_filters;
            }
        });

        // Avancerade filter (datum)
        if self.show_advanced_filters {
            ui.add_space(4.0);
            egui::Frame::none()
                .fill(ui.visuals().faint_bg_color)
                .rounding(4.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Datumfilter").strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("Återställ").clicked() {
                                self.filter.birth_after = None;
                                self.filter.birth_before = None;
                                self.filter.death_after = None;
                                self.filter.death_before = None;
                                self.birth_after_str.clear();
                                self.birth_before_str.clear();
                                self.death_after_str.clear();
                                self.death_before_str.clear();
                                changed = true;
                            }
                        });
                    });

                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.label("Född:");
                        ui.label(RichText::new("från").small().color(Colors::TEXT_MUTED));
                        if ui.add(
                            egui::TextEdit::singleline(&mut self.birth_after_str)
                                .hint_text("YYYY eller YYYY-MM-DD")
                                .desired_width(120.0),
                        ).changed() {
                            self.filter.birth_after = Self::parse_date(&self.birth_after_str);
                            changed = true;
                        }

                        ui.label(RichText::new("till").small().color(Colors::TEXT_MUTED));
                        if ui.add(
                            egui::TextEdit::singleline(&mut self.birth_before_str)
                                .hint_text("YYYY eller YYYY-MM-DD")
                                .desired_width(120.0),
                        ).changed() {
                            self.filter.birth_before = Self::parse_date(&self.birth_before_str);
                            changed = true;
                        }

                        ui.separator();

                        ui.label("Död:");
                        ui.label(RichText::new("från").small().color(Colors::TEXT_MUTED));
                        if ui.add(
                            egui::TextEdit::singleline(&mut self.death_after_str)
                                .hint_text("YYYY")
                                .desired_width(80.0),
                        ).changed() {
                            self.filter.death_after = Self::parse_date(&self.death_after_str);
                            changed = true;
                        }

                        ui.label(RichText::new("till").small().color(Colors::TEXT_MUTED));
                        if ui.add(
                            egui::TextEdit::singleline(&mut self.death_before_str)
                                .hint_text("YYYY")
                                .desired_width(80.0),
                        ).changed() {
                            self.filter.death_before = Self::parse_date(&self.death_before_str);
                            changed = true;
                        }
                    });
                });
        }

        if changed {
            self.refresh(db);
        }

        ui.add_space(4.0);

        // Statistik
        let total = self.results.len();
        let completed = self.results.iter().filter(|r| r.item.is_completed).count();
        let visible = if self.show_completed {
            total
        } else {
            total - completed
        };
        ui.label(
            RichText::new(format!(
                "{} uppgifter ({} klara, {} visas)",
                total, completed, visible
            ))
            .small()
            .color(Colors::TEXT_MUTED),
        );

        ui.separator();

        // Resultat grupperade per person
        egui::ScrollArea::vertical().show(ui, |ui| {
            if self.results.is_empty() {
                ui.add_space(16.0);
                ui.label(
                    RichText::new("Inga checklistobjekt hittades")
                        .color(Colors::TEXT_MUTED),
                );
                return;
            }

            let mut current_person_id: Option<i64> = None;

            let results_snapshot: Vec<_> = self.results.clone();

            for result in &results_snapshot {
                if !self.show_completed && result.item.is_completed {
                    continue;
                }

                // Personheader vid byte av person
                if current_person_id != Some(result.item.person_id) {
                    current_person_id = Some(result.item.person_id);

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(Icons::PERSON).strong());
                        if ui.link(RichText::new(&result.person_name).strong()).clicked() {
                            state.navigate_to_person(result.item.person_id);
                        }
                    });
                    ui.add_space(2.0);
                }

                // Checklistobjekt
                ui.horizontal(|ui| {
                    ui.add_space(24.0);

                    let mut is_completed = result.item.is_completed;
                    if ui.checkbox(&mut is_completed, "").changed() {
                        if let Some(id) = result.item.id {
                            if db.checklists().toggle_completed(id).is_ok() {
                                self.needs_refresh = true;
                            }
                        }
                    }

                    let prio_color = match result.item.priority {
                        ChecklistPriority::Critical => Colors::ERROR,
                        ChecklistPriority::High => Colors::WARNING,
                        ChecklistPriority::Medium => Colors::INFO,
                        ChecklistPriority::Low => Colors::TEXT_MUTED,
                    };
                    ui.label(RichText::new("●").small().color(prio_color));

                    let title_text = if result.item.is_completed {
                        RichText::new(&result.item.title)
                            .strikethrough()
                            .color(Colors::TEXT_MUTED)
                    } else {
                        RichText::new(&result.item.title)
                    };
                    ui.label(title_text);

                    ui.label(
                        RichText::new(result.item.category.display_name())
                            .small()
                            .color(Colors::TEXT_SECONDARY),
                    );
                });
            }
        });
    }

    fn parse_date(s: &str) -> Option<NaiveDate> {
        if s.is_empty() {
            return None;
        }
        if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            return Some(date);
        }
        if let Ok(year) = s.parse::<i32>() {
            return NaiveDate::from_ymd_opt(year, 1, 1);
        }
        None
    }

    fn has_date_filters(&self) -> bool {
        self.filter.birth_after.is_some()
            || self.filter.birth_before.is_some()
            || self.filter.death_after.is_some()
            || self.filter.death_before.is_some()
    }

    fn refresh(&mut self, db: &Database) {
        self.results = db
            .checklists()
            .search_items_with_person(&self.filter)
            .unwrap_or_default();
        self.needs_refresh = false;
    }

    pub fn mark_needs_refresh(&mut self) {
        self.needs_refresh = true;
    }
}
