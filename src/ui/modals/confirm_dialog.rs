use egui::{self, RichText};

use crate::db::Database;
use crate::ui::{
    state::{AppState, ConfirmAction},
    theme::{Colors, Icons},
};

pub struct ConfirmDialog;

impl ConfirmDialog {
    /// Visar bekräftelsedialog och returnerar true om åtgärden bekräftades
    pub fn show(ctx: &egui::Context, state: &mut AppState, db: &Database) -> Option<bool> {
        if !state.show_confirm_dialog {
            return None;
        }

        let mut result = None;

        egui::Window::new("Bekräfta")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(300.0);

                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(Icons::DELETE).size(32.0).color(Colors::WARNING));
                    ui.add_space(8.0);
                    ui.label(&state.confirm_dialog_message);
                });

                ui.add_space(16.0);

                ui.horizontal(|ui| {
                    if ui.button("Avbryt").clicked() {
                        state.close_confirm();
                        result = Some(false);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let confirm_button = ui.button(
                            RichText::new("Bekräfta").color(Colors::ERROR)
                        );

                        if confirm_button.clicked() {
                            if let Some(ref action) = state.confirm_dialog_action.clone() {
                                Self::execute_action(action, state, db);
                            }
                            state.close_confirm();
                            result = Some(true);
                        }
                    });
                });
            });

        result
    }

    fn execute_action(action: &ConfirmAction, state: &mut AppState, db: &Database) {
        match action {
            ConfirmAction::DeletePerson(id) => {
                match db.persons().delete(*id) {
                    Ok(_) => {
                        state.show_success("Person raderad");
                        state.selected_person_id = None;
                        state.navigate(crate::ui::View::PersonList);
                    }
                    Err(e) => {
                        state.show_error(&format!("Kunde inte radera: {}", e));
                    }
                }
            }
            ConfirmAction::DeleteRelationship(id) => {
                match db.relationships().delete(*id) {
                    Ok(_) => {
                        state.show_success("Relation raderad");
                    }
                    Err(e) => {
                        state.show_error(&format!("Kunde inte radera: {}", e));
                    }
                }
            }
            ConfirmAction::DeleteDocument(id) => {
                match db.documents().delete(*id) {
                    Ok(_) => {
                        state.show_success("Dokument raderat");
                    }
                    Err(e) => {
                        state.show_error(&format!("Kunde inte radera: {}", e));
                    }
                }
            }
        }
    }
}
