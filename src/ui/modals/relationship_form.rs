//! Modal för att skapa relationer mellan personer

use egui::{self, RichText};

use crate::db::Database;
use crate::models::{Person, PersonRelationship, RelationshipType};
use crate::ui::{
    state::AppState,
    theme::{Colors, Icons},
};

/// Modal för att skapa en relation
pub struct RelationshipFormModal {
    /// Alla personer (för dropdown)
    persons_cache: Vec<Person>,
    /// Vald person att relatera till
    selected_other_person_id: Option<i64>,
    /// Vald relationstyp
    selected_relationship_type: Option<RelationshipType>,
    /// Söktext för personfiltrering
    search_query: String,
    /// Felmeddelande
    error_message: Option<String>,
    /// Behöver refresha
    needs_refresh: bool,
}

impl Default for RelationshipFormModal {
    fn default() -> Self {
        Self::new()
    }
}

impl RelationshipFormModal {
    pub fn new() -> Self {
        Self {
            persons_cache: Vec::new(),
            selected_other_person_id: None,
            selected_relationship_type: None,
            search_query: String::new(),
            error_message: None,
            needs_refresh: true,
        }
    }

    /// Återställ modal
    pub fn reset(&mut self) {
        self.selected_other_person_id = None;
        self.selected_relationship_type = None;
        self.search_query.clear();
        self.error_message = None;
        self.needs_refresh = true;
    }

    /// Visa modalen. Returnerar true om den ska stängas.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        state: &mut AppState,
        db: &Database,
        current_person: &Person,
    ) -> bool {
        let mut should_close = false;

        // Ladda personer om nödvändigt
        if self.needs_refresh {
            self.persons_cache = db.persons().find_all().unwrap_or_default();
            self.needs_refresh = false;
        }

        let current_person_id = current_person.id.unwrap_or(0);

        egui::Window::new(format!("{} Lägg till relation", Icons::PEOPLE))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(400.0);

                ui.label(format!("För: {}", current_person.full_name()));
                ui.add_space(16.0);

                // Välj relationstyp
                ui.horizontal(|ui| {
                    ui.label("Relationstyp:");

                    let type_name = self.selected_relationship_type
                        .map(|t| t.display_name())
                        .unwrap_or("Välj typ...");

                    egui::ComboBox::from_id_salt("rel_type_combo")
                        .selected_text(type_name)
                        .show_ui(ui, |ui| {
                            for rel_type in RelationshipType::all() {
                                let is_selected = self.selected_relationship_type == Some(*rel_type);
                                if ui.selectable_label(is_selected, rel_type.display_name()).clicked() {
                                    self.selected_relationship_type = Some(*rel_type);
                                }
                            }
                        });
                });

                // Visa beskrivning av vald relationstyp
                if let Some(rel_type) = self.selected_relationship_type {
                    ui.add_space(4.0);
                    let description = match rel_type {
                        RelationshipType::Parent => format!("Den valda personen är förälder till {}", current_person.full_name()),
                        RelationshipType::Child => format!("Den valda personen är barn till {}", current_person.full_name()),
                        RelationshipType::Spouse => format!("Den valda personen är gift med {}", current_person.full_name()),
                        RelationshipType::Sibling => format!("Den valda personen är syskon med {}", current_person.full_name()),
                    };
                    ui.label(RichText::new(description).small().color(Colors::TEXT_SECONDARY));
                }

                ui.add_space(12.0);

                // Sök/filtrera personer
                ui.horizontal(|ui| {
                    ui.label(Icons::SEARCH);
                    ui.add(
                        egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text("Sök person...")
                            .desired_width(200.0),
                    );
                });

                ui.add_space(8.0);

                // Lista personer att välja
                ui.label("Välj person:");

                let filtered_persons: Vec<&Person> = self.persons_cache
                    .iter()
                    .filter(|p| {
                        // Exkludera aktuell person
                        p.id != Some(current_person_id)
                    })
                    .filter(|p| {
                        // Filtrera på sökfråga
                        if self.search_query.is_empty() {
                            true
                        } else {
                            let query = self.search_query.to_lowercase();
                            p.full_name().to_lowercase().contains(&query)
                                || p.directory_name.to_lowercase().contains(&query)
                        }
                    })
                    .collect();

                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        if filtered_persons.is_empty() {
                            ui.label(RichText::new("Inga personer hittades").color(Colors::TEXT_MUTED));
                        } else {
                            for person in filtered_persons {
                                let is_selected = self.selected_other_person_id == person.id;

                                let response = ui.selectable_label(is_selected, format!(
                                    "{} {}",
                                    Icons::PERSON,
                                    person.full_name()
                                ));

                                if response.clicked() {
                                    self.selected_other_person_id = person.id;
                                }

                                // Visa år om tillgängligt
                                if is_selected {
                                    let years = person.years_display();
                                    if !years.is_empty() {
                                        ui.horizontal(|ui| {
                                            ui.add_space(24.0);
                                            ui.label(RichText::new(years).small().color(Colors::TEXT_MUTED));
                                        });
                                    }
                                }
                            }
                        }
                    });

                // Felmeddelande
                if let Some(ref error) = self.error_message {
                    ui.add_space(8.0);
                    ui.label(RichText::new(error).color(Colors::ERROR));
                }

                ui.add_space(16.0);

                // Knappar
                ui.horizontal(|ui| {
                    if ui.button("Avbryt").clicked() {
                        self.reset();
                        should_close = true;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let can_save = self.selected_other_person_id.is_some()
                            && self.selected_relationship_type.is_some();

                        ui.add_enabled_ui(can_save, |ui| {
                            if ui.button(format!("{} Spara", Icons::SAVE)).clicked() {
                                match self.save_relationship(db, current_person_id) {
                                    Ok(_) => {
                                        state.show_success("Relation skapad!");
                                        self.reset();
                                        should_close = true;
                                    }
                                    Err(e) => {
                                        self.error_message = Some(e.to_string());
                                    }
                                }
                            }
                        });
                    });
                });
            });

        should_close
    }

    fn save_relationship(&self, db: &Database, current_person_id: i64) -> anyhow::Result<()> {
        let other_person_id = self.selected_other_person_id
            .ok_or_else(|| anyhow::anyhow!("Ingen person vald"))?;

        let relationship_type = self.selected_relationship_type
            .ok_or_else(|| anyhow::anyhow!("Ingen relationstyp vald"))?;

        // Kontrollera att relationen inte redan finns
        if db.relationships().exists(current_person_id, other_person_id)? {
            return Err(anyhow::anyhow!("Relation finns redan mellan dessa personer"));
        }

        // Skapa relationen
        // Den valda personen (other) har relationstyp "relationship_type" till current_person
        let mut relationship = PersonRelationship::new(
            other_person_id,
            current_person_id,
            relationship_type,
        );

        db.relationships().create(&mut relationship)?;

        tracing::info!(
            "Skapade relation: {} -> {} ({})",
            current_person_id,
            other_person_id,
            relationship_type.display_name()
        );

        Ok(())
    }
}
