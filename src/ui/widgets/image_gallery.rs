//! Bildgalleri-widget för att visa thumbnails och fullstorlek

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use egui::{self, ColorImage, RichText, TextureHandle, TextureOptions};

use crate::db::Database;
use crate::models::Document;
use crate::ui::theme::{Colors, Icons};
use crate::utils::exif::ExifData;

/// Bildgalleri-widget
pub struct ImageGallery {
    /// Cachade texturer (document_id -> texture)
    textures: HashMap<i64, TextureHandle>,
    /// Cachade EXIF-data (document_id -> exif data)
    exif_cache: HashMap<i64, Option<ExifData>>,
    /// Lista med bilder
    images: Vec<Document>,
    /// Vald bild för fullstorlek
    selected_image: Option<i64>,
    /// Visa EXIF-info i lightbox
    show_exif_info: bool,
    /// Behöver refresh
    needs_refresh: bool,
    /// Person ID
    person_id: Option<i64>,
    /// Person directory name (cachad)
    person_dir: Option<String>,
    /// Profilbild som valdes (returneras till caller)
    pending_profile_image: Option<String>,
}

impl Default for ImageGallery {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageGallery {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            exif_cache: HashMap::new(),
            images: Vec::new(),
            selected_image: None,
            show_exif_info: false,
            needs_refresh: true,
            person_id: None,
            person_dir: None,
            pending_profile_image: None,
        }
    }

    /// Visa galleriet
    /// Returnerar Some(path) om användaren valde att sätta en bild som profilbild
    pub fn show(&mut self, ui: &mut egui::Ui, db: &Database, person_id: i64, media_root: &Path) -> Option<String> {
        // Kolla om vi har en pending profilbild att returnera
        let result = self.pending_profile_image.take();

        // Refresh om nödvändigt
        if self.needs_refresh || self.person_id != Some(person_id) {
            self.refresh(db, person_id);
        }

        if self.images.is_empty() {
            ui.label(RichText::new("Inga bilder").color(Colors::TEXT_MUTED));
            return result;
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

        // Ladda texturer först
        for (image_id, relative_path, _) in &image_data {
            // Bygg full sökväg: media_root/persons/person_dir/relative_path
            let image_path = if let Some(ref person_dir) = self.person_dir {
                media_root.join("persons").join(person_dir).join(relative_path)
            } else {
                media_root.join(relative_path)
            };
            let _ = self.get_or_load_texture(ui.ctx(), *image_id, &image_path);
        }

        let mut clicked_image: Option<i64> = None;

        egui::Grid::new("image_gallery_grid")
            .spacing([spacing, spacing])
            .show(ui, |ui| {
                for (i, (image_id, _relative_path, filename)) in image_data.iter().enumerate() {
                    if i > 0 && i % columns == 0 {
                        ui.end_row();
                    }

                    // Hämta textur (redan laddad)
                    let texture = self.textures.get(image_id);

                    // Visa thumbnail
                    let response = if let Some(tex) = texture {
                        let size = egui::vec2(thumbnail_size, thumbnail_size);
                        ui.add(egui::Image::new(tex).fit_to_exact_size(size))
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

                    // Klick för att visa fullstorlek
                    if response.clicked() {
                        clicked_image = Some(*image_id);
                    }

                    // Hover-effekt
                    if response.hovered() {
                        response.on_hover_text(filename);
                    }
                }
            });

        if let Some(id) = clicked_image {
            self.selected_image = Some(id);
        }

        // Lightbox för fullstorlek
        if let Some(selected_id) = self.selected_image {
            self.show_lightbox(ui.ctx(), selected_id, media_root);
        }

        result
    }

    /// Visa lightbox med fullstorleksbild
    fn show_lightbox(&mut self, ctx: &egui::Context, image_id: i64, media_root: &Path) {
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

        // Bygg full sökväg: media_root/persons/person_dir/relative_path
        let image_path = if let Some(ref person_dir) = self.person_dir {
            media_root.join("persons").join(person_dir).join(&relative_path)
        } else {
            media_root.join(&relative_path)
        };
        let total_images = self.images.len();

        // Ladda textur
        let _ = self.get_or_load_texture(ctx, image_id, &image_path);
        let texture = self.textures.get(&image_id).cloned();

        // Ladda EXIF-data om ej cachad
        self.load_exif_data(image_id, &image_path);
        let exif_data = self.exif_cache.get(&image_id).cloned().flatten();

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
        let mut set_as_profile = false;

        egui::Window::new("Bild")
            .collapsible(false)
            .resizable(true)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(&filename);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Stäng").clicked() {
                            should_close = true;
                        }
                    });
                });

                ui.separator();

                // Visa bild i större storlek
                if let Some(tex) = texture {
                    let max_size = egui::vec2(800.0, 600.0);
                    ui.add(egui::Image::new(&tex).max_size(max_size));
                } else {
                    ui.label("Kunde inte ladda bild");
                }

                // EXIF-info (collapsible)
                if exif_data.is_some() {
                    ui.add_space(4.0);
                    let exif_header = if self.show_exif_info {
                        format!("{} Bildinfo ▼", Icons::CAMERA)
                    } else {
                        format!("{} Bildinfo ▶", Icons::CAMERA)
                    };
                    if ui.selectable_label(self.show_exif_info, exif_header).clicked() {
                        self.show_exif_info = !self.show_exif_info;
                    }

                    if self.show_exif_info {
                        if let Some(ref exif) = exif_data {
                            egui::Frame::none()
                                .fill(ui.visuals().faint_bg_color)
                                .rounding(4.0)
                                .inner_margin(8.0)
                                .show(ui, |ui| {
                                    egui::Grid::new("exif_grid")
                                        .num_columns(2)
                                        .spacing([16.0, 4.0])
                                        .show(ui, |ui| {
                                            // Datum
                                            if let Some(dt) = exif.date_taken {
                                                ui.label(RichText::new("Datum:").color(Colors::TEXT_SECONDARY));
                                                ui.label(dt.format("%Y-%m-%d %H:%M").to_string());
                                                ui.end_row();
                                            }

                                            // Kamera
                                            if let Some(cam) = exif.camera_info() {
                                                ui.label(RichText::new("Kamera:").color(Colors::TEXT_SECONDARY));
                                                ui.label(cam);
                                                ui.end_row();
                                            }

                                            // Exponering
                                            if let Some(exp) = exif.exposure_info() {
                                                ui.label(RichText::new("Exponering:").color(Colors::TEXT_SECONDARY));
                                                ui.label(exp);
                                                ui.end_row();
                                            }

                                            // Bildstorlek
                                            if let Some(dim) = exif.dimensions_string() {
                                                ui.label(RichText::new("Storlek:").color(Colors::TEXT_SECONDARY));
                                                ui.label(format!("{} px", dim));
                                                ui.end_row();
                                            }

                                            // GPS
                                            if let Some(gps) = exif.gps_string() {
                                                ui.label(RichText::new(format!("{} GPS:", Icons::LOCATION)).color(Colors::TEXT_SECONDARY));
                                                ui.label(gps);
                                                ui.end_row();
                                            }

                                            // Fotograf
                                            if let Some(ref artist) = exif.artist {
                                                ui.label(RichText::new("Fotograf:").color(Colors::TEXT_SECONDARY));
                                                ui.label(artist);
                                                ui.end_row();
                                            }

                                            // Copyright
                                            if let Some(ref copyright) = exif.copyright {
                                                ui.label(RichText::new("Copyright:").color(Colors::TEXT_SECONDARY));
                                                ui.label(copyright);
                                                ui.end_row();
                                            }
                                        });
                                });
                        }
                    }
                }

                ui.add_space(8.0);

                // Navigation och profilbild-knapp
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

                    // Sätt som profilbild-knapp
                    if ui.button(format!("{} Sätt som profilbild", Icons::PERSON)).clicked() {
                        set_as_profile = true;
                    }
                });
            });

        if should_close {
            self.selected_image = None;
        } else if let Some(id) = new_selected {
            self.selected_image = Some(id);
        }

        // Om användaren valde att sätta som profilbild
        if set_as_profile {
            self.pending_profile_image = Some(relative_path);
            self.selected_image = None; // Stäng lightbox
        }
    }

    /// Ladda EXIF-data för en bild
    fn load_exif_data(&mut self, image_id: i64, path: &PathBuf) {
        if self.exif_cache.contains_key(&image_id) {
            return;
        }

        let exif = ExifData::from_file(path).ok().flatten();
        self.exif_cache.insert(image_id, exif);
    }

    fn get_or_load_texture(
        &mut self,
        ctx: &egui::Context,
        image_id: i64,
        path: &Path,
    ) -> Option<&TextureHandle> {
        // Returnera cached om finns
        if self.textures.contains_key(&image_id) {
            return self.textures.get(&image_id);
        }

        // Försök ladda bilden
        if let Some(texture) = Self::load_image(ctx, path) {
            self.textures.insert(image_id, texture);
            return self.textures.get(&image_id);
        }

        None
    }

    fn load_image(ctx: &egui::Context, path: &Path) -> Option<TextureHandle> {
        // Läs fil
        let image_data = std::fs::read(path).ok()?;

        // Dekoda bild
        let image = image::load_from_memory(&image_data).ok()?;
        let rgba = image.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let pixels = rgba.into_raw();

        // Skapa färgbild
        let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);

        // Skapa textur
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
        self.textures.clear();
        self.exif_cache.clear();
        self.person_dir = None;
    }

    /// Sätt vald bild som profilbild
    pub fn get_selected_image_path(&self) -> Option<String> {
        self.selected_image
            .and_then(|id| self.images.iter().find(|i| i.id == Some(id)))
            .map(|i| i.relative_path.clone())
    }
}
