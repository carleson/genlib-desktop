//! Vy för checklistmallar

use egui::{self, RichText};

use crate::db::Database;
use crate::models::{
    ChecklistCategory, ChecklistPriority, ChecklistTemplate, ChecklistTemplateItem, Person,
};
use crate::services::ChecklistSyncService;
use crate::ui::{
    state::AppState,
    theme::{Colors, Icons},
};

pub struct ChecklistTemplatesView {
    templates: Vec<ChecklistTemplate>,
    template_items: Vec<ChecklistTemplateItem>,
    persons: Vec<Person>,
    selected_template_id: Option<i64>,
    last_selected_template_id: Option<i64>,
    selected_person_id: Option<i64>,
    needs_refresh: bool,
    error_message: Option<String>,
    info_message: Option<String>,

    // Mall-form
    show_new_template_form: bool,
    new_template_name: String,
    new_template_description: String,

    edit_template_name: String,
    edit_template_description: String,
    edit_template_active: bool,

    // Mall-item form
    new_item_title: String,
    new_item_description: String,
    new_item_category: ChecklistCategory,
    new_item_priority: ChecklistPriority,

    editing_item_id: Option<i64>,
    edit_item_title: String,
    edit_item_description: String,
    edit_item_category: ChecklistCategory,
    edit_item_priority: ChecklistPriority,
}

impl Default for ChecklistTemplatesView {
    fn default() -> Self {
        Self::new()
    }
}

