//! Splashscreen — visas vid appstart

use eframe::egui::{self, TextureHandle};
use std::time::Instant;

use crate::ui::View;

const SPLASH_DURATION_SECS: f32 = 2.5;
const ICON_SIZE: f32 = 128.0;

pub struct SplashScreenView {
    start_time: Instant,
    icon_texture: Option<TextureHandle>,
    next_view: View,
}

impl SplashScreenView {
    pub fn new(next_view: View) -> Self {
        Self {
            start_time: Instant::now(),
            icon_texture: None,
            next_view,
        }
    }

    /// Returnerar true när splash är klar och vi ska navigera vidare
    pub fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) -> bool {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let progress = (elapsed / SPLASH_DURATION_SECS).min(1.0);

        // Fade-in under första 0.5s, full synlighet resten
        let alpha = if elapsed < 0.5 {
            (elapsed / 0.5).min(1.0)
        } else if elapsed > SPLASH_DURATION_SECS - 0.4 {
            // Fade-out sista 0.4s
            ((SPLASH_DURATION_SECS - elapsed) / 0.4).max(0.0)
        } else {
            1.0
        };

        let alpha_byte = (alpha * 255.0) as u8;

        // Ladda ikon-textur (en gång)
        let texture = self.icon_texture.get_or_insert_with(|| {
            let png_bytes = include_bytes!("../../../resources/icons/genlib-256.png");
            let img = image::load_from_memory(png_bytes).expect("Kunde inte ladda splashikon");
            let rgba = img.to_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            let pixels = rgba.into_raw();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
            ctx.load_texture("splash-icon", color_image, egui::TextureOptions::LINEAR)
        });

        let available = ui.available_size();

        ui.vertical_centered(|ui| {
            // Centrera vertikalt
            let content_height = ICON_SIZE + 80.0; // ikon + text
            let top_padding = (available.y - content_height) / 2.0;
            ui.add_space(top_padding.max(20.0));

            // Ikon
            let tint = egui::Color32::from_white_alpha(alpha_byte);
            let sized = egui::load::SizedTexture::new(texture.id(), egui::vec2(ICON_SIZE, ICON_SIZE));
            ui.add(
                egui::Image::new(sized)
                    .tint(tint),
            );

            ui.add_space(16.0);

            // Appnamn
            ui.label(
                egui::RichText::new("Genlib")
                    .size(36.0)
                    .strong()
                    .color(egui::Color32::from_rgba_unmultiplied(
                        200, 200, 200, alpha_byte,
                    )),
            );

            ui.add_space(4.0);

            // Underrubrik
            ui.label(
                egui::RichText::new("Släktforskning")
                    .size(16.0)
                    .color(egui::Color32::from_rgba_unmultiplied(
                        140, 140, 140, alpha_byte,
                    )),
            );

            ui.add_space(24.0);

            // Version
            ui.label(
                egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                    .size(12.0)
                    .color(egui::Color32::from_rgba_unmultiplied(
                        100, 100, 100, alpha_byte,
                    )),
            );
        });

        // Fortsätt animera
        ctx.request_repaint();

        progress >= 1.0
    }

    /// Vilken vy vi navigerar till efter splash
    pub fn next_view(&self) -> View {
        self.next_view
    }
}
