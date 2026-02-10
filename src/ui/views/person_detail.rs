use egui::{self, ColorImage, RichText, TextureHandle, TextureOptions};
use std::path::{Path, PathBuf};

use crate::db::Database;
use crate::models::{Document, Person, RelationshipType};
use crate::ui::{
    state::{AppState, ConfirmAction},
    theme::{Colors, Icons},
    widgets::{ChecklistPanel, ImageGallery},
    View,
};
use crate::utils::file_ops;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum PersonDetailTab {
    #[default]
    PersonInfo,
    Documents,
    Images,
    Checklist,
}

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
    /// Vald flik
    selected_tab: PersonDetailTab,
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
            selected_tab: PersonDetailTab::default(),
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

        // Flikrad
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.selected_tab, PersonDetailTab::PersonInfo, format!("{} Personuppgifter", Icons::PERSON));
            ui.selectable_value(&mut self.selected_tab, PersonDetailTab::Documents, format!("{} Dokument", Icons::DOCUMENT));
            ui.selectable_value(&mut self.selected_tab, PersonDetailTab::Images, format!("{} Bilder", Icons::IMAGE));
            ui.selectable_value(&mut self.selected_tab, PersonDetailTab::Checklist, format!("{} Checklista", Icons::CHECK));
        });

        ui.separator();

        // Flikinnehåll
        egui::ScrollArea::vertical().show(ui, |ui| {
            match self.selected_tab {
                PersonDetailTab::PersonInfo => {
                    ui.columns(2, |columns| {
                        columns[0].vertical(|ui| {
                            Self::show_person_info_static(ui, &person, self.profile_texture.as_ref());
                        });
                        columns[1].vertical(|ui| {
                            Self::show_relations_static(ui, state, db, person_id);
                        });
                    });
                }
                PersonDetailTab::Documents => {
                    Self::show_documents_panel(ui, state, db, person_id, document_count);
                }
                PersonDetailTab::Images => {
                    self.show_image_gallery_panel(ui, state, db, person_id);
                }
                PersonDetailTab::Checklist => {
                    self.checklist_panel.show(ui, state, db, person_id);
                }
            }
        });
    }

    fn show_image_gallery_panel(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database, person_id: i64) {
        // Hämta media root från config
        let media_root = db
            .config()
            .get()
            .map(|c| c.media_directory_path.clone())
            .unwrap_or_default();

        // Hämta person directory name för upload
        let person_dir = self.person_cache.as_ref()
            .map(|p| p.directory_name.clone())
            .unwrap_or_default();

        // Kolla drag-and-drop (måste göras före UI-rendering)
        let dropped_files: Vec<PathBuf> = ui.ctx().input(|i| {
            i.raw.dropped_files.iter()
                .filter_map(|f| f.path.clone())
                .filter(|p| file_ops::is_image_path(p))
                .collect()
        });

        // Visa drop-overlay om filer hovrar — kontrollera path eller MIME
        let is_hovering_files = ui.ctx().input(|i| {
            !i.raw.hovered_files.is_empty() && i.raw.hovered_files.iter().any(|f| {
                // Kolla path om tillgänglig
                if let Some(ref p) = f.path {
                    return file_ops::is_image_path(p);
                }
                // Kolla MIME-typ
                if f.mime.starts_with("image/") {
                    return true;
                }
                // Om varken path eller MIME finns, visa overlay ändå
                true
            })
        });

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

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // "+" knapp för att ladda upp bilder
                        if ui.small_button(format!("{}", Icons::ADD))
                            .on_hover_text("Ladda upp bilder")
                            .clicked()
                        {
                            if let Some(paths) = rfd::FileDialog::new()
                                .add_filter("Bilder", &["jpg", "jpeg", "png", "gif", "webp", "bmp"])
                                .pick_files()
                            {
                                match import_images_to_person(&paths, db, person_id, &person_dir, &media_root) {
                                    Ok(count) => {
                                        state.show_success(&format!("{} bild(er) importerade", count));
                                        self.image_gallery.mark_needs_refresh();
                                        self.needs_refresh = true;
                                    }
                                    Err(e) => {
                                        state.show_error(&format!("Import misslyckades: {}", e));
                                    }
                                }
                            }
                        }
                    });
                });

                // Drag-and-drop overlay
                if is_hovering_files {
                    ui.add_space(8.0);
                    egui::Frame::none()
                        .fill(Colors::PRIMARY.linear_multiply(0.15))
                        .rounding(8.0)
                        .inner_margin(24.0)
                        .stroke(egui::Stroke::new(2.0, Colors::PRIMARY))
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    RichText::new(format!("{} Släpp bilder här", Icons::IMPORT))
                                        .size(16.0)
                                        .color(Colors::PRIMARY),
                                );
                            });
                        });
                }

                ui.add_space(8.0);

                // Visa galleriet och hantera actions
                let action = self.image_gallery.show(ui, db, person_id, &media_root);

                // Hantera profilbild-val
                if let Some(profile_path) = action.set_profile_image {
                    // Bygg full relativ sökväg från media_root: persons/<dir>/<relative_path>
                    let full_profile_path = format!("persons/{}/{}", person_dir, profile_path);
                    match db.persons().set_profile_image(person_id, Some(&full_profile_path)) {
                        Ok(()) => {
                            state.show_success("Profilbild uppdaterad");
                            self.needs_refresh = true;
                        }
                        Err(e) => {
                            state.show_error(&format!("Kunde inte sätta profilbild: {}", e));
                        }
                    }
                }

                // Hantera dubbelklick → öppna dokumentvisaren
                if let Some(doc_id) = action.open_document {
                    state.navigate_to_document(doc_id);
                }

                // Hantera radering
                if let Some((doc_id, filename)) = action.delete_image {
                    state.show_confirm(
                        &format!("Vill du radera bilden \"{}\"? Filen tas bort permanent.", filename),
                        ConfirmAction::DeleteDocument(doc_id),
                    );
                }
            });

        // Hantera droppade filer
        if !dropped_files.is_empty() {
            match import_images_to_person(&dropped_files, db, person_id, &person_dir, &media_root) {
                Ok(count) => {
                    state.show_success(&format!("{} bild(er) importerade", count));
                    self.image_gallery.mark_needs_refresh();
                    self.needs_refresh = true;
                }
                Err(e) => {
                    state.show_error(&format!("Import misslyckades: {}", e));
                }
            }
        }
    }

    fn show_person_info_static(ui: &mut egui::Ui, person: &Person, profile_texture: Option<&TextureHandle>) {
        egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .rounding(8.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.set_min_height(ui.available_height());
                ui.heading("Personuppgifter");
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    // Vänster: info-grid
                    ui.vertical(|ui| {
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

                    // Höger: profilbild (thumbnail-storlek)
                    if let Some(tex) = profile_texture {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            let size = egui::vec2(80.0, 80.0);
                            ui.add(egui::Image::new(tex).fit_to_exact_size(size).rounding(8.0));
                        });
                    }
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
                ui.set_min_height(ui.available_height());
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

                egui::ScrollArea::vertical().show(ui, |ui| {
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

/// Importera bildfiler till en persons katalog och skapa DB-poster
fn import_images_to_person(
    paths: &[PathBuf],
    db: &Database,
    person_id: i64,
    person_dir: &str,
    media_root: &Path,
) -> Result<usize, String> {
    if paths.is_empty() {
        return Ok(0);
    }

    // Hitta bildtyp (target_directory som börjar med "bilder/")
    let image_type_id = db.documents().get_all_types()
        .unwrap_or_default()
        .into_iter()
        .find(|t| t.target_directory.starts_with("bilder"))
        .and_then(|t| t.id);

    // Bestäm målkatalog
    let target_subdir = "bilder";
    let dest_dir = media_root.join("persons").join(person_dir).join(target_subdir);

    let mut imported = 0;

    for path in paths {
        let original_filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("bild.jpg")
            .to_string();

        // Generera unikt filnamn
        let filename = file_ops::unique_filename(&dest_dir, &original_filename);

        // Kopiera fil
        match file_ops::copy_file_to_directory(path, &dest_dir, &filename) {
            Ok(dest_path) => {
                // Skapa relativ sökväg (relativt personkatalogen)
                let relative_path = format!("{}/{}", target_subdir, filename);

                // Hämta filstorlek
                let file_size = file_ops::get_file_size(&dest_path).unwrap_or(0) as i64;
                let file_modified = file_ops::get_modified_time(&dest_path).ok();

                // Skapa Document-post
                let mut doc = Document::new(person_id, filename.clone(), relative_path);
                doc.document_type_id = image_type_id;
                doc.file_size = file_size;
                doc.file_modified_at = file_modified;

                match db.documents().create(&mut doc) {
                    Ok(_) => imported += 1,
                    Err(e) => {
                        eprintln!("Kunde inte skapa dokument-post för {}: {}", filename, e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Kunde inte kopiera {}: {}", original_filename, e);
            }
        }
    }

    if imported > 0 {
        Ok(imported)
    } else {
        Err("Inga bilder kunde importeras".to_string())
    }
}
