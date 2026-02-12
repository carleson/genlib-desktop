use std::collections::HashMap;
use std::path::PathBuf;

use chrono::NaiveDate;
use egui::{self, ColorImage, RichText, TextureHandle, TextureOptions};

use crate::db::{Database, SearchField, SearchFilter};
use crate::models::Person;
use crate::ui::{state::AppState, theme::{Colors, Icons}};

pub struct PersonListView {
    /// Sökfilter
    filter: SearchFilter,
    /// Visa avancerade filter
    show_advanced_filters: bool,
    /// Temporära värden för datumfält (strängar för enklare redigering)
    birth_after_str: String,
    birth_before_str: String,
    death_after_str: String,
    death_before_str: String,
    /// Cache
    persons_cache: Vec<Person>,
    bookmarks_cache: std::collections::HashSet<i64>,
    needs_refresh: bool,
    /// Cachade profilbild-texturer (person_id -> texture)
    profile_textures: HashMap<i64, TextureHandle>,
    /// Media root för att ladda bilder
    media_root: Option<PathBuf>,
}

impl PersonListView {
    pub fn new() -> Self {
        Self {
            filter: SearchFilter::new(),
            show_advanced_filters: false,
            birth_after_str: String::new(),
            birth_before_str: String::new(),
            death_after_str: String::new(),
            death_before_str: String::new(),
            persons_cache: Vec::new(),
            bookmarks_cache: std::collections::HashSet::new(),
            needs_refresh: true,
            profile_textures: HashMap::new(),
            media_root: None,
        }
    }

