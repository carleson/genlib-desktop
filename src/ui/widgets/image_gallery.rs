//! Bildgalleri-widget för att visa thumbnails och fullstorlek

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use egui::{self, ColorImage, RichText, TextureHandle, TextureOptions};

use crate::db::Database;
use crate::models::Document;
use crate::ui::theme::{Colors, Icons};


/// Actions som galleriet returnerar till callern
#[derive(Default)]
pub struct GalleryAction {
    /// Användaren vill sätta en bild som profilbild (relative_path)
    pub set_profile_image: Option<String>,
    /// Användaren vill radera en bild (document_id, filename)
    pub delete_image: Option<(i64, String)>,
    /// Användaren klickade på en bild — öppna dokumentvisaren (document_id)
    pub open_document: Option<i64>,
}

/// Bildgalleri-widget
pub struct ImageGallery {
    /// Cachade thumbnail-texturer (document_id -> texture)
    thumbnails: HashMap<i64, TextureHandle>,
    /// Cachade fullstora texturer för lightbox (document_id -> texture)
    full_textures: HashMap<i64, TextureHandle>,
    /// Lista med bilder
    images: Vec<Document>,
    /// Vald bild för fullstorlek
    selected_image: Option<i64>,
    /// Behöver refresh
    needs_refresh: bool,
    /// Person ID
    person_id: Option<i64>,
    /// Person directory name (cachad)
    person_dir: Option<String>,
    /// Profilbild som valdes (returneras till caller)
    pending_profile_image: Option<String>,
    /// Bild som ska raderas (returneras till caller)
    pending_delete: Option<(i64, String)>,
    /// Dokument som ska öppnas i dokumentvisaren (returneras till caller)
    pending_open_document: Option<i64>,
    /// Zoom-nivå i lightbox (1.0 = originalstorlek relativt fönstret)
    zoom_level: f32,
}

impl Default for ImageGallery {
    fn default() -> Self {
        Self::new()
    }
}

/// Max storlek för thumbnails (2x för retina-skärpa)
const THUMBNAIL_MAX_SIZE: u32 = 160;

impl ImageGallery {
    pub fn new() -> Self {
        Self {
            thumbnails: HashMap::new(),
            full_textures: HashMap::new(),
            images: Vec::new(),
            selected_image: None,
            needs_refresh: true,
            person_id: None,
            person_dir: None,
            pending_profile_image: None,
            pending_delete: None,
            pending_open_document: None,
            zoom_level: 1.0,
        }
    }

    /// Visa galleriet och returnera actions
    pub fn show(&mut self, ui: &mut egui::Ui, db: &Database, person_id: i64, media_root: &Path) -> GalleryAction {
        // Samla pending actions
        let mut action = GalleryAction {
            set_profile_image: self.pending_profile_image.take(),
            delete_image: self.pending_delete.take(),
            open_document: None,
        };

        // Refresh om nödvändigt
        if self.needs_refresh || self.person_id != Some(person_id) {
            self.refresh(db, person_id);
        }

        if self.images.is_empty() {
            ui.label(RichText::new("Inga bilder").color(Colors::TEXT_MUTED));
            return action;
        }

        // Grid med thumbnails
        let thumbnail_size = 80.0;
        let spacing = 8.0;
        let available_width = ui.available_width();
        let columns = ((available_width + spacing) / (thumbnail_size + spacing)).floor() as usize;
        let columns = columns.max(1);

        // Samla data för rendering
        let image_data: Vec<_> = self.images.iter().map(|img| {
            (img.id.unwrap_or(0), img.relative_path.clone(), img.filename.clone())
        }).collect();

        // Ladda thumbnails
        for (image_id, relative_path, _) in &image_data {
            let image_path = self.build_image_path(media_root, relative_path);
            let _ = self.get_or_load_thumbnail(ui.ctx(), *image_id, &image_path);
        }

        egui::Grid::new("image_gallery_grid")
            .spacing([spacing, spacing])
            .show(ui, |ui| {
                for (i, (image_id, _relative_path, filename)) in image_data.iter().enumerate() {
                    if i > 0 && i % columns == 0 {
                        ui.end_row();
                    }

                    // Hämta thumbnail (redan laddad)
                    let texture = self.thumbnails.get(image_id);

                    // Visa thumbnail
                    let response = if let Some(tex) = texture {
                        let size = egui::vec2(thumbnail_size, thumbnail_size);
                        ui.add(egui::Image::new(tex).fit_to_exact_size(size).sense(egui::Sense::click()))
                    } else {
                        // Placeholder
                        let (rect, response) = ui.allocate_exact_size(
                            egui::vec2(thumbnail_size, thumbnail_size),
                            egui::Sense::click(),
                        );
                        ui.painter()
                            .rect_filled(rect, 4.0, ui.visuals().faint_bg_color);
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            Icons::IMAGE,
                            egui::FontId::default(),
                            Colors::TEXT_MUTED,
                        );
                        response
                    };

                    // Enkelklick → öppna lightbox
                    if response.clicked() {
                        self.selected_image = Some(*image_id);
                    }

                    // Hover-effekt
                    if response.hovered() {
                        response.on_hover_text(filename);
                    }
                }
            });

