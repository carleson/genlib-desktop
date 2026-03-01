use egui::{self, ColorImage, RichText, TextureHandle, TextureOptions};
use std::collections::HashMap;

use crate::db::Database;
use crate::models::{Resource, ResourceAddress, ResourceDocument, ResourceType};
use crate::ui::{
    state::{AppState, ConfirmAction},
    theme::{Colors, Icons},
    View,
};
use crate::utils::file_ops;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ResourceDetailTab {
    #[default]
    Info,
    Images,
    Documents,
}

pub struct ResourceDetailView {
    resource_cache: Option<Resource>,
    type_cache: Option<ResourceType>,
    addresses_cache: Vec<ResourceAddress>,
    documents_cache: Vec<ResourceDocument>,
    needs_refresh: bool,
    selected_tab: ResourceDetailTab,
    /// Cachade miniatyrer (document_id -> texture)
    thumbnails: HashMap<i64, TextureHandle>,
    /// Vald bild för lightbox
    lightbox_image: Option<i64>,
    /// Lightbox-textur (fullstorlek)
    lightbox_texture: Option<TextureHandle>,
    /// Inline-formulär för ny adress
    show_add_address: bool,
    new_addr_street: String,
    new_addr_postal_code: String,
    new_addr_city: String,
    new_addr_country: String,
}

impl ResourceDetailView {
    pub fn new() -> Self {
        Self {
            resource_cache: None,
            type_cache: None,
            addresses_cache: Vec::new(),
            documents_cache: Vec::new(),
            needs_refresh: true,
            selected_tab: ResourceDetailTab::default(),
            thumbnails: HashMap::new(),
            lightbox_image: None,
            lightbox_texture: None,
            show_add_address: false,
            new_addr_street: String::new(),
            new_addr_postal_code: String::new(),
            new_addr_city: String::new(),
            new_addr_country: String::new(),
        }
    }

