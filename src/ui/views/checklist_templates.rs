//! Vy för att definiera uppgifter (enkel lista)

use egui::{self, RichText};

use crate::db::Database;
use crate::models::{ChecklistTemplate, ChecklistTemplateItem};
use crate::ui::{
    state::AppState,
    theme::{Colors, Icons},
};

/// Namn på den enda interna mallen
const DEFAULT_TEMPLATE_NAME: &str = "Standard";

pub struct ChecklistTemplatesView {
    items: Vec<ChecklistTemplateItem>,
    template_id: Option<i64>,
    needs_refresh: bool,

    new_item_title: String,
    editing_item_id: Option<i64>,
    edit_item_title: String,
}

impl Default for ChecklistTemplatesView {
    fn default() -> Self {
        Self::new()
    }
}

impl ChecklistTemplatesView {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            template_id: None,
            needs_refresh: true,
            new_item_title: String::new(),
            editing_item_id: None,
            edit_item_title: String::new(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) {
        if self.needs_refresh {
            self.refresh(db);
        }

        ui.heading(format!("{} Uppgifter", Icons::CHECK));
        ui.add_space(4.0);
        ui.label(
            RichText::new("Definiera uppgifter som kan tilldelas personer.")
                .small()
                .color(Colors::TEXT_MUTED),
        );
        ui.add_space(16.0);

        // Lägg till ny uppgift
        ui.horizontal(|ui| {
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.new_item_title)
                    .hint_text("Lägg till uppgift...")
                    .desired_width(400.0),
            );

            let enter_pressed = response.lost_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter));

            if (ui.button(format!("{} Lägg till", Icons::ADD)).clicked() || enter_pressed)
                && !self.new_item_title.trim().is_empty()
            {
                if let Some(template_id) = self.template_id {
                    let mut item = ChecklistTemplateItem {
                        id: None,
                        template_id,
                        title: self.new_item_title.trim().to_string(),
                        sort_order: 0,
                    };

                    if db.checklists().create_template_item(&mut item).is_ok() {
                        self.new_item_title.clear();
                        self.needs_refresh = true;
                        state.show_success("Uppgift tillagd");
                    }
                }
            }
        });

        ui.add_space(12.0);

        if self.items.is_empty() {
            ui.label(
                RichText::new("Inga uppgifter definierade ännu.")
                    .color(Colors::TEXT_MUTED),
            );
            return;
        }

        // Lista med uppgifter
        for item in self.items.clone() {
            if self.editing_item_id == item.id {
                // Redigeringsläge
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.edit_item_title)
                            .desired_width(400.0),
                    );

                    if ui.small_button(Icons::SAVE).clicked()
                        && !self.edit_item_title.trim().is_empty()
                    {
                        if let (Some(id), Some(template_id)) = (item.id, self.template_id) {
                            let updated = ChecklistTemplateItem {
                                id: Some(id),
                                template_id,
                                title: self.edit_item_title.trim().to_string(),
                                sort_order: item.sort_order,
                            };

                            if db.checklists().update_template_item(&updated).is_ok() {
                                self.editing_item_id = None;
                                self.needs_refresh = true;
                                state.show_success("Uppgift uppdaterad");
                            }
                        }
                    }

                    if ui.small_button(Icons::CROSS).clicked() {
                        self.editing_item_id = None;
                    }
                });
            } else {
                // Visningsläge
                ui.horizontal(|ui| {
                    ui.label(&item.title);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .small_button(RichText::new(Icons::DELETE).color(Colors::ERROR))
                            .on_hover_text("Ta bort")
                            .clicked()
                        {
                            if let Some(id) = item.id {
                                if db.checklists().delete_template_item(id).is_ok() {
                                    self.needs_refresh = true;
                                    state.show_success("Uppgift borttagen");
                                }
                            }
                        }

                        if ui
                            .small_button(RichText::new(Icons::EDIT).color(Colors::TEXT_MUTED))
                            .on_hover_text("Redigera")
                            .clicked()
                        {
                            self.editing_item_id = item.id;
                            self.edit_item_title = item.title.clone();
                        }
                    });
                });
            }
        }
    }

    fn refresh(&mut self, db: &Database) {
        // Hämta eller skapa den enda standardmallen
        self.template_id = self.ensure_default_template(db);

        if let Some(template_id) = self.template_id {
            self.items = db
                .checklists()
                .list_template_items(template_id)
                .unwrap_or_default();
        } else {
            self.items.clear();
        }

        self.needs_refresh = false;
    }

    /// Säkerställ att det finns en standardmall, skapa annars
    fn ensure_default_template(&self, db: &Database) -> Option<i64> {
        let templates = db.checklists().list_templates(true).unwrap_or_default();

        // Använd första mallen som finns, eller skapa en ny
        if let Some(template) = templates.first() {
            return template.id;
        }

        let mut template = ChecklistTemplate::new(DEFAULT_TEMPLATE_NAME.to_string());
        db.checklists().create_template(&mut template).ok()?;
        template.id
    }

    pub fn mark_needs_refresh(&mut self) {
        self.needs_refresh = true;
    }
}