        // Visa lightbox om en bild är vald, annars återställ egui:s zoom-tangenter
        if let Some(image_id) = self.selected_image {
            self.show_lightbox(ui.ctx(), image_id, media_root);
        } else {
            ui.ctx().options_mut(|o| o.zoom_with_keyboard = true);
        }

        // Samla actions från lightbox
        if self.pending_profile_image.is_some() {
            action.set_profile_image = self.pending_profile_image.take();
        }
        if self.pending_delete.is_some() {
            action.delete_image = self.pending_delete.take();
        }
        if self.pending_open_document.is_some() {
            action.open_document = self.pending_open_document.take();
        }

        action
    }

    /// Visa lightbox med fullstorleksbild
    fn show_lightbox(&mut self, ctx: &egui::Context, image_id: i64, media_root: &Path) {
        // Stäng av egui:s inbyggda CTRL+/- zoom så vi kan använda det själva
        ctx.options_mut(|o| o.zoom_with_keyboard = false);

        // Läs zoom-input INNAN fönstret (ctx-nivå, ej konsumerat av egui)
        let scroll_zoom = ctx.input(|i| i.zoom_delta());
        let key_zoom_in = ctx.input_mut(|i| {
            i.consume_key(egui::Modifiers::COMMAND, egui::Key::Plus)
                || i.consume_key(egui::Modifiers::COMMAND, egui::Key::Equals)
        });
        let key_zoom_out = ctx.input_mut(|i| {
            i.consume_key(egui::Modifiers::COMMAND, egui::Key::Minus)
        });
        let key_zoom_reset = ctx.input_mut(|i| {
            i.consume_key(egui::Modifiers::COMMAND, egui::Key::Num0)
        });

        // Applicera zoom
        if key_zoom_reset {
            self.zoom_level = 1.0;
        } else {
            let mut factor = scroll_zoom; // zoom_delta är multiplikativ (1.0 = ingen ändring)
            if key_zoom_in { factor *= 1.25; }
            if key_zoom_out { factor /= 1.25; }
            if factor != 1.0 {
                self.zoom_level = (self.zoom_level * factor).clamp(0.1, 10.0);
            }
        }

        // Hitta bilden och extrahera data
        let image_data = self.images.iter()
            .enumerate()
            .find(|(_, i)| i.id == Some(image_id))
            .map(|(idx, img)| (idx, img.filename.clone(), img.relative_path.clone()));

        let (current_index, filename, relative_path) = match image_data {
            Some(data) => data,
            None => {
                self.selected_image = None;
                return;
            }
        };

        // Bygg full sökväg
        let image_path = self.build_image_path(media_root, &relative_path);
        let total_images = self.images.len();

        // Ladda fullstor textur för lightbox
        let _ = self.get_or_load_full_texture(ctx, image_id, &image_path);
        let texture = self.full_textures.get(&image_id).cloned();

        // Hämta prev/next IDs
        let prev_id = if current_index > 0 {
            self.images.get(current_index - 1).and_then(|i| i.id)
        } else {
            None
        };
        let next_id = if current_index < total_images - 1 {
            self.images.get(current_index + 1).and_then(|i| i.id)
        } else {
            None
        };

        let mut should_close = false;
        let mut new_selected: Option<i64> = None;
        let mut open_in_viewer = false;

        // Beräkna fönsterstorlek baserat på skärmstorlek
        let screen = ctx.screen_rect();
        let max_w = (screen.width() - 40.0).min(900.0);
        let max_h = (screen.height() - 40.0).min(750.0);

        egui::Window::new("Bild")
            .collapsible(false)
            .resizable(true)
            .default_size([max_w, max_h])
            .max_size([max_w, max_h])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                // === HEADER ===
                ui.horizontal(|ui| {
                    ui.label(&filename);

                    if (self.zoom_level - 1.0).abs() > 0.01 {
                        ui.label(
                            RichText::new(format!("{}%", (self.zoom_level * 100.0) as i32))
                                .small()
                                .color(Colors::TEXT_MUTED),
                        );
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Stäng").clicked() {
                            should_close = true;
                        }
                    });
                });

                ui.separator();

                // === BILDYTA ===
                let image_height = (ui.available_height() - 36.0).max(100.0);

                if let Some(tex) = texture {
                    let image_area_width = ui.available_width();
                    let tex_size = tex.size_vec2();
                    let base_scale = (image_area_width / tex_size.x)
                        .min(image_height / tex_size.y)
                        .min(1.0);
                    let display_size = tex_size * base_scale * self.zoom_level;

                    egui::ScrollArea::both()
                        .max_height(image_height)
                        .show(ui, |ui| {
                            ui.image((tex.id(), display_size));
                        });
                } else {
                    ui.allocate_space(egui::vec2(0.0, image_height));
                    ui.label("Kunde inte ladda bild");
                }

                // === FOOTER ===
                ui.horizontal(|ui| {
                    ui.add_enabled_ui(prev_id.is_some(), |ui| {
                        if ui.button(format!("{} Föregående", Icons::ARROW_LEFT)).clicked() {
                            new_selected = prev_id;
                        }
                    });

                    ui.label(format!("{} / {}", current_index + 1, total_images));

                    ui.add_enabled_ui(next_id.is_some(), |ui| {
                        if ui.button(format!("Nästa {}", Icons::ARROW_RIGHT)).clicked() {
                            new_selected = next_id;
                        }
                    });

                    ui.separator();

                    if ui.button(format!("{} Visa detaljer", Icons::DOCUMENT)).clicked() {
                        open_in_viewer = true;
                    }

                });
            });

        if should_close {
            self.selected_image = None;
            self.zoom_level = 1.0;
        } else if let Some(id) = new_selected {
            self.selected_image = Some(id);
            self.zoom_level = 1.0;
        }

        if open_in_viewer {
            self.pending_open_document = Some(image_id);
            self.selected_image = None;
        }
    }

    /// Bygg full sökväg till en bild
    fn build_image_path(&self, media_root: &Path, relative_path: &str) -> PathBuf {
        if let Some(ref person_dir) = self.person_dir {
            media_root.join("persons").join(person_dir).join(relative_path)
        } else {
            media_root.join(relative_path)
        }
    }

    /// Hämta eller ladda thumbnail (nedskalad)
    fn get_or_load_thumbnail(
        &mut self,
        ctx: &egui::Context,
        image_id: i64,
        path: &Path,
    ) -> Option<&TextureHandle> {
        if self.thumbnails.contains_key(&image_id) {
            return self.thumbnails.get(&image_id);
        }

        if let Some(texture) = Self::load_thumbnail(ctx, path) {
            self.thumbnails.insert(image_id, texture);
            return self.thumbnails.get(&image_id);
        }

        None
    }

    /// Hämta eller ladda fullstor textur (för lightbox)
    fn get_or_load_full_texture(
        &mut self,
        ctx: &egui::Context,
        image_id: i64,
        path: &Path,
    ) -> Option<&TextureHandle> {
        if self.full_textures.contains_key(&image_id) {
            return self.full_textures.get(&image_id);
        }

        if let Some(texture) = Self::load_image(ctx, path) {
            self.full_textures.insert(image_id, texture);
            return self.full_textures.get(&image_id);
        }

        None
    }

    /// Ladda nedskalad thumbnail
    fn load_thumbnail(ctx: &egui::Context, path: &Path) -> Option<TextureHandle> {
        let image_data = std::fs::read(path).ok()?;
        let image = image::load_from_memory(&image_data).ok()?;

        // Skala ner till thumbnail-storlek
        let thumb = image.thumbnail(THUMBNAIL_MAX_SIZE, THUMBNAIL_MAX_SIZE);
        let rgba = thumb.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let pixels = rgba.into_raw();

        let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);

        let texture = ctx.load_texture(
            format!("thumb_{}", path.display()),
            color_image,
            TextureOptions::LINEAR,
        );

        Some(texture)
    }

    /// Ladda fullstor bild
    fn load_image(ctx: &egui::Context, path: &Path) -> Option<TextureHandle> {
        let image_data = std::fs::read(path).ok()?;
        let image = image::load_from_memory(&image_data).ok()?;
        let rgba = image.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let pixels = rgba.into_raw();

        let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);

        let texture = ctx.load_texture(
            format!("image_{}", path.display()),
            color_image,
            TextureOptions::LINEAR,
        );

        Some(texture)
    }

    fn refresh(&mut self, db: &Database, person_id: i64) {
        self.person_id = Some(person_id);

        // Hämta person directory name
        self.person_dir = db.persons()
            .find_by_id(person_id)
            .ok()
            .flatten()
            .map(|p| p.directory_name);

        // Hämta alla bilder för personen
        let all_docs = db.documents().find_by_person(person_id).unwrap_or_default();
        self.images = all_docs.into_iter().filter(|d| d.is_image()).collect();

        self.needs_refresh = false;
    }

    /// Antal bilder
    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    pub fn mark_needs_refresh(&mut self) {
        self.needs_refresh = true;
        self.thumbnails.clear();
        self.full_textures.clear();
        self.person_dir = None;
    }
}
