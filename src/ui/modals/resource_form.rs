use egui::{self, RichText};

use crate::db::Database;
use crate::models::{Resource, ResourceType};
use crate::models::resource::sanitize_directory_name;
use crate::ui::{state::AppState, theme::{Colors, Icons}};

pub struct ResourceFormModal {
    name: String,
    resource_type_id: Option<i64>,
    information: String,
    comment: String,
    lat_str: String,
    lon_str: String,
    directory_name: String,
    /// true = katalognamn redigerat manuellt (låst)
    dir_locked: bool,
    types_cache: Vec<ResourceType>,
    error_message: Option<String>,
    loaded_for_id: Option<i64>,
}

impl ResourceFormModal {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            resource_type_id: None,
            information: String::new(),
            comment: String::new(),
            lat_str: String::new(),
            lon_str: String::new(),
            directory_name: String::new(),
            dir_locked: false,
            types_cache: Vec::new(),
            error_message: None,
            loaded_for_id: None,
        }
    }

    /// Returnerar true när modalen ska stängas (sparad)
    pub fn show(&mut self, ctx: &egui::Context, state: &mut AppState, db: &Database) -> bool {
        // Ladda typer om inte laddade
        if self.types_cache.is_empty() {
            if let Ok(types) = db.resources().get_all_types() {
                self.types_cache = types;
            }
        }

        // Välj standardtyp om ingen vald
        if self.resource_type_id.is_none() {
            if let Some(first) = self.types_cache.first() {
                self.resource_type_id = first.id;
            }
        }

        // Ladda befintlig resurs vid redigering
        let editing_id = state.editing_resource_id;
        if editing_id != self.loaded_for_id {
            self.loaded_for_id = editing_id;
            if let Some(id) = editing_id {
                if let Ok(Some(resource)) = db.resources().find_by_id(id) {
                    self.name = resource.name.clone();
                    self.resource_type_id = Some(resource.resource_type_id);
                    self.information = resource.information.unwrap_or_default();
                    self.comment = resource.comment.unwrap_or_default();
                    self.lat_str = resource.lat.map(|v| v.to_string()).unwrap_or_default();
                    self.lon_str = resource.lon.map(|v| v.to_string()).unwrap_or_default();
                    self.directory_name = resource.directory_name.clone();
                    self.dir_locked = true;
                    self.error_message = None;
                }
            } else {
                self.clear();
            }
        }

        let title = if editing_id.is_some() { "Redigera resurs" } else { "Ny resurs" };
        let mut saved = false;

        egui::Window::new(title)
            .collapsible(false)
            .resizable(true)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .min_width(480.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Namn
                    ui.label(RichText::new("Namn *").strong());
                    let name_resp = ui.add(
                        egui::TextEdit::singleline(&mut self.name)
                            .hint_text("Namn på resursen")
                            .desired_width(f32::INFINITY),
                    );
                    if name_resp.changed() && !self.dir_locked {
                        self.directory_name = sanitize_directory_name(&self.name);
                    }

                    ui.add_space(8.0);

                    // Typ
                    ui.label(RichText::new("Typ *").strong());
                    let selected_type_name = self.resource_type_id
                        .and_then(|id| self.types_cache.iter().find(|t| t.id == Some(id)))
                        .map(|t| t.name.as_str())
                        .unwrap_or("Välj typ");

                    egui::ComboBox::from_id_salt("resource_form_type")
                        .selected_text(selected_type_name)
                        .show_ui(ui, |ui| {
                            for t in &self.types_cache {
                                ui.selectable_value(&mut self.resource_type_id, t.id, &t.name);
                            }
                        });

                    ui.add_space(8.0);

                    // Information
                    ui.label(RichText::new("Information").strong());
                    ui.add(
                        egui::TextEdit::multiline(&mut self.information)
                            .hint_text("Beskriv resursen...")
                            .desired_width(f32::INFINITY)
                            .desired_rows(4),
                    );

                    ui.add_space(8.0);

                    // Kommentar
                    ui.label(RichText::new("Kommentar").strong());
                    ui.add(
                        egui::TextEdit::multiline(&mut self.comment)
                            .hint_text("Interna anteckningar...")
                            .desired_width(f32::INFINITY)
                            .desired_rows(3),
                    );

                    ui.add_space(8.0);

                    // Koordinater
                    ui.label(RichText::new("Koordinater").strong());
                    ui.horizontal(|ui| {
                        ui.label("Lat:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.lat_str)
                                .hint_text("t.ex. 59.3293")
                                .desired_width(120.0),
                        );
                        ui.add_space(8.0);
                        ui.label("Lon:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.lon_str)
                                .hint_text("t.ex. 18.0686")
                                .desired_width(120.0),
                        );
                    });

                    ui.add_space(8.0);

                    // Katalognamn
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Katalognamn").strong());
                        if self.dir_locked {
                            if ui.small_button(Icons::EDIT)
                                .on_hover_text("Lås upp för redigering")
                                .clicked()
                            {
                                self.dir_locked = false;
                            }
                        }
                    });
                    ui.add_enabled(
                        !self.dir_locked,
                        egui::TextEdit::singleline(&mut self.directory_name)
                            .desired_width(f32::INFINITY),
                    );
                    ui.label(
                        RichText::new("Används som mappnamn på disk. Ändra med försiktighet.")
                            .small()
                            .color(Colors::TEXT_MUTED),
                    );

                    // Felmeddelande
                    if let Some(ref msg) = self.error_message {
                        ui.add_space(4.0);
                        ui.label(RichText::new(msg).color(Colors::ERROR));
                    }

                    ui.add_space(16.0);

                    // Knappar
                    ui.horizontal(|ui| {
                        if ui.button("Avbryt").clicked() {
                            state.close_resource_form();
                            self.clear();
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(format!("{} Spara", Icons::SAVE)).clicked() {
                                if let Some(result) = self.save(state, db) {
                                    if result {
                                        saved = true;
                                        self.clear();
                                    }
                                }
                            }
                        });
                    });
                });
            });

        saved
    }

    fn save(&mut self, state: &mut AppState, db: &Database) -> Option<bool> {
        // Validering
        if self.name.trim().is_empty() {
            self.error_message = Some("Namn krävs".to_string());
            return Some(false);
        }
        let Some(type_id) = self.resource_type_id else {
            self.error_message = Some("Välj en typ".to_string());
            return Some(false);
        };

        if self.directory_name.is_empty() {
            self.directory_name = sanitize_directory_name(&self.name);
        }

        let lat = if self.lat_str.is_empty() {
            None
        } else {
            match self.lat_str.parse::<f64>() {
                Ok(v) => Some(v),
                Err(_) => {
                    self.error_message = Some("Ogiltigt latitudvärde".to_string());
                    return Some(false);
                }
            }
        };

        let lon = if self.lon_str.is_empty() {
            None
        } else {
            match self.lon_str.parse::<f64>() {
                Ok(v) => Some(v),
                Err(_) => {
                    self.error_message = Some("Ogiltigt longitudvärde".to_string());
                    return Some(false);
                }
            }
        };

        let mut resource = Resource::new(self.name.trim().to_string(), type_id);
        resource.directory_name = self.directory_name.clone();
        resource.information = if self.information.is_empty() { None } else { Some(self.information.clone()) };
        resource.comment = if self.comment.is_empty() { None } else { Some(self.comment.clone()) };
        resource.lat = lat;
        resource.lon = lon;

        if let Some(editing_id) = state.editing_resource_id {
            resource.id = Some(editing_id);
            match db.resources().update(&resource) {
                Ok(_) => {
                    state.show_success("Resurs uppdaterad");
                    state.close_resource_form();
                    Some(true)
                }
                Err(e) => {
                    self.error_message = Some(format!("Fel: {}", e));
                    Some(false)
                }
            }
        } else {
            match db.resources().create(&resource) {
                Ok(created) => {
                    state.show_success("Resurs skapad");
                    state.navigate_to_resource(created.id.unwrap_or(0));
                    state.close_resource_form();
                    Some(true)
                }
                Err(e) => {
                    self.error_message = Some(format!("Fel: {}", e));
                    Some(false)
                }
            }
        }
    }

    fn clear(&mut self) {
        self.name = String::new();
        self.resource_type_id = None;
        self.information = String::new();
        self.comment = String::new();
        self.lat_str = String::new();
        self.lon_str = String::new();
        self.directory_name = String::new();
        self.dir_locked = false;
        self.error_message = None;
        self.loaded_for_id = None;
        self.types_cache.clear();
    }
}