    /// Parsa datumfält (YYYY-MM-DD eller YYYY)
    fn parse_date(s: &str) -> Option<NaiveDate> {
        if s.is_empty() {
            return None;
        }
        // Försök full format först
        if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            return Some(date);
        }
        // Försök bara år (antar 1 januari)
        if let Ok(year) = s.parse::<i32>() {
            return NaiveDate::from_ymd_opt(year, 1, 1);
        }
        None
    }

    /// Returnerar true om en person valdes
    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) -> bool {
        let mut person_selected = false;

        // Refresh data om nödvändigt
        if self.needs_refresh {
            self.refresh_persons(db);
            self.needs_refresh = false;
        }

        // Ladda profilbilder för personer i cachen
        self.load_profile_textures(ui.ctx());

        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading(format!("{} Personer", Icons::PEOPLE));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(format!("{} Ny person", Icons::ADD)).clicked() {
                        state.open_new_person_form();
                    }
                });
            });

            ui.add_space(8.0);

            // Fokusera sökfältet om signalerat
            let search_id = egui::Id::new("person_search_field");
            if state.focus_search {
                ui.memory_mut(|m| m.request_focus(search_id));
                state.focus_search = false;
            }

            // Sökfält och grundläggande filter
            ui.horizontal(|ui| {
                // Sökfält
                ui.label(Icons::SEARCH);
                let search_response = ui.add(
                    egui::TextEdit::singleline(&mut self.filter.query)
                        .id(search_id)
                        .hint_text("Sök...")
                        .desired_width(200.0)
                );
                if search_response.changed() {
                    self.needs_refresh = true;
                }
                egui::ComboBox::from_id_salt("search_field")
                    .selected_text(match self.filter.search_field {
                        SearchField::Name => "Namn",
                        SearchField::Firstname => "Förnamn",
                        SearchField::Surname => "Efternamn",
                        SearchField::Directory => "Katalog",
                    })
                    .width(90.0)
                    .show_ui(ui, |ui| {
                        if ui.selectable_value(&mut self.filter.search_field, SearchField::Name, "Namn").changed() {
                            self.needs_refresh = true;
                        }
                        if ui.selectable_value(&mut self.filter.search_field, SearchField::Firstname, "Förnamn").changed() {
                            self.needs_refresh = true;
                        }
                        if ui.selectable_value(&mut self.filter.search_field, SearchField::Surname, "Efternamn").changed() {
                            self.needs_refresh = true;
                        }
                        if ui.selectable_value(&mut self.filter.search_field, SearchField::Directory, "Katalog").changed() {
                            self.needs_refresh = true;
                        }
                    });

                ui.separator();

                // Filter: Levande/Avlidna
                if ui.selectable_label(self.filter.filter_alive.is_none(), "Alla").clicked() {
                    self.filter.filter_alive = None;
                    self.needs_refresh = true;
                }
                if ui.selectable_label(self.filter.filter_alive == Some(true), "Levande").clicked() {
                    self.filter.filter_alive = Some(true);
                    self.needs_refresh = true;
                }
                if ui.selectable_label(self.filter.filter_alive == Some(false), "Avlidna").clicked() {
                    self.filter.filter_alive = Some(false);
                    self.needs_refresh = true;
                }

                ui.separator();

                // Toggle för avancerade filter
                let advanced_label = if self.filter.has_advanced_filters() {
                    format!("{} Filter aktiva", Icons::FILTER)
                } else {
                    format!("{} Fler filter", Icons::FILTER)
                };
                if ui.selectable_label(self.show_advanced_filters, advanced_label).clicked() {
                    self.show_advanced_filters = !self.show_advanced_filters;
                }
            });

            // Avancerade filter (collapsible)
            if self.show_advanced_filters {
                ui.add_space(4.0);
                egui::Frame::none()
                    .fill(ui.visuals().faint_bg_color)
                    .rounding(4.0)
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Avancerade filter").strong());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("Återställ").clicked() {
                                    self.filter.reset_advanced();
                                    self.birth_after_str.clear();
                                    self.birth_before_str.clear();
                                    self.death_after_str.clear();
                                    self.death_before_str.clear();
                                    self.needs_refresh = true;
                                }
                            });
                        });

                        ui.add_space(4.0);

                        // Rad 1: Datumfilter
                        ui.horizontal(|ui| {
                            ui.label("Född:");
                            ui.label(RichText::new("från").small().color(Colors::TEXT_MUTED));
                            let resp = ui.add(
                                egui::TextEdit::singleline(&mut self.birth_after_str)
                                    .hint_text("YYYY eller YYYY-MM-DD")
                                    .desired_width(120.0)
                            );
                            if resp.changed() {
                                self.filter.birth_after = Self::parse_date(&self.birth_after_str);
                                self.needs_refresh = true;
                            }

                            ui.label(RichText::new("till").small().color(Colors::TEXT_MUTED));
                            let resp = ui.add(
                                egui::TextEdit::singleline(&mut self.birth_before_str)
                                    .hint_text("YYYY eller YYYY-MM-DD")
                                    .desired_width(120.0)
                            );
                            if resp.changed() {
                                self.filter.birth_before = Self::parse_date(&self.birth_before_str);
                                self.needs_refresh = true;
                            }

                            ui.separator();

                            ui.label("Död:");
                            ui.label(RichText::new("från").small().color(Colors::TEXT_MUTED));
                            let resp = ui.add(
                                egui::TextEdit::singleline(&mut self.death_after_str)
                                    .hint_text("YYYY")
                                    .desired_width(80.0)
                            );
                            if resp.changed() {
                                self.filter.death_after = Self::parse_date(&self.death_after_str);
                                self.needs_refresh = true;
                            }

                            ui.label(RichText::new("till").small().color(Colors::TEXT_MUTED));
                            let resp = ui.add(
                                egui::TextEdit::singleline(&mut self.death_before_str)
                                    .hint_text("YYYY")
                                    .desired_width(80.0)
                            );
                            if resp.changed() {
                                self.filter.death_before = Self::parse_date(&self.death_before_str);
                                self.needs_refresh = true;
                            }
                        });

                        ui.add_space(4.0);

                        // Rad 2: Övriga filter
                        ui.horizontal(|ui| {
                            // Bokmärken
                            if ui.checkbox(&mut self.filter.only_bookmarked, "Endast bokmärkta").changed() {
                                self.needs_refresh = true;
                            }

                            ui.separator();

                            // Har relationer
                            ui.label("Relationer:");
                            let rel_text = match self.filter.has_relations {
                                None => "Alla",
                                Some(true) => "Har",
                                Some(false) => "Saknar",
                            };
                            egui::ComboBox::from_id_salt("filter_relations")
                                .selected_text(rel_text)
                                .show_ui(ui, |ui| {
                                    if ui.selectable_label(self.filter.has_relations.is_none(), "Alla").clicked() {
                                        self.filter.has_relations = None;
                                        self.needs_refresh = true;
                                    }
                                    if ui.selectable_label(self.filter.has_relations == Some(true), "Har relationer").clicked() {
                                        self.filter.has_relations = Some(true);
                                        self.needs_refresh = true;
                                    }
                                    if ui.selectable_label(self.filter.has_relations == Some(false), "Saknar relationer").clicked() {
                                        self.filter.has_relations = Some(false);
                                        self.needs_refresh = true;
                                    }
                                });

                            ui.separator();

                            // Har dokument
                            ui.label("Dokument:");
                            let doc_text = match self.filter.has_documents {
                                None => "Alla",
                                Some(true) => "Har",
                                Some(false) => "Saknar",
                            };
                            egui::ComboBox::from_id_salt("filter_documents")
                                .selected_text(doc_text)
                                .show_ui(ui, |ui| {
                                    if ui.selectable_label(self.filter.has_documents.is_none(), "Alla").clicked() {
                                        self.filter.has_documents = None;
                                        self.needs_refresh = true;
                                    }
                                    if ui.selectable_label(self.filter.has_documents == Some(true), "Har dokument").clicked() {
                                        self.filter.has_documents = Some(true);
                                        self.needs_refresh = true;
                                    }
                                    if ui.selectable_label(self.filter.has_documents == Some(false), "Saknar dokument").clicked() {
                                        self.filter.has_documents = Some(false);
                                        self.needs_refresh = true;
                                    }
                                });

                            ui.separator();

                            // Har profilbild
                            ui.label("Profilbild:");
                            let img_text = match self.filter.has_profile_image {
                                None => "Alla",
                                Some(true) => "Har",
                                Some(false) => "Saknar",
                            };
                            egui::ComboBox::from_id_salt("filter_profile_image")
                                .selected_text(img_text)
                                .show_ui(ui, |ui| {
                                    if ui.selectable_label(self.filter.has_profile_image.is_none(), "Alla").clicked() {
                                        self.filter.has_profile_image = None;
                                        self.needs_refresh = true;
                                    }
                                    if ui.selectable_label(self.filter.has_profile_image == Some(true), "Har profilbild").clicked() {
                                        self.filter.has_profile_image = Some(true);
                                        self.needs_refresh = true;
                                    }
                                    if ui.selectable_label(self.filter.has_profile_image == Some(false), "Saknar profilbild").clicked() {
                                        self.filter.has_profile_image = Some(false);
                                        self.needs_refresh = true;
                                    }
                                });
                        });
                    });
            }

            ui.add_space(8.0);

            // Statistik
            ui.label(RichText::new(format!("{} personer", self.persons_cache.len())).small().color(Colors::TEXT_SECONDARY));

            ui.separator();

            // Personlista
            egui::ScrollArea::vertical().show(ui, |ui| {
                for person in &self.persons_cache {
                    let is_bookmarked = person.id.map(|id| self.bookmarks_cache.contains(&id)).unwrap_or(false);

                    let response = egui::Frame::none()
                        .fill(ui.visuals().extreme_bg_color)
                        .rounding(4.0)
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Profilbild eller placeholder
                                let thumb_size = egui::vec2(32.0, 32.0);
                                if let Some(texture) = person.id.and_then(|id| self.profile_textures.get(&id)) {
                                    ui.add(egui::Image::new(texture).fit_to_exact_size(thumb_size).rounding(16.0));
                                } else {
                                    // Placeholder med ikon
                                    let (rect, _) = ui.allocate_exact_size(thumb_size, egui::Sense::hover());
                                    ui.painter().circle_filled(rect.center(), 16.0, ui.visuals().faint_bg_color);
                                    ui.painter().text(
                                        rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        Icons::PERSON,
                                        egui::FontId::proportional(14.0),
                                        Colors::TEXT_MUTED,
                                    );
                                }

                                // Info
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(person.full_name()).strong());

                                        if is_bookmarked {
                                            ui.label(RichText::new(Icons::BOOKMARK).color(Colors::WARNING));
                                        }
                                    });

                                    let years = person.years_display();
                                    if !years.is_empty() {
                                        ui.label(RichText::new(years).small().color(Colors::TEXT_SECONDARY));
                                    }
                                });

                                // Katalognamn till höger
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(&person.directory_name).small().color(Colors::TEXT_MUTED));
                                });
                            });
                        });

                    // Gör hela raden klickbar
                    if response.response.interact(egui::Sense::click()).clicked() {
                        if let Some(id) = person.id {
                            state.selected_person_id = Some(id);
                            person_selected = true;
                        }
                    }

                    ui.add_space(4.0);
                }
            });
        });

        person_selected
    }

    fn refresh_persons(&mut self, db: &Database) {
        // Hämta media root
        self.media_root = db
            .config()
            .get()
            .ok()
            .map(|c| c.media_directory_path);

        // Hämta personer med avancerad sökning
        self.persons_cache = db
            .persons()
            .advanced_search(&self.filter)
            .unwrap_or_default();

        // Sortera på efternamn, sedan förnamn
        self.persons_cache.sort_by(|a, b| {
            a.surname.cmp(&b.surname).then(a.firstname.cmp(&b.firstname))
        });

        // Hämta bokmärken
        self.bookmarks_cache = db
            .persons()
            .get_bookmarked()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|p| p.id)
            .collect();

        // Rensa textur-cache vid refresh (profilbilder kan ha ändrats)
        self.profile_textures.clear();
    }

    /// Ladda profilbilder för alla personer i cachen
    fn load_profile_textures(&mut self, ctx: &egui::Context) {
        let Some(ref media_root) = self.media_root else {
            return;
        };

        for person in &self.persons_cache {
            let Some(person_id) = person.id else {
                continue;
            };

            // Hoppa över om redan laddad
            if self.profile_textures.contains_key(&person_id) {
                continue;
            }

            // Hoppa över om ingen profilbild
            let Some(ref profile_path) = person.profile_image_path else {
                continue;
            };

            // Ladda bilden
            let full_path = media_root.join(profile_path);
            if !full_path.exists() {
                continue;
            }

            if let Some(texture) = Self::load_thumbnail(ctx, &full_path, person_id) {
                self.profile_textures.insert(person_id, texture);
            }
        }
    }

    /// Ladda en bild som thumbnail-textur
    fn load_thumbnail(ctx: &egui::Context, path: &PathBuf, person_id: i64) -> Option<TextureHandle> {
        let image_data = std::fs::read(path).ok()?;
        let image = image::load_from_memory(&image_data).ok()?;

        // Skala ner för thumbnail (max 64x64)
        let thumbnail = image.thumbnail(64, 64);
        let rgba = thumbnail.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let pixels = rgba.into_raw();

        let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);
        let texture = ctx.load_texture(
            format!("profile_thumb_{}", person_id),
            color_image,
            TextureOptions::LINEAR,
        );

        Some(texture)
    }

    pub fn mark_needs_refresh(&mut self) {
        self.needs_refresh = true;
    }
}
