use chrono::NaiveDate;
use egui::{self, RichText};

use crate::db::Database;
use crate::models::Person;
use crate::ui::{
    state::{AppState, PersonFormData},
    theme::{Colors, Icons},
};

pub struct PersonFormModal {
    form_data: PersonFormData,
    error_message: Option<String>,
    auto_generate_dir: bool,  // Auto-generera katalognamn
}

impl PersonFormModal {
    pub fn new() -> Self {
        Self {
            form_data: PersonFormData::default(),
            error_message: None,
            auto_generate_dir: true,
        }
    }

    /// Visar modalen och returnerar true om den ska stängas
    pub fn show(&mut self, ctx: &egui::Context, state: &mut AppState, db: &Database) -> bool {
        let mut should_close = false;

        // Ladda befintlig person om vi redigerar
        if state.show_person_form && state.editing_person_id.is_some() {
            let person_id = state.editing_person_id.unwrap();
            if self.form_data.directory_name.is_empty() {
                if let Ok(Some(person)) = db.persons().find_by_id(person_id) {
                    self.form_data = PersonFormData::from_person(&person);
                    self.auto_generate_dir = false;  // Redigerar befintlig, auto-generera inte
                }
            }
        }

        let title = if state.editing_person_id.is_some() {
            "Redigera person"
        } else {
            "Ny person"
        };

        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(400.0);

                // Formulär
                let mut name_changed = false;

                egui::Grid::new("person_form_grid")
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Förnamn:");
                        let firstname_response = ui.text_edit_singleline(&mut self.form_data.firstname);
                        if firstname_response.changed() {
                            name_changed = true;
                        }
                        ui.end_row();

                        ui.label("Efternamn:");
                        let surname_response = ui.text_edit_singleline(&mut self.form_data.surname);
                        if surname_response.changed() {
                            name_changed = true;
                        }
                        ui.end_row();

                        ui.label("Födelsedatum:");
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut self.form_data.birth_date)
                                .desired_width(100.0));
                            ui.label(RichText::new("YYYY-MM-DD").small().color(Colors::TEXT_MUTED));
                        });
                        ui.end_row();

                        ui.label("Dödsdatum:");
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut self.form_data.death_date)
                                .desired_width(100.0));
                            ui.label(RichText::new("YYYY-MM-DD").small().color(Colors::TEXT_MUTED));
                        });
                        ui.end_row();

                        ui.label("Katalognamn:");
                        ui.horizontal(|ui| {
                            let dir_response = ui.text_edit_singleline(&mut self.form_data.directory_name);
                            if dir_response.changed() {
                                // Användaren redigerade manuellt, sluta auto-generera
                                self.auto_generate_dir = false;
                            }
                            if ui.small_button("🔄").on_hover_text("Auto-generera från namn").clicked() {
                                let fmt = db.config().get().map(|c| c.dir_name_format).unwrap_or_default();
                                self.form_data.directory_name = Person::generate_directory_name(
                                    &Some(self.form_data.firstname.clone()).filter(|s| !s.is_empty()),
                                    &Some(self.form_data.surname.clone()).filter(|s| !s.is_empty()),
                                    &Some(self.form_data.birth_date.clone()).filter(|s| !s.is_empty()),
                                    fmt,
                                );
                                self.auto_generate_dir = true;
                            }
                        });
                        ui.end_row();

                    });

                // Auto-generera katalognamn om aktiverat och namn ändrats
                if name_changed && self.auto_generate_dir && state.editing_person_id.is_none() {
                    let fmt = db.config().get().map(|c| c.dir_name_format).unwrap_or_default();
                    self.form_data.directory_name = Person::generate_directory_name(
                        &Some(self.form_data.firstname.clone()).filter(|s| !s.is_empty()),
                        &Some(self.form_data.surname.clone()).filter(|s| !s.is_empty()),
                        &Some(self.form_data.birth_date.clone()).filter(|s| !s.is_empty()),
                        fmt,
                    );
                }

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
                        if ui.button(format!("{} Spara", Icons::SAVE)).clicked() {
                            match self.save(state, db) {
                                Ok(_) => {
                                    self.reset();
                                    should_close = true;
                                    state.show_success("Person sparad!");
                                }
                                Err(e) => {
                                    self.error_message = Some(e.to_string());
                                }
                            }
                        }
                    });
                });
            });

        should_close
    }

    fn save(&mut self, state: &AppState, db: &Database) -> anyhow::Result<()> {
        // Validera
        if self.form_data.firstname.is_empty() && self.form_data.surname.is_empty() {
            return Err(anyhow::anyhow!("Minst förnamn eller efternamn krävs"));
        }

        if self.form_data.directory_name.is_empty() {
            return Err(anyhow::anyhow!("Katalognamn krävs"));
        }

        // Parse datum
        let birth_date = if self.form_data.birth_date.is_empty() {
            None
        } else {
            Some(NaiveDate::parse_from_str(&self.form_data.birth_date, "%Y-%m-%d")
                .map_err(|_| anyhow::anyhow!("Ogiltigt födelsedatum (använd YYYY-MM-DD)"))?)
        };

        let death_date = if self.form_data.death_date.is_empty() {
            None
        } else {
            Some(NaiveDate::parse_from_str(&self.form_data.death_date, "%Y-%m-%d")
                .map_err(|_| anyhow::anyhow!("Ogiltigt dödsdatum (använd YYYY-MM-DD)"))?)
        };

        // Kontrollera att katalognamn är unikt
        if !db.persons().is_directory_name_unique(&self.form_data.directory_name, state.editing_person_id)? {
            return Err(anyhow::anyhow!("Katalognamnet används redan"));
        }

        let firstname = if self.form_data.firstname.is_empty() {
            None
        } else {
            Some(self.form_data.firstname.clone())
        };

        let surname = if self.form_data.surname.is_empty() {
            None
        } else {
            Some(self.form_data.surname.clone())
        };

        if let Some(person_id) = state.editing_person_id {
            // Uppdatera befintlig
            let mut person = db.persons().find_by_id(person_id)?
                .ok_or_else(|| anyhow::anyhow!("Person hittades inte"))?;

            person.firstname = firstname;
            person.surname = surname;
            person.birth_date = birth_date;
            person.death_date = death_date;
            person.directory_name = self.form_data.directory_name.clone();

            db.persons().update(&mut person)?;
        } else {
            // Skapa ny
            let mut person = Person {
                id: None,
                firstname,
                surname,
                birth_date,
                death_date,
                age: None,
                directory_name: self.form_data.directory_name.clone(),
                profile_image_path: None,
                created_at: None,
                updated_at: None,
            };

            db.persons().create(&mut person)?;

            // Skapa katalog
            let config = db.config().get()?;
            let person_dir = config.persons_directory().join(&person.directory_name);
            std::fs::create_dir_all(&person_dir)?;
        }

        Ok(())
    }

    fn reset(&mut self) {
        self.form_data.clear();
        self.error_message = None;
        self.auto_generate_dir = true;
    }
}