    pub fn mark_needs_refresh(&mut self) {
        self.needs_refresh = true;
        self.thumbnails.clear();
        self.lightbox_image = None;
        self.lightbox_texture = None;
        self.show_add_address = false;
        self.clear_address_form();
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) {
        let Some(resource_id) = state.selected_resource_id else {
            ui.label("Ingen resurs vald");
            return;
        };

        if self.needs_refresh
            || self.resource_cache.as_ref().and_then(|r| r.id) != Some(resource_id)
        {
            self.refresh(db, resource_id);
            self.needs_refresh = false;
        }

        let Some(resource) = self.resource_cache.clone() else {
            ui.label("Resursen hittades inte");
            return;
        };

        let media_root = db
            .config()
            .get()
            .map(|c| c.media_directory_path.display().to_string())
            .unwrap_or_default();

        let type_dir = self
            .type_cache
            .as_ref()
            .map(|t| t.directory_name.clone())
            .unwrap_or_default();

        let resource_dir = resource.directory_name.clone();

        // Ladda thumbnails för bilder
        self.load_thumbnails(ui.ctx(), &media_root, &type_dir, &resource_dir);

        // Header
        ui.horizontal(|ui| {
            if ui.button(format!("{} Tillbaka", Icons::ARROW_LEFT)).clicked() {
                state.navigate(View::ResourceList);
            }

            ui.add_space(8.0);
            ui.heading(format!("{} {}", Icons::LOCATION, resource.name));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(format!("{}", Icons::DELETE))
                    .on_hover_text("Ta bort resurs")
                    .clicked()
                {
                    state.show_confirm(
                        &format!("Ta bort resursen \"{}\"?", resource.name),
                        ConfirmAction::DeleteResource(resource_id),
                    );
                }
                if ui.button(format!("{} Redigera", Icons::EDIT)).clicked() {
                    state.open_edit_resource_form(resource_id);
                }
            });
        });

        if let Some(ref rtype) = self.type_cache {
            ui.label(
                RichText::new(format!("Typ: {}", rtype.name))
                    .color(Colors::TEXT_SECONDARY)
                    .size(12.0),
            );
        }

        ui.add_space(8.0);
        ui.separator();

        // Flikar
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.selected_tab, ResourceDetailTab::Info, "Info");
            ui.selectable_value(&mut self.selected_tab, ResourceDetailTab::Images, "Bilder");
            ui.selectable_value(&mut self.selected_tab, ResourceDetailTab::Documents, "Dokument");
        });

        ui.separator();
        ui.add_space(8.0);

        match self.selected_tab {
            ResourceDetailTab::Info => {
                self.show_info_tab(ui, state, db, &resource, &media_root, &type_dir, &resource_dir);
            }
            ResourceDetailTab::Images => {
                self.show_images_tab(ui, state, db, &resource, &media_root, &type_dir, &resource_dir);
            }
            ResourceDetailTab::Documents => {
                self.show_documents_tab(ui, state, db, &resource, &media_root, &type_dir, &resource_dir);
            }
        }
    }

    fn show_info_tab(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut AppState,
        db: &Database,
        resource: &Resource,
        media_root: &str,
        type_dir: &str,
        resource_dir: &str,
    ) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Information
            if let Some(ref info) = resource.information {
                if !info.is_empty() {
                    ui.label(RichText::new("Information").strong());
                    ui.label(info);
                    ui.add_space(8.0);
                }
            }

            // Kommentar
            if let Some(ref comment) = resource.comment {
                if !comment.is_empty() {
                    ui.label(RichText::new("Kommentar").strong());
                    ui.label(comment);
                    ui.add_space(8.0);
                }
            }

            // Koordinater
            if resource.lat.is_some() || resource.lon.is_some() {
                ui.label(RichText::new("Koordinater").strong());
                ui.horizontal(|ui| {
                    if let Some(lat) = resource.lat {
                        ui.label(format!("Lat: {:.6}", lat));
                    }
                    if let Some(lon) = resource.lon {
                        ui.label(format!("Lon: {:.6}", lon));
                    }
                });
                ui.add_space(8.0);
            }

            // Katalog
            {
                let resource_dir_path = std::path::Path::new(media_root)
                    .join("resurser")
                    .join(type_dir)
                    .join(resource_dir);
                ui.label(RichText::new("Katalog:").color(Colors::TEXT_SECONDARY));
                ui.horizontal(|ui| {
                    ui.label(resource_dir_path.display().to_string())
                        .on_hover_text(resource_dir_path.display().to_string());
                    if resource_dir_path.exists() {
                        if ui.small_button(Icons::FOLDER)
                            .on_hover_text("Öppna i filhanteraren")
                            .clicked()
                        {
                            open_in_file_explorer(&resource_dir_path);
                        }
                    }
                });
                ui.add_space(8.0);
            }

            // Adresser
            ui.horizontal(|ui| {
                ui.label(RichText::new("Adresser").strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if !self.show_add_address {
                        if ui.small_button(format!("{} Lägg till", Icons::ADD)).clicked() {
                            self.show_add_address = true;
                        }
                    }
                });
            });

            // Befintliga adresser
            if self.addresses_cache.is_empty() && !self.show_add_address {
                ui.label(RichText::new("Inga adresser").color(Colors::TEXT_MUTED).size(12.0));
            } else {
                let addresses = self.addresses_cache.clone();
                for addr in &addresses {
                    egui::Frame::none()
                        .fill(ui.visuals().extreme_bg_color)
                        .rounding(4.0)
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let display = addr.display();
                                if display.is_empty() {
                                    ui.label(RichText::new("(tom adress)").color(Colors::TEXT_MUTED));
                                } else {
                                    ui.label(&display);
                                }
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if let Some(addr_id) = addr.id {
                                        if ui.small_button(Icons::DELETE)
                                            .on_hover_text("Ta bort adress")
                                            .clicked()
                                        {
                                            state.show_confirm(
                                                "Ta bort adressen?",
                                                ConfirmAction::DeleteResourceAddress(addr_id),
                                            );
                                        }
                                    }
                                });
                            });
                        });
                    ui.add_space(4.0);
                }
            }

            // Inline-formulär för ny adress
            if self.show_add_address {
                if let Some(resource_id) = resource.id {
                    egui::Frame::none()
                        .fill(ui.visuals().extreme_bg_color)
                        .rounding(4.0)
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.label(RichText::new("Ny adress").strong().size(12.0));
                            ui.add_space(4.0);

                            egui::Grid::new("new_address_grid")
                                .num_columns(2)
                                .spacing([8.0, 4.0])
                                .show(ui, |ui| {
                                    ui.label("Gata:");
                                    ui.add(egui::TextEdit::singleline(&mut self.new_addr_street)
                                        .desired_width(240.0));
                                    ui.end_row();

                                    ui.label("Postnummer:");
                                    ui.add(egui::TextEdit::singleline(&mut self.new_addr_postal_code)
                                        .desired_width(100.0));
                                    ui.end_row();

                                    ui.label("Ort:");
                                    ui.add(egui::TextEdit::singleline(&mut self.new_addr_city)
                                        .desired_width(200.0));
                                    ui.end_row();

                                    ui.label("Land:");
                                    ui.add(egui::TextEdit::singleline(&mut self.new_addr_country)
                                        .desired_width(150.0));
                                    ui.end_row();
                                });

                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                if ui.button("Avbryt").clicked() {
                                    self.show_add_address = false;
                                    self.clear_address_form();
                                }
                                if ui.button(format!("{} Spara adress", Icons::SAVE)).clicked() {
                                    let mut addr = ResourceAddress::new(resource_id);
                                    addr.street = non_empty(&self.new_addr_street);
                                    addr.postal_code = non_empty(&self.new_addr_postal_code);
                                    addr.city = non_empty(&self.new_addr_city);
                                    addr.country = non_empty(&self.new_addr_country);

                                    match db.resources().create_address(&addr) {
                                        Ok(_) => {
                                            self.show_add_address = false;
                                            self.clear_address_form();
                                            self.refresh(db, resource_id);
                                            state.show_success("Adress sparad");
                                        }
                                        Err(e) => {
                                            state.show_error(&format!("Kunde inte spara adress: {}", e));
                                        }
                                    }
                                }
                            });
                        });
                }
            }
        });
    }

    fn show_images_tab(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut AppState,
        db: &Database,
        resource: &Resource,
        media_root: &str,
        type_dir: &str,
        resource_dir: &str,
    ) {
        let resource_id = resource.id.unwrap_or(0);

        ui.horizontal(|ui| {
            ui.label(RichText::new("Bilder").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("🔄").on_hover_text("Synkronisera från filsystem").clicked() {
                    let summary = self.sync_from_filesystem(db, resource_id, media_root, type_dir, resource_dir);
                    if summary.is_empty() {
                        state.show_status("Inga ändringar", crate::ui::StatusType::Info);
                    } else {
                        state.show_success(&format!("Synkronisering klar: {}", summary));
                    }
                }
                if ui.small_button(format!("{}", Icons::IMPORT)).on_hover_text("Ladda upp bild").clicked() {
                    self.upload_file(ui.ctx(), state, db, resource_id, media_root, type_dir, resource_dir, true);
                }
            });
        });

        ui.add_space(8.0);

        let images: Vec<&ResourceDocument> = self
            .documents_cache
            .iter()
            .filter(|d| d.is_image())
            .collect();

        if images.is_empty() {
            ui.label(RichText::new("Inga bilder uppladdade").color(Colors::TEXT_MUTED));
        } else {
            // Bildgrid
            egui::ScrollArea::vertical().show(ui, |ui| {
                let thumb_size = 160.0;
                let spacing = 8.0;
                let available_width = ui.available_width();
                let cols = ((available_width + spacing) / (thumb_size + spacing)).max(1.0) as usize;

                let docs = images.iter().map(|d| (*d).clone()).collect::<Vec<_>>();
                let rows = (docs.len() + cols - 1) / cols;

                for row_idx in 0..rows {
                    ui.horizontal(|ui| {
                        for col_idx in 0..cols {
                            let doc_idx = row_idx * cols + col_idx;
                            if doc_idx >= docs.len() {
                                break;
                            }
                            let doc = &docs[doc_idx];
                            let doc_id = doc.id.unwrap_or(0);

                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(thumb_size, thumb_size),
                                egui::Sense::click(),
                            );

                            if response.clicked() {
                                self.lightbox_image = Some(doc_id);
                                self.lightbox_texture = None;
                            }

                            if ui.is_rect_visible(rect) {
                                if let Some(tex) = self.thumbnails.get(&doc_id) {
                                    ui.painter().image(
                                        tex.id(),
                                        rect,
                                        egui::Rect::from_min_max(
                                            egui::pos2(0.0, 0.0),
                                            egui::pos2(1.0, 1.0),
                                        ),
                                        egui::Color32::WHITE,
                                    );
                                } else {
                                    ui.painter().rect_filled(
                                        rect,
                                        4.0,
                                        ui.visuals().extreme_bg_color,
                                    );
                                    ui.painter().text(
                                        rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        Icons::IMAGE,
                                        egui::FontId::proportional(24.0),
                                        ui.visuals().text_color(),
                                    );
                                }
                            }

                            response.on_hover_text(&doc.filename);
                        }
                    });
                    ui.add_space(spacing);
                }
            });
        }

        // Lightbox
        if let Some(lightbox_id) = self.lightbox_image {
            if let Some(doc) = self.documents_cache.iter().find(|d| d.id == Some(lightbox_id)) {
                let doc = doc.clone();
                let full_path = doc.full_path(media_root, type_dir, resource_dir);

                // Ladda lightbox-textur om det behövs
                if self.lightbox_texture.is_none() {
                    if let Ok(img) = image::open(&full_path) {
                        let rgba = img.to_rgba8();
                        let (w, h) = (rgba.width() as usize, rgba.height() as usize);
                        let color_image = ColorImage::from_rgba_unmultiplied([w, h], &rgba);
                        self.lightbox_texture = Some(
                            ui.ctx().load_texture("lb_resource", color_image, TextureOptions::LINEAR),
                        );
                    }
                }

                let mut close_lightbox = false;
                egui::Window::new("Bild")
                    .collapsible(false)
                    .resizable(true)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .default_size([800.0, 600.0])
                    .show(ui.ctx(), |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&doc.filename).strong());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(Icons::CROSS).clicked() {
                                    close_lightbox = true;
                                }
                                if ui.button(format!("{} Ta bort", Icons::DELETE))
                                    .on_hover_text("Ta bort bild")
                                    .clicked()
                                {
                                    if let Some(doc_id) = doc.id {
                                        state.show_confirm(
                                            &format!("Ta bort bilden \"{}\"?", doc.filename),
                                            ConfirmAction::DeleteResourceDocument(doc_id),
                                        );
                                        close_lightbox = true;
                                    }
                                }
                            });
                        });
                        ui.separator();

                        if let Some(ref tex) = self.lightbox_texture {
                            let available = ui.available_size();
                            let (tw, th) = (tex.size()[0] as f32, tex.size()[1] as f32);
                            let scale = (available.x / tw).min(available.y / th).min(1.0);
                            let size = egui::vec2(tw * scale, th * scale);
                            ui.image((tex.id(), size));
                        } else {
                            ui.label("Laddar bild...");
                        }
                    });

                if close_lightbox {
                    self.lightbox_image = None;
                    self.lightbox_texture = None;
                }
            }
        }
    }

    fn show_documents_tab(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut AppState,
        db: &Database,
        resource: &Resource,
        media_root: &str,
        type_dir: &str,
        resource_dir: &str,
    ) {
        let resource_id = resource.id.unwrap_or(0);

        ui.horizontal(|ui| {
            ui.label(RichText::new("Dokument").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("🔄").on_hover_text("Synkronisera från filsystem").clicked() {
                    let summary = self.sync_from_filesystem(db, resource_id, media_root, type_dir, resource_dir);
                    if summary.is_empty() {
                        state.show_status("Inga ändringar", crate::ui::StatusType::Info);
                    } else {
                        state.show_success(&format!("Synkronisering klar: {}", summary));
                    }
                }
                if ui.small_button(format!("{}", Icons::IMPORT)).on_hover_text("Ladda upp dokument").clicked() {
                    self.upload_file(ui.ctx(), state, db, resource_id, media_root, type_dir, resource_dir, false);
                }
            });
        });

        ui.add_space(8.0);

        let docs: Vec<&ResourceDocument> = self
            .documents_cache
            .iter()
            .filter(|d| !d.is_image())
            .collect();

        if docs.is_empty() {
            ui.label(RichText::new("Inga dokument uppladdade").color(Colors::TEXT_MUTED));
        } else {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let docs_cloned: Vec<ResourceDocument> = docs.iter().map(|d| (*d).clone()).collect();
                for doc in &docs_cloned {
                    ui.horizontal(|ui| {
                        ui.label(Icons::DOCUMENT);
                        ui.label(&doc.filename);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if let Some(doc_id) = doc.id {
                                if ui.small_button(Icons::DELETE)
                                    .on_hover_text("Ta bort dokument")
                                    .clicked()
                                {
                                    state.show_confirm(
                                        &format!("Ta bort dokumentet \"{}\"?", doc.filename),
                                        ConfirmAction::DeleteResourceDocument(doc_id),
                                    );
                                }

                                // Öppna med systemapplikation
                                let full_path = doc.full_path(media_root, type_dir, resource_dir);
                                if full_path.exists() {
                                    if ui.small_button(Icons::EXPORT)
                                        .on_hover_text("Öppna fil")
                                        .clicked()
                                    {
                                        open_in_file_explorer(&full_path);
                                    }
                                }
                            }
                        });
                    });
                    ui.separator();
                }
            });
        }
    }

    fn upload_file(
        &mut self,
        _ctx: &egui::Context,
        state: &mut AppState,
        db: &Database,
        resource_id: i64,
        media_root: &str,
        type_dir: &str,
        resource_dir: &str,
        images_only: bool,
    ) {
        let mut dialog = rfd::FileDialog::new();
        if images_only {
            dialog = dialog.add_filter("Bilder", &["jpg", "jpeg", "png", "gif", "webp", "bmp"]);
        }

        if let Some(paths) = dialog.pick_files() {
            let dest_subdir = if images_only { "bilder" } else { "dokument" };
            let dest_dir = std::path::Path::new(media_root)
                .join("resurser")
                .join(type_dir)
                .join(resource_dir)
                .join(dest_subdir);

            if let Err(e) = file_ops::ensure_directory(&dest_dir) {
                state.show_error(&format!("Kunde inte skapa katalog: {}", e));
                return;
            }

            for source_path in paths {
                let filename = source_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("fil")
                    .to_string();

                let unique_name = file_ops::unique_filename(&dest_dir, &filename);
                match file_ops::copy_file_to_directory(&source_path, &dest_dir, &unique_name) {
                    Ok(dest_path) => {
                        let ext = file_ops::get_file_extension(&dest_path)
                            .unwrap_or_default();
                        let file_size = file_ops::get_file_size(&dest_path).unwrap_or(0);
                        let relative_path = format!("{}/{}", dest_subdir, unique_name);

                        let doc = ResourceDocument {
                            id: None,
                            resource_id,
                            document_type_id: None,
                            filename: unique_name,
                            relative_path,
                            file_size: file_size as i64,
                            file_type: if ext.is_empty() { None } else { Some(ext) },
                            file_modified_at: None,
                            created_at: None,
                            updated_at: None,
                        };

                        match db.resources().create_document(&doc) {
                            Ok(_) => {}
                            Err(e) => state.show_error(&format!("Kunde inte spara dokument: {}", e)),
                        }
                    }
                    Err(e) => state.show_error(&format!("Kunde inte kopiera fil: {}", e)),
                }
            }

            // Uppdatera cachen
            self.refresh(db, resource_id);
            state.show_success("Fil(er) uppladdad(e)");
        }
    }

    /// Skannar resursens bilder/- och dokument/-kataloger och lägger till
    /// filer som saknas i databasen. Returnerar en sammanfattningssträng.
    fn sync_from_filesystem(
        &mut self,
        db: &Database,
        resource_id: i64,
        media_root: &str,
        type_dir: &str,
        resource_dir: &str,
    ) -> String {
        let base_dir = std::path::Path::new(media_root)
            .join("resurser")
            .join(type_dir)
            .join(resource_dir);

        let existing_paths: std::collections::HashSet<String> = self
            .documents_cache
            .iter()
            .map(|d| d.relative_path.clone())
            .collect();

        let mut added = 0usize;
        let mut errors = 0usize;

        for subdir in &["bilder", "dokument"] {
            let scan_dir = base_dir.join(subdir);
            let files = match file_ops::scan_directory_relative(&scan_dir) {
                Ok(f) => f,
                Err(_) => continue,
            };

            for (full_path, relative_in_subdir) in files {
                let relative_path = format!("{}/{}", subdir, relative_in_subdir);
                if existing_paths.contains(&relative_path) {
                    continue;
                }

                let filename = full_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("fil")
                    .to_string();
                let ext = file_ops::get_file_extension(&full_path).unwrap_or_default();
                let file_size = file_ops::get_file_size(&full_path).unwrap_or(0);

                let doc = ResourceDocument {
                    id: None,
                    resource_id,
                    document_type_id: None,
                    filename,
                    relative_path,
                    file_size: file_size as i64,
                    file_type: if ext.is_empty() { None } else { Some(ext) },
                    file_modified_at: None,
                    created_at: None,
                    updated_at: None,
                };

                match db.resources().create_document(&doc) {
                    Ok(_) => added += 1,
                    Err(_) => errors += 1,
                }
            }
        }

        self.refresh(db, resource_id);

        if added == 0 && errors == 0 {
            String::new()
        } else {
            let mut parts = Vec::new();
            if added > 0 {
                parts.push(format!("{} tillagd(a)", added));
            }
            if errors > 0 {
                parts.push(format!("{} fel", errors));
            }
            parts.join(", ")
        }
    }

    fn load_thumbnails(&mut self, ctx: &egui::Context, media_root: &str, type_dir: &str, resource_dir: &str) {
        let images: Vec<ResourceDocument> = self
            .documents_cache
            .iter()
            .filter(|d| d.is_image())
            .cloned()
            .collect();

        for doc in images {
            let doc_id = match doc.id {
                Some(id) => id,
                None => continue,
            };
            if self.thumbnails.contains_key(&doc_id) {
                continue;
            }

            let full_path = doc.full_path(media_root, type_dir, resource_dir);
            if !full_path.exists() {
                continue;
            }

            if let Ok(img) = image::open(&full_path) {
                let thumb = img.thumbnail(160, 160);
                let rgba = thumb.to_rgba8();
                let (w, h) = (rgba.width() as usize, rgba.height() as usize);
                let color_image = ColorImage::from_rgba_unmultiplied([w, h], &rgba);
                let texture = ctx.load_texture(
                    format!("resource_thumb_{}", doc_id),
                    color_image,
                    TextureOptions::LINEAR,
                );
                self.thumbnails.insert(doc_id, texture);
            }
        }
    }

    fn clear_address_form(&mut self) {
        self.new_addr_street.clear();
        self.new_addr_postal_code.clear();
        self.new_addr_city.clear();
        self.new_addr_country.clear();
    }

    fn refresh(&mut self, db: &Database, resource_id: i64) {
        if let Ok(Some((resource, resource_type))) = db.resources().find_with_type(resource_id) {
            self.resource_cache = Some(resource);
            self.type_cache = Some(resource_type);
        }
        if let Ok(addresses) = db.resources().get_addresses(resource_id) {
            self.addresses_cache = addresses;
        }
        if let Ok(docs) = db.resources().get_documents(resource_id) {
            self.documents_cache = docs;
        }
    }
}

/// Returnerar Some(s) om s inte är tom, annars None
fn non_empty(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() { None } else { Some(t.to_string()) }
}

fn open_in_file_explorer(path: &std::path::Path) {
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(path).spawn();
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(path).spawn();
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("explorer").arg(path).spawn();
}
