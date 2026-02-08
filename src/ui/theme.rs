use egui::{Color32, FontFamily, FontId, TextStyle, Visuals};

/// Konfigurera applikationens utseende
pub fn configure_style(ctx: &egui::Context, dark_mode: bool) {
    let mut style = (*ctx.style()).clone();

    // Typsnitt
    style.text_styles = [
        (TextStyle::Heading, FontId::new(24.0, FontFamily::Proportional)),
        (TextStyle::Name("heading2".into()), FontId::new(20.0, FontFamily::Proportional)),
        (TextStyle::Name("heading3".into()), FontId::new(16.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(13.0, FontFamily::Monospace)),
        (TextStyle::Button, FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Small, FontId::new(12.0, FontFamily::Proportional)),
    ]
    .into();

    // Spacing
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(12.0);

    // Visuella stilar
    if dark_mode {
        style.visuals = dark_visuals();
    } else {
        style.visuals = light_visuals();
    }

    ctx.set_style(style);
}

fn dark_visuals() -> Visuals {
    let mut visuals = Visuals::dark();

    // Bakgrundsfärger
    visuals.panel_fill = Color32::from_rgb(30, 30, 35);
    visuals.window_fill = Color32::from_rgb(40, 40, 45);
    visuals.extreme_bg_color = Color32::from_rgb(20, 20, 25);

    // Widget-färger
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(45, 45, 50);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(50, 50, 55);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(60, 60, 70);
    visuals.widgets.active.bg_fill = Color32::from_rgb(70, 70, 85);

    // Accentfärg (blå)
    visuals.selection.bg_fill = Color32::from_rgb(60, 100, 180);
    visuals.hyperlink_color = Color32::from_rgb(100, 150, 255);

    visuals
}

fn light_visuals() -> Visuals {
    let mut visuals = Visuals::light();

    // Bakgrundsfärger
    visuals.panel_fill = Color32::from_rgb(248, 248, 250);
    visuals.window_fill = Color32::from_rgb(255, 255, 255);
    visuals.extreme_bg_color = Color32::from_rgb(240, 240, 242);

    // Widget-färger
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(235, 235, 240);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(230, 230, 235);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(220, 220, 230);
    visuals.widgets.active.bg_fill = Color32::from_rgb(200, 200, 220);

    // Accentfärg (blå)
    visuals.selection.bg_fill = Color32::from_rgb(180, 210, 255);
    visuals.hyperlink_color = Color32::from_rgb(0, 100, 200);

    visuals
}

/// Färgpalett för applikationen
pub struct Colors;

impl Colors {
    // Primär
    pub const PRIMARY: Color32 = Color32::from_rgb(59, 130, 246);
    pub const PRIMARY_HOVER: Color32 = Color32::from_rgb(37, 99, 235);

    // Framgång
    pub const SUCCESS: Color32 = Color32::from_rgb(34, 197, 94);
    pub const SUCCESS_BG: Color32 = Color32::from_rgb(220, 252, 231);

    // Varning
    pub const WARNING: Color32 = Color32::from_rgb(234, 179, 8);
    pub const WARNING_BG: Color32 = Color32::from_rgb(254, 249, 195);

    // Fel
    pub const ERROR: Color32 = Color32::from_rgb(239, 68, 68);
    pub const ERROR_BG: Color32 = Color32::from_rgb(254, 226, 226);

    // Info
    pub const INFO: Color32 = Color32::from_rgb(59, 130, 246);
    pub const INFO_BG: Color32 = Color32::from_rgb(219, 234, 254);

    // Text
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(17, 24, 39);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(107, 114, 128);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(156, 163, 175);

    // Relationstyper
    pub const PARENT: Color32 = Color32::from_rgb(147, 51, 234);
    pub const CHILD: Color32 = Color32::from_rgb(34, 197, 94);
    pub const SPOUSE: Color32 = Color32::from_rgb(239, 68, 68);
    pub const SIBLING: Color32 = Color32::from_rgb(59, 130, 246);
}

/// Ikoner (Unicode)
pub struct Icons;

impl Icons {
    pub const PERSON: &'static str = "👤";
    pub const PEOPLE: &'static str = "👥";
    pub const FAMILY: &'static str = "👨‍👩‍👧‍👦";
    pub const TREE: &'static str = "🌳";
    pub const DOCUMENT: &'static str = "📄";
    pub const FOLDER: &'static str = "📁";
    pub const IMAGE: &'static str = "🖼";
    pub const SEARCH: &'static str = "🔍";
    pub const SETTINGS: &'static str = "⚙";
    pub const ADD: &'static str = "➕";
    pub const EDIT: &'static str = "✏";
    pub const DELETE: &'static str = "🗑";
    pub const SAVE: &'static str = "💾";
    pub const BOOKMARK: &'static str = "⭐";
    pub const BOOKMARK_EMPTY: &'static str = "☆";
    pub const CHECK: &'static str = "✓";
    pub const CROSS: &'static str = "✗";
    pub const ARROW_LEFT: &'static str = "←";
    pub const ARROW_RIGHT: &'static str = "→";
    pub const CALENDAR: &'static str = "📅";
    pub const NOTE: &'static str = "📝";
    pub const DASHBOARD: &'static str = "📊";
    pub const BACKUP: &'static str = "💾";
    pub const IMPORT: &'static str = "📥";
    pub const EXPORT: &'static str = "📤";
    pub const DOWNLOAD: &'static str = "⬇";
    pub const LINK: &'static str = "🔗";
    pub const HEART: &'static str = "❤";
    pub const FILTER: &'static str = "⏷";
    pub const CAMERA: &'static str = "📷";
    pub const LOCATION: &'static str = "📍";
}
