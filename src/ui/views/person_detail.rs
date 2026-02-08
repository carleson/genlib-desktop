use egui::{self, ColorImage, RichText, TextureHandle, TextureOptions};
use std::path::PathBuf;

use crate::db::Database;
use crate::models::{Person, RelationshipType};
use crate::ui::{
    state::{AppState, ConfirmAction},
    theme::{Colors, Icons},
    widgets::{ChecklistPanel, ImageGallery},
    View,
};

pub struct PersonDetailView {
    person_cache: Option<Person>,
    document_count: i64,
    is_bookmarked: bool,
    needs_refresh: bool,
    checklist_panel: ChecklistPanel,
    image_gallery: ImageGallery,
    /// Cachad profilbild-textur
    profile_texture: Option<TextureHandle>,
    /// Sökväg till cachad profilbild
    profile_texture_path: Option<String>,
}

impl PersonDetailView {
    pub fn new() -> Self {
        Self {
            person_cache: None,
            document_count: 0,
            is_bookmarked: false,
            needs_refresh: true,
            checklist_panel: ChecklistPanel::new(),
            image_gallery: ImageGallery::new(),
            profile_texture: None,
            profile_texture_path: None,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) {
        let Some(person_id) = state.selected_person_id else {
            ui.label("Ingen person vald");
            return;
        };

        // Refresh om nödvändigt eller om person_id ändrats
        if self.needs_refresh || self.person_cache.as_ref().and_then(|p| p.id) != Some(person_id) {
            self.refresh_person(db, person_id);
            self.needs_refresh = false;
        }

        let Some(person) = self.person_cache.clone() else {
            ui.label("Person hittades inte");
            return;
        };

        // Extrahera data för att undvika borrow-konflikter
        let person_name = person.full_name();
        let is_bookmarked = self.is_bookmarked;
        let document_count = self.document_count;

        // Hämta media root för profilbild
        let media_root = db
            .config()
            .get()
            .map(|c| c.media_directory_path.clone())
            .unwrap_or_default();

        // Ladda profilbild-textur om nödvändigt
        self.load_profile_texture(ui.ctx(), &media_root);

        // Tillbaka-knapp och header
        ui.horizontal(|ui| {
            if ui.button(format!("{} Tillbaka", Icons::ARROW_LEFT)).clicked() {
                state.navigate(View::PersonList);
            }

            ui.separator();

            // Visa profilbild om den finns
            if let Some(ref tex) = self.profile_texture {
                let size = egui::vec2(40.0, 40.0);
                ui.add(egui::Image::new(tex).fit_to_exact_size(size).rounding(20.0));
            }

            ui.heading(&person_name);

            // Bokmärke
            let bookmark_icon = if is_bookmarked {
                Icons::BOOKMARK
            } else {
                Icons::BOOKMARK_EMPTY
            };

            if ui.button(bookmark_icon).clicked() {
                if let Ok(is_now_bookmarked) = db.persons().toggle_bookmark(person_id) {
                    self.is_bookmarked = is_now_bookmarked;
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Radera
                if ui.button(format!("{} Radera", Icons::DELETE)).clicked() {
                    state.show_confirm(
                        &format!("Vill du verkligen radera {}?", person_name),
                        ConfirmAction::DeletePerson(person_id),
                    );
                }

                // Redigera
                if ui.button(format!("{} Redigera", Icons::EDIT)).clicked() {
                    state.open_edit_person_form(person_id);
                }
            });
        });

        ui.add_space(16.0);

        // ScrollArea för allt innehåll
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Rad 1: Personuppgifter och relationer
            ui.columns(2, |columns| {
                // Vänster kolumn - personuppgifter
                columns[0].vertical(|ui| {
                    Self::show_person_info_static(ui, &person);
                });

                // Höger kolumn - relationer
                columns[1].vertical(|ui| {
                    Self::show_relations_static(ui, state, db, person_id);
                });
            });

            ui.add_space(16.0);

            // Rad 2: Dokument och checklista
            ui.columns(2, |columns| {
                // Vänster kolumn - dokument
                columns[0].vertical(|ui| {
                    Self::show_documents_panel(ui, state, db, person_id, document_count);
                });

                // Höger kolumn - checklista
                columns[1].vertical(|ui| {
                    self.checklist_panel.show(ui, state, db, person_id);
                });
            });

            ui.add_space(16.0);

            // Rad 3: Bildgalleri
            self.show_image_gallery_panel(ui, state, db, person_id);
        });
    }

    fn show_image_gallery_panel(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database, person_id: i64) {
        // Hämta media root från config
        let media_root = db
            .config()
            .get()
            .map(|c| c.media_directory_path.clone())
            .unwrap_or_default();

        egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .rounding(8.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.heading(format!("{} Bilder", Icons::IMAGE));
                    let count = self.image_gallery.image_count();
                    if count > 0 {
                        ui.label(RichText::new(format!("({})", count)).color(Colors::TEXT_MUTED));
                    }
                });
                ui.add_space(8.0);

                // Visa galleriet och hantera profilbild-val
                if let Some(profile_path) = self.image_gallery.show(ui, db, person_id, &media_root) {
                    // Användaren valde att sätta en bild som profilbild
                    match db.persons().set_profile_image(person_id, Some(&profile_path)) {
                        Ok(()) => {
                            state.show_success("Profilbild uppdaterad");
                            self.needs_refresh = true;
                        }
                        Err(e) => {
                            state.show_error(&format!("Kunde inte sätta profilbild: {}", e));
                        }
                    }
                }
            });
    }

    fn show_person_info_static(ui: &mut egui::Ui, person: &Person) {
        egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .rounding(8.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.heading("Personuppgifter");
                ui.add_space(8.0);

                egui::Grid::new("person_info_grid")
                    .num_columns(2)
                    .spacing([16.0, 8.0])
                    .show(ui, |ui| {
                        if let Some(ref firstname) = person.firstname {
                            ui.label(RichText::new("Förnamn:").color(Colors::TEXT_SECONDARY));
                            ui.label(firstname);
                            ui.end_row();
                        }

                        if let Some(ref surname) = person.surname {
                            ui.label(RichText::new("Efternamn:").color(Colors::TEXT_SECONDARY));
                            ui.label(surname);
                            ui.end_row();
                        }

                        if let Some(birth_date) = person.birth_date {
                            ui.label(RichText::new("Födelsedatum:").color(Colors::TEXT_SECONDARY));
                            ui.label(format!("{} {}", Icons::CALENDAR, birth_date.format("%Y-%m-%d")));
                            ui.end_row();
                        }

                        if let Some(death_date) = person.death_date {
                            ui.label(RichText::new("Dödsdatum:").color(Colors::TEXT_SECONDARY));
                            ui.label(format!("{} {}", Icons::CALENDAR, death_date.format("%Y-%m-%d")));
                            ui.end_row();
                        }

                        if let Some(age) = person.age {
                            ui.label(RichText::new("Ålder:").color(Colors::TEXT_SECONDARY));
                            ui.label(format!("{} år", age));
                            ui.end_row();
                        }

                        ui.label(RichText::new("Katalog:").color(Colors::TEXT_SECONDARY));
                        ui.label(&person.directory_name);
                        ui.end_row();
                    });

            });
    }

    fn show_relations_static(ui: &mut egui::Ui, state: &mut AppState, db: &Database, person_id: i64) {
        egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .rounding(8.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.heading("Relationer");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button(format!("{}", Icons::ADD)).clicked() {
                            state.show_relationship_form = true;
                        }
                    });
                });
                ui.add_space(8.0);

                let grouped = db.relationships().find_by_person_grouped(person_id).unwrap_or_default();

                if grouped.is_empty() {
                    ui.label(RichText::new("Inga relationer").color(Colors::TEXT_MUTED));
                    return;
                }

                for (rel_type, views) in grouped {
                    let color = match rel_type {
                        RelationshipType::Parent => Colors::PARENT,
                        RelationshipType::Child => Colors::CHILD,
                        RelationshipType::Spouse => Colors::SPOUSE,
                        RelationshipType::Sibling => Colors::SIBLING,
                    };

                    ui.horizontal(|ui| {
                        ui.label(RichText::new(rel_type.display_name()).strong().color(color));
                    });

                    for view in views {
                        ui.horizontal(|ui| {
                            ui.add_space(16.0);
                            if ui.link(&view.other_person_name).clicked() {
                                state.navigate_to_person(view.other_person_id);
                            }

                            // Delete-knapp för relation
                            if ui
                                .small_button(RichText::new(Icons::DELETE).color(Colors::TEXT_MUTED))
                                .on_hover_text("Ta bort relation")
                                .clicked()
                            {
                                state.show_confirm(
                                    &format!(
                                        "Ta bort relation till {}?",
                                        view.other_person_name
                                    ),
                                    ConfirmAction::DeleteRelationship(view.relationship_id),
                                );
                            }
                        });
                    }

                    ui.add_space(4.0);
                }
            });
    }

    fn show_documents_panel(
        ui: &mut egui::Ui,
        state: &mut AppState,
        db: &Database,
        person_id: i64,
        document_count: i64,
    ) {
        egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .rounding(8.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.heading(format!("{} Dokument", Icons::DOCUMENT));
                    ui.label(RichText::new(format!("({})", document_count)).color(Colors::TEXT_MUTED));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Skapa nytt dokument
                        if ui.small_button(format!("{}", Icons::ADD)).on_hover_text("Skapa nytt dokument").clicked() {
                            state.open_document_create();
                        }

                        // Importera filer
                        if ui.small_button(format!("{}", Icons::IMPORT)).on_hover_text("Importera filer").clicked() {
                            state.open_document_import();
                        }

                        // Synkronisera knapp
                        if ui.small_button("🔄").on_hover_text("Synkronisera från filsystem").clicked() {
                            // Synkronisera dokument
                            if let Ok(Some(person)) = db.persons().find_by_id(person_id) {
                                use crate::services::DocumentSyncService;
                                let sync_service = DocumentSyncService::new(db);
                                match sync_service.sync_person(&person) {
                                    Ok(sync_result) => {
                                        if sync_result.has_changes() {
                                            state.show_success(&format!(
                                                "Synkronisering klar: {}",
                                                sync_result.summary()
                                            ));
                                        } else {
                                            state.show_status("Inga ändringar", crate::ui::StatusType::Info);
                                        }
                                    }
                                    Err(e) => {
                                        state.show_error(&format!("Synkronisering misslyckades: {}", e));
                                    }
                                }
                            }
                        }
                    });
                });
                ui.add_space(8.0);

                let grouped = db.documents().find_by_person_grouped(person_id).unwrap_or_default();

                if grouped.is_empty() {
                    ui.label(RichText::new("Inga dokument").color(Colors::TEXT_MUTED));
                    ui.add_space(8.0);
                    ui.label(RichText::new("Klicka + för att skapa, 📥 för att importera, eller 🔄 för att synka.").small().color(Colors::TEXT_MUTED));
                    return;
                }

                egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                    for (doc_type, docs) in grouped {
                        let type_name = doc_type.map(|t| t.name).unwrap_or_else(|| "Okategoriserade".to_string());

                        ui.collapsing(format!("{} ({})", type_name, docs.len()), |ui| {
                            for doc in docs {
                                let doc_id = doc.id;

                                ui.horizontal(|ui| {
                                    let icon = if doc.is_image() {
                                        Icons::IMAGE
                                    } else if doc.is_pdf() {
                                        Icons::DOCUMENT
                                    } else {
                                        Icons::NOTE
                                    };

                                    ui.label(icon);

                                    // Klickbart filnamn
                                    if ui.link(&doc.filename).clicked() {
                                        if let Some(id) = doc_id {
                                            state.navigate_to_document(id);
                                        }
                                    }

                                    ui.label(RichText::new(doc.file_size_display()).small().color(Colors::TEXT_MUTED));
                                });
                            }
                        });
                    }
                });
            });
    }

    fn refresh_person(&mut self, db: &Database, person_id: i64) {
        self.person_cache = db.persons().find_by_id(person_id).unwrap_or(None);
        self.document_count = db.documents().count_by_person(person_id).unwrap_or(0);
        self.is_bookmarked = db.persons().is_bookmarked(person_id).unwrap_or(false);

        // Rensa profilbild-cache om sökvägen ändrats
        if let Some(ref person) = self.person_cache {
            if self.profile_texture_path != person.profile_image_path {
                self.profile_texture = None;
                self.profile_texture_path = person.profile_image_path.clone();
            }
        }
    }

    fn load_profile_texture(&mut self, ctx: &egui::Context, media_root: &PathBuf) {
        // Returnera om redan laddad eller ingen sökväg
        if self.profile_texture.is_some() {
            return;
        }

        let Some(ref path) = self.profile_texture_path else {
            return;
        };

        let full_path = media_root.join(path);
        if !full_path.exists() {
            return;
        }

        // Ladda bilden
        if let Ok(image_data) = std::fs::read(&full_path) {
            if let Ok(image) = image::load_from_memory(&image_data) {
                let rgba = image.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.into_raw();
                let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);

                let texture = ctx.load_texture(
                    format!("profile_{}", path),
                    color_image,
                    TextureOptions::LINEAR,
                );

                self.profile_texture = Some(texture);
            }
        }
    }

    pub fn mark_needs_refresh(&mut self) {
        self.needs_refresh = true;
        self.checklist_panel.mark_needs_refresh();
        self.image_gallery.mark_needs_refresh();
        self.profile_texture = None;
        self.profile_texture_path = None;
    }
}
