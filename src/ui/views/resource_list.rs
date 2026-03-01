use egui::{self, RichText};

use crate::db::Database;
use crate::models::{Resource, ResourceType};
use crate::ui::{state::AppState, theme::{Colors, Icons}};

pub struct ResourceListView {
    search_query: String,
    type_filter: Option<i64>,
    resources_cache: Vec<(Resource, ResourceType)>,
    types_cache: Vec<ResourceType>,
    needs_refresh: bool,
}

impl ResourceListView {
    pub fn new() -> Self {
        Self {
            search_query: String::new(),
            type_filter: None,
            resources_cache: Vec::new(),
            types_cache: Vec::new(),
            needs_refresh: true,
        }
    }

    pub fn mark_needs_refresh(&mut self) {
        self.needs_refresh = true;
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) {
        if self.needs_refresh {
            self.refresh(db);
            self.needs_refresh = false;
        }

        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading(format!("{} Resurser", Icons::LOCATION));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(format!("{} Ny resurs", Icons::ADD)).clicked() {
                        state.open_new_resource_form();
                    }
                });
            });

            ui.add_space(8.0);

            // Sök + typfilter
            ui.horizontal(|ui| {
                let search_response = ui.add(
                    egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text(format!("{} Sök resurser...", Icons::SEARCH))
                        .desired_width(300.0),
                );
                if search_response.changed() {
                    self.refresh(db);
                }

                ui.add_space(8.0);

                // Typfilter
                let selected_label = if let Some(type_id) = self.type_filter {
                    self.types_cache
                        .iter()
                        .find(|t| t.id == Some(type_id))
                        .map(|t| t.name.as_str())
                        .unwrap_or("Alla")
                        .to_string()
                } else {
                    "Alla typer".to_string()
                };

                egui::ComboBox::from_id_salt("resource_type_filter")
                    .selected_text(&selected_label)
                    .show_ui(ui, |ui| {
                        let changed = ui
                            .selectable_value(&mut self.type_filter, None, "Alla typer")
                            .changed();
                        let mut any_changed = changed;
                        for t in &self.types_cache {
                            let changed = ui
                                .selectable_value(&mut self.type_filter, t.id, &t.name)
                                .changed();
                            any_changed = any_changed || changed;
                        }
                        if any_changed {
                            self.refresh(db);
                        }
                    });
            });

            ui.add_space(8.0);
            ui.separator();

            // Tabell
            if self.resources_cache.is_empty() {
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("Inga resurser").color(Colors::TEXT_MUTED));
                    ui.add_space(8.0);
                    if ui.button(format!("{} Skapa första resursen", Icons::ADD)).clicked() {
                        state.open_new_resource_form();
                    }
                });
            } else {
                // Rubrikrad
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    ui.label(RichText::new("Namn").strong().size(12.0));
                    ui.add_space(200.0);
                    ui.label(RichText::new("Typ").strong().size(12.0));
                    ui.add_space(120.0);
                    ui.label(RichText::new("Ort").strong().size(12.0));
                });
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let resources = self.resources_cache.clone();
                    for (resource, resource_type) in &resources {
                        let row = ui.horizontal(|ui| {
                            ui.add_space(4.0);

                            // Namn (klickbart)
                            let name_label = ui.add(
                                egui::Label::new(
                                    RichText::new(&resource.name).color(Colors::PRIMARY),
                                )
                                .sense(egui::Sense::click()),
                            );
                            if name_label.clicked() {
                                if let Some(id) = resource.id {
                                    state.navigate_to_resource(id);
                                }
                            }
                            name_label.on_hover_cursor(egui::CursorIcon::PointingHand);

                            ui.add_space(8.0);
                            ui.label(
                                RichText::new(&resource_type.name)
                                    .color(Colors::TEXT_SECONDARY)
                                    .size(12.0),
                            );
                        });

                        // Hela raden klickbar
                        let row_resp = row.response.interact(egui::Sense::click());
                        if row_resp.clicked() {
                            if let Some(id) = resource.id {
                                state.navigate_to_resource(id);
                            }
                        }

                        ui.separator();
                    }
                });
            }
        });
    }

    fn refresh(&mut self, db: &Database) {
        if let Ok(types) = db.resources().get_all_types() {
            self.types_cache = types;
        }
        let query = self.search_query.trim().to_string();
        match db.resources().search(&query, self.type_filter) {
            Ok(results) => self.resources_cache = results,
            Err(e) => eprintln!("Fel vid sökning av resurser: {}", e),
        }
    }
}