impl ChecklistTemplatesView {
    pub fn new() -> Self {
        Self {
            templates: Vec::new(),
            template_items: Vec::new(),
            persons: Vec::new(),
            selected_template_id: None,
            last_selected_template_id: None,
            selected_person_id: None,
            needs_refresh: true,
            error_message: None,
            info_message: None,
            show_new_template_form: false,
            new_template_name: String::new(),
            new_template_description: String::new(),
            edit_template_name: String::new(),
            edit_template_description: String::new(),
            edit_template_active: true,
            new_item_title: String::new(),
            new_item_description: String::new(),
            new_item_category: ChecklistCategory::default(),
            new_item_priority: ChecklistPriority::default(),
            editing_item_id: None,
            edit_item_title: String::new(),
            edit_item_description: String::new(),
            edit_item_category: ChecklistCategory::default(),
            edit_item_priority: ChecklistPriority::default(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) {
        if self.needs_refresh {
            self.refresh(db);
        }

        if self.selected_template_id != self.last_selected_template_id {
            self.load_selected_template_fields();
            self.last_selected_template_id = self.selected_template_id;
        }

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading(format!("{} Checklistmallar", Icons::CHECK));
            });

            ui.add_space(16.0);

            if let Some(ref error) = self.error_message {
                ui.label(RichText::new(error).color(Colors::ERROR));
                ui.add_space(8.0);
            }

            if let Some(ref info) = self.info_message {
                ui.label(RichText::new(info).color(Colors::SUCCESS));
                ui.add_space(8.0);
            }

            ui.horizontal(|ui| {
                // Vänsterkolumn: mallar
                egui::Frame::none()
                    .fill(ui.visuals().extreme_bg_color)
                    .rounding(8.0)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.set_min_width(220.0);
                        ui.label(RichText::new("Mallar").strong());
                        ui.add_space(8.0);

                        if self.templates.is_empty() {
                            ui.label(
                                RichText::new("Inga mallar ännu")
                                    .color(Colors::TEXT_MUTED),
                            );
                        } else {
                            for template in self.templates.clone() {
                                let label = if template.is_active {
                                    template.name.clone()
                                } else {
                                    format!("{} (inaktiv)", template.name)
                                };
                                if ui
                                    .selectable_label(
                                        self.selected_template_id == template.id,
                                        label,
                                    )
                                    .clicked()
                                {
                                    self.selected_template_id = template.id;
                                    self.needs_refresh = true;
                                }
                            }
                        }

                        ui.add_space(8.0);
                        if ui.button(format!("{} Ny mall", Icons::ADD)).clicked() {
                            self.show_new_template_form = true;
                            self.error_message = None;
                            self.info_message = None;
                        }

                        if self.show_new_template_form {
                            ui.add_space(8.0);
                            ui.label("Namn");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.new_template_name)
                                    .desired_width(180.0),
                            );
                            ui.label("Beskrivning");
                            ui.add(
                                egui::TextEdit::multiline(&mut self.new_template_description)
                                    .desired_width(180.0)
                                    .desired_rows(2),
                            );

                            ui.horizontal(|ui| {
                                if ui
                                    .small_button(RichText::new(Icons::SAVE).strong())
                                    .clicked()
                                    && !self.new_template_name.trim().is_empty()
                                {
                                    let mut template =
                                        ChecklistTemplate::new(self.new_template_name.trim().to_string());
                                    if !self.new_template_description.trim().is_empty() {
                                        template.description =
                                            Some(self.new_template_description.trim().to_string());
                                    }

                                    match db.checklists().create_template(&mut template) {
                                        Ok(id) => {
                                            self.selected_template_id = Some(id);
                                            self.needs_refresh = true;
                                            self.show_new_template_form = false;
                                            self.new_template_name.clear();
                                            self.new_template_description.clear();
                                            state.show_success("Mall skapad");
                                        }
                                        Err(e) => {
                                            self.error_message =
                                                Some(format!("Kunde inte skapa mall: {}", e));
                                        }
                                    }
                                }

                                if ui.small_button(Icons::CROSS).clicked() {
                                    self.show_new_template_form = false;
                                    self.new_template_name.clear();
                                    self.new_template_description.clear();
                                }
                            });
                        }
                    });

                ui.add_space(12.0);

                // Högerkolumn: detaljer
                egui::Frame::none()
                    .fill(ui.visuals().extreme_bg_color)
                    .rounding(8.0)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.set_min_width(420.0);
                        if let Some(template_id) = self.selected_template_id {
                            self.show_template_details(ui, state, db, template_id);
                        } else {
                            ui.label(
                                RichText::new("Välj en mall för att redigera")
                                    .color(Colors::TEXT_MUTED),
                            );
                        }
                    });
            });
        });
    }

    fn show_template_details(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut AppState,
        db: &Database,
        template_id: i64,
    ) {
        ui.label(RichText::new("Mallinformation").strong());
        ui.add_space(8.0);

        ui.label("Namn");
        ui.add(
            egui::TextEdit::singleline(&mut self.edit_template_name)
                .desired_width(260.0),
        );
        ui.label("Beskrivning");
        ui.add(
            egui::TextEdit::multiline(&mut self.edit_template_description)
                .desired_width(260.0)
                .desired_rows(2),
        );
        ui.checkbox(&mut self.edit_template_active, "Aktiv");

        ui.horizontal(|ui| {
            if ui
                .small_button(RichText::new(format!("{} Spara", Icons::SAVE)).strong())
                .clicked()
                && !self.edit_template_name.trim().is_empty()
            {
                let template = ChecklistTemplate {
                    id: Some(template_id),
                    name: self.edit_template_name.trim().to_string(),
                    description: if self.edit_template_description.trim().is_empty() {
                        None
                    } else {
                        Some(self.edit_template_description.trim().to_string())
                    },
                    is_active: self.edit_template_active,
                };

                if let Ok(_) = db.checklists().update_template(&template) {
                    self.info_message = Some("Mall uppdaterad".to_string());
                    self.error_message = None;
                    self.needs_refresh = true;
                    state.show_success("Mall uppdaterad");
                    self.load_selected_template_fields();
                }
            }

            if ui
                .small_button(RichText::new(format!("{} Radera", Icons::DELETE)).color(Colors::ERROR))
                .clicked()
            {
                if let Ok(_) = db.checklists().delete_template(template_id) {
                    self.selected_template_id = None;
                    self.needs_refresh = true;
                    state.show_success("Mall raderad");
                }
            }
        });

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);

        self.show_apply_section(ui, state, db, template_id);

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);

        self.show_template_items(ui, state, db, template_id);
    }

    fn show_apply_section(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut AppState,
        db: &Database,
        template_id: i64,
    ) {
        ui.label(RichText::new("Applicera mall").strong());
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label("Person:");
            egui::ComboBox::from_id_salt("template_apply_person")
                .selected_text(match self.selected_person_id {
                    None => "Alla personer".to_string(),
                    Some(id) => self
                        .persons
                        .iter()
                        .find(|p| p.id == Some(id))
                        .map(|p| p.full_name())
                        .unwrap_or_else(|| "Okänd".to_string()),
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.selected_person_id, None, "Alla personer");
                    for person in &self.persons {
                        if let Some(id) = person.id {
                            ui.selectable_value(
                                &mut self.selected_person_id,
                                Some(id),
                                person.full_name(),
                            );
                        }
                    }
                });

            if ui
                .button(format!("{} Applicera", Icons::EXPORT))
                .clicked()
            {
                let service = ChecklistSyncService::new(db);
                let result = match self.selected_person_id {
                    Some(person_id) => service.apply_template_to_person(template_id, person_id),
                    None => service.apply_template_to_all(template_id),
                };

                match result {
                    Ok(result) => {
                        let target = if self.selected_person_id.is_some() {
                            "person"
                        } else {
                            "personer"
                        };
                        state.show_success(&format!(
                            "Mall applicerad till {}: {} nya, {} redan fanns",
                            target, result.created, result.skipped
                        ));
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Kunde inte applicera mall: {}", e));
                    }
                }
            }
        });
    }

    fn show_template_items(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut AppState,
        db: &Database,
        template_id: i64,
    ) {
        ui.label(RichText::new("Mall-objekt").strong());
        ui.add_space(6.0);

        egui::Frame::none()
            .fill(ui.visuals().faint_bg_color)
            .rounding(6.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Ny uppgift:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_item_title)
                            .hint_text("Titel...")
                            .desired_width(160.0),
                    );

                    egui::ComboBox::from_id_salt("template_item_category")
                        .selected_text(self.new_item_category.display_name())
                        .show_ui(ui, |ui| {
                            for cat in ChecklistCategory::all() {
                                ui.selectable_value(
                                    &mut self.new_item_category,
                                    *cat,
                                    cat.display_name(),
                                );
                            }
                        });

                    egui::ComboBox::from_id_salt("template_item_priority")
                        .selected_text(self.new_item_priority.display_name())
                        .show_ui(ui, |ui| {
                            for prio in ChecklistPriority::all() {
                                ui.selectable_value(
                                    &mut self.new_item_priority,
                                    *prio,
                                    prio.display_name(),
                                );
                            }
                        });

                    if ui.small_button(Icons::SAVE).clicked()
                        && !self.new_item_title.trim().is_empty()
                    {
                        let mut item = ChecklistTemplateItem {
                            id: None,
                            template_id,
                            title: self.new_item_title.trim().to_string(),
                            description: if self.new_item_description.trim().is_empty() {
                                None
                            } else {
                                Some(self.new_item_description.trim().to_string())
                            },
                            category: self.new_item_category,
                            priority: self.new_item_priority,
                            sort_order: 0,
                        };

                        if db.checklists().create_template_item(&mut item).is_ok() {
                            self.new_item_title.clear();
                            self.new_item_description.clear();
                            self.needs_refresh = true;
                            state.show_success("Mall-objekt tillagt");
                        }
                    }
                });

                ui.add_space(4.0);
                ui.label("Beskrivning");
                ui.add(
                    egui::TextEdit::multiline(&mut self.new_item_description)
                        .desired_width(300.0)
                        .desired_rows(2),
                );
            });

        ui.add_space(8.0);

        if self.template_items.is_empty() {
            ui.label(
                RichText::new("Inga mall-objekt ännu")
                    .color(Colors::TEXT_MUTED),
            );
            return;
        }

        for item in self.template_items.clone() {
            if self.editing_item_id == item.id {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.edit_item_title)
                                .desired_width(160.0),
                        );

                        egui::ComboBox::from_id_salt("edit_item_category")
                            .selected_text(self.edit_item_category.display_name())
                            .show_ui(ui, |ui| {
                                for cat in ChecklistCategory::all() {
                                    ui.selectable_value(
                                        &mut self.edit_item_category,
                                        *cat,
                                        cat.display_name(),
                                    );
                                }
                            });

                        egui::ComboBox::from_id_salt("edit_item_priority")
                            .selected_text(self.edit_item_priority.display_name())
                            .show_ui(ui, |ui| {
                                for prio in ChecklistPriority::all() {
                                    ui.selectable_value(
                                        &mut self.edit_item_priority,
                                        *prio,
                                        prio.display_name(),
                                    );
                                }
                            });

                        if ui.small_button(Icons::SAVE).clicked() {
                            if let Some(id) = item.id {
                                let updated = ChecklistTemplateItem {
                                    id: Some(id),
                                    template_id,
                                    title: self.edit_item_title.trim().to_string(),
                                    description: if self.edit_item_description.trim().is_empty() {
                                        None
                                    } else {
                                        Some(self.edit_item_description.trim().to_string())
                                    },
                                    category: self.edit_item_category,
                                    priority: self.edit_item_priority,
                                    sort_order: item.sort_order,
                                };

                                if db.checklists().update_template_item(&updated).is_ok() {
                                    self.editing_item_id = None;
                                    self.needs_refresh = true;
                                    state.show_success("Mall-objekt uppdaterat");
                                }
                            }
                        }

                        if ui.small_button(Icons::CROSS).clicked() {
                            self.editing_item_id = None;
                        }
                    });

                    ui.add(
                        egui::TextEdit::singleline(&mut self.edit_item_description)
                            .hint_text("Beskrivning")
                            .desired_width(300.0),
                    );
                });
            } else {
                ui.horizontal(|ui| {
                    ui.label(&item.title);
                    ui.label(
                        RichText::new(item.category.display_name())
                            .small()
                            .color(Colors::TEXT_MUTED),
                    );
                    ui.label(
                        RichText::new(item.priority.display_name())
                            .small()
                            .color(Colors::TEXT_MUTED),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .small_button(RichText::new(Icons::DELETE).color(Colors::ERROR))
                            .clicked()
                        {
                            if let Some(id) = item.id {
                                if db.checklists().delete_template_item(id).is_ok() {
                                    self.needs_refresh = true;
                                    state.show_success("Mall-objekt raderat");
                                }
                            }
                        }

                        if ui
                            .small_button(RichText::new(Icons::EDIT).color(Colors::TEXT_MUTED))
                            .clicked()
                        {
                            self.editing_item_id = item.id;
                            self.edit_item_title = item.title.clone();
                            self.edit_item_description =
                                item.description.clone().unwrap_or_default();
                            self.edit_item_category = item.category;
                            self.edit_item_priority = item.priority;
                        }
                    });
                });
            }
        }
    }

    fn refresh(&mut self, db: &Database) {
        self.templates = db.checklists().list_templates(true).unwrap_or_default();
        self.persons = db.persons().find_all().unwrap_or_default();

        if let Some(template_id) = self.selected_template_id {
            self.template_items = db
                .checklists()
                .list_template_items(template_id)
                .unwrap_or_default();
        } else {
            self.template_items.clear();
        }

        self.needs_refresh = false;
    }

    fn load_selected_template_fields(&mut self) {
        if let Some(template_id) = self.selected_template_id {
            if let Some(template) = self.templates.iter().find(|t| t.id == Some(template_id)) {
                self.edit_template_name = template.name.clone();
                self.edit_template_description = template.description.clone().unwrap_or_default();
                self.edit_template_active = template.is_active;
                self.editing_item_id = None;
            }
        } else {
            self.edit_template_name.clear();
            self.edit_template_description.clear();
            self.edit_template_active = true;
            self.editing_item_id = None;
        }
    }

    pub fn mark_needs_refresh(&mut self) {
        self.needs_refresh = true;
    }
}
