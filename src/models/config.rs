use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

/// Format för personkatalogsnamn
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DirNameFormat {
    /// förnamn_efternamn_födelsedatum
    #[default]
    FirstnameFirst,
    /// efternamn_förnamn_födelsedatum
    SurnameFirst,
    /// födelsedatum_förnamn_efternamn
    DateFirst,
}

impl DirNameFormat {
    pub fn label(&self) -> &'static str {
        match self {
            Self::FirstnameFirst => "Förnamn först",
            Self::SurnameFirst => "Efternamn först",
            Self::DateFirst => "Datum först",
        }
    }

    pub fn example(&self) -> &'static str {
        match self {
            Self::FirstnameFirst => "svensson/gosta_anders_svensson_1921_12_07",
            Self::SurnameFirst => "svensson/svensson_gosta_anders_1921_12_07",
            Self::DateFirst => "svensson/1921_12_07_gosta_anders_svensson",
        }
    }

    pub fn all() -> &'static [DirNameFormat] {
        &[Self::FirstnameFirst, Self::SurnameFirst, Self::DateFirst]
    }

    pub fn from_db_str(s: &str) -> Self {
        match s {
            "surname_first" => Self::SurnameFirst,
            "date_first" => Self::DateFirst,
            _ => Self::FirstnameFirst,
        }
    }
}

impl fmt::Display for DirNameFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FirstnameFirst => write!(f, "firstname_first"),
            Self::SurnameFirst => write!(f, "surname_first"),
            Self::DateFirst => write!(f, "date_first"),
        }
    }
}

/// Systemkonfiguration (singleton, id=1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub id: i64,
    pub media_directory_path: PathBuf,
    pub backup_directory_path: PathBuf,
    pub dir_name_format: DirNameFormat,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl Default for SystemConfig {
    fn default() -> Self {
        // Använd directories crate för platform-specifika sökvägar
        let data_dir = directories::ProjectDirs::from("se", "genlib", "Genlib")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("./data"));

        Self {
            id: 1,
            media_directory_path: data_dir.join("media"),
            backup_directory_path: data_dir.join("backups"),
            dir_name_format: DirNameFormat::default(),
            created_at: None,
            updated_at: None,
        }
    }
}

impl SystemConfig {
    pub fn get_media_root(&self) -> &PathBuf {
        &self.media_directory_path
    }

    pub fn get_backup_root(&self) -> &PathBuf {
        &self.backup_directory_path
    }

    pub fn persons_directory(&self) -> PathBuf {
        self.media_directory_path.join("persons")
    }

    pub fn ensure_directories(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.media_directory_path)?;
        std::fs::create_dir_all(self.persons_directory())?;
        std::fs::create_dir_all(&self.backup_directory_path)?;
        Ok(())
    }
}

/// Katalogmall för personkataloger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub directories: String, // Newline-separerade
}

impl Template {
    pub fn new(name: String, directories: Vec<String>) -> Self {
        Self {
            id: None,
            name,
            description: None,
            directories: directories.join("\n"),
        }
    }

    pub fn get_directories_list(&self) -> Vec<&str> {
        self.directories
            .lines()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Fördefinierade mallar
    pub fn default_templates() -> Vec<Self> {
        vec![
            Self::new(
                "Standard".into(),
                vec![
                    "dokument".into(),
                    "bilder".into(),
                    "anteckningar".into(),
                    "media".into(),
                    "källor".into(),
                ],
            ),
            Self::new(
                "Utökad".into(),
                vec![
                    "dokument/personbevis".into(),
                    "dokument/folkräkning".into(),
                    "dokument/kyrkoböcker".into(),
                    "bilder/porträtt".into(),
                    "bilder/dokument".into(),
                    "anteckningar".into(),
                    "media/ljud".into(),
                    "media/video".into(),
                    "källor".into(),
                ],
            ),
            Self::new(
                "Minimal".into(),
                vec!["dokument".into(), "anteckningar".into()],
            ),
        ]
    }
}

// ============================================================
// Kortkommandon
// ============================================================

/// Bindbar åtgärd
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutAction {
    NavigateDashboard,
    NavigatePersonList,
    NavigateFamilyTree,
    NavigateChecklistSearch,
    NavigateSettings,
    NewPerson,
    FocusSearch,
    Backup,
    CloseModal,
    ToggleDarkMode,
}

impl ShortcutAction {
    pub const ALL: &'static [Self] = &[
        Self::NavigateDashboard,
        Self::NavigatePersonList,
        Self::NavigateFamilyTree,
        Self::NavigateChecklistSearch,
        Self::NavigateSettings,
        Self::NewPerson,
        Self::FocusSearch,
        Self::Backup,
        Self::CloseModal,
        Self::ToggleDarkMode,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::NavigateDashboard => "Dashboard",
            Self::NavigatePersonList => "Personer",
            Self::NavigateFamilyTree => "Släktträd",
            Self::NavigateChecklistSearch => "Uppgifter",
            Self::NavigateSettings => "Inställningar",
            Self::NewPerson => "Ny person",
            Self::FocusSearch => "Sök",
            Self::Backup => "Backup",
            Self::CloseModal => "Stäng dialog",
            Self::ToggleDarkMode => "Mörkt/ljust läge",
        }
    }

    fn to_key(&self) -> &'static str {
        match self {
            Self::NavigateDashboard => "navigate_dashboard",
            Self::NavigatePersonList => "navigate_person_list",
            Self::NavigateFamilyTree => "navigate_family_tree",
            Self::NavigateChecklistSearch => "navigate_checklist_search",
            Self::NavigateSettings => "navigate_settings",
            Self::NewPerson => "new_person",
            Self::FocusSearch => "focus_search",
            Self::Backup => "backup",
            Self::CloseModal => "close_modal",
            Self::ToggleDarkMode => "toggle_dark_mode",
        }
    }

    fn from_key(s: &str) -> Option<Self> {
        match s {
            "navigate_dashboard" => Some(Self::NavigateDashboard),
            "navigate_person_list" => Some(Self::NavigatePersonList),
            "navigate_family_tree" => Some(Self::NavigateFamilyTree),
            "navigate_checklist_search" => Some(Self::NavigateChecklistSearch),
            "navigate_settings" => Some(Self::NavigateSettings),
            "new_person" => Some(Self::NewPerson),
            "focus_search" => Some(Self::FocusSearch),
            "backup" => Some(Self::Backup),
            "close_modal" => Some(Self::CloseModal),
            "toggle_dark_mode" => Some(Self::ToggleDarkMode),
            _ => None,
        }
    }
}

/// Modifierare för kortkommando
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShortcutModifiers {
    /// Ctrl (Linux/Windows) eller Cmd (macOS)
    pub ctrl_or_cmd: bool,
    pub shift: bool,
    pub alt: bool,
}

/// Ett kortkommando — tangentkombination
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardShortcut {
    pub key: egui::Key,
    pub modifiers: ShortcutModifiers,
}

impl KeyboardShortcut {
    pub fn new(key: egui::Key, ctrl_or_cmd: bool, shift: bool, alt: bool) -> Self {
        Self {
            key,
            modifiers: ShortcutModifiers { ctrl_or_cmd, shift, alt },
        }
    }

    /// Visningstext, t.ex. "Ctrl+N" eller "Cmd+N"
    pub fn display(&self) -> String {
        let mut parts = Vec::new();
        if self.modifiers.ctrl_or_cmd {
            if cfg!(target_os = "macos") {
                parts.push("Cmd");
            } else {
                parts.push("Ctrl");
            }
        }
        if self.modifiers.alt {
            parts.push("Alt");
        }
        if self.modifiers.shift {
            parts.push("Shift");
        }
        parts.push(self.key.name());
        parts.join("+")
    }

    /// Serialisera till sträng (alltid "Ctrl" oavsett plattform)
    fn to_string_canonical(&self) -> String {
        let mut parts = Vec::new();
        if self.modifiers.ctrl_or_cmd {
            parts.push("Ctrl".to_string());
        }
        if self.modifiers.alt {
            parts.push("Alt".to_string());
        }
        if self.modifiers.shift {
            parts.push("Shift".to_string());
        }
        parts.push(self.key.name().to_string());
        parts.join("+")
    }

    /// Deserialisera från sträng
    fn from_str(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
        if parts.is_empty() {
            return None;
        }
        let mut ctrl_or_cmd = false;
        let mut shift = false;
        let mut alt = false;

        for &part in &parts[..parts.len() - 1] {
            match part {
                "Ctrl" | "Cmd" => ctrl_or_cmd = true,
                "Shift" => shift = true,
                "Alt" => alt = true,
                _ => return None,
            }
        }

        let key_name = parts.last()?;
        let key = egui::Key::from_name(key_name)?;

        Some(Self::new(key, ctrl_or_cmd, shift, alt))
    }

    /// Matchar denna genväg mot egui-modifierare och tangent?
    pub fn matches(&self, key: egui::Key, modifiers: &egui::Modifiers) -> bool {
        self.key == key
            && self.modifiers.ctrl_or_cmd == modifiers.command
            && self.modifiers.shift == modifiers.shift
            && self.modifiers.alt == modifiers.alt
    }
}

pub type ShortcutMap = HashMap<ShortcutAction, KeyboardShortcut>;

/// Standard-genvägar
pub fn default_shortcuts() -> ShortcutMap {
    let mut m = ShortcutMap::new();
    m.insert(ShortcutAction::NavigateDashboard, KeyboardShortcut::new(egui::Key::Num1, true, false, false));
    m.insert(ShortcutAction::NavigatePersonList, KeyboardShortcut::new(egui::Key::Num2, true, false, false));
    m.insert(ShortcutAction::NavigateFamilyTree, KeyboardShortcut::new(egui::Key::Num3, true, false, false));
    m.insert(ShortcutAction::NavigateChecklistSearch, KeyboardShortcut::new(egui::Key::Num4, true, false, false));
    m.insert(ShortcutAction::NavigateSettings, KeyboardShortcut::new(egui::Key::Comma, true, false, false));
    m.insert(ShortcutAction::NewPerson, KeyboardShortcut::new(egui::Key::N, true, false, false));
    m.insert(ShortcutAction::FocusSearch, KeyboardShortcut::new(egui::Key::F, true, false, false));
    m.insert(ShortcutAction::Backup, KeyboardShortcut::new(egui::Key::B, true, false, false));
    m.insert(ShortcutAction::CloseModal, KeyboardShortcut::new(egui::Key::Escape, false, false, false));
    m.insert(ShortcutAction::ToggleDarkMode, KeyboardShortcut::new(egui::Key::D, true, false, false));
    m
}

/// Applikationstillstånd som inte sparas i databas
#[derive(Debug, Clone)]
pub struct AppSettings {
    pub dark_mode: bool,
    pub window_width: f32,
    pub window_height: f32,
    pub sidebar_width: f32,
    pub show_welcome: bool,
    pub shortcuts: ShortcutMap,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            dark_mode: false,
            window_width: 1280.0,
            window_height: 800.0,
            sidebar_width: 200.0,
            show_welcome: true,
            shortcuts: default_shortcuts(),
        }
    }
}

impl AppSettings {
    pub fn load() -> Self {
        // Försök ladda från config-fil
        let config_path = directories::ProjectDirs::from("se", "genlib", "Genlib")
            .map(|dirs| dirs.config_dir().join("settings.toml"))
            .unwrap_or_else(|| PathBuf::from("settings.toml"));

        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(settings) = toml::from_str(&content) {
                return settings;
            }
        }

        Self::default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let config_dir = directories::ProjectDirs::from("se", "genlib", "Genlib")
            .map(|dirs| dirs.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        std::fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("settings.toml");
        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;

        Ok(())
    }
}

impl Serialize for AppSettings {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AppSettings", 6)?;
        state.serialize_field("dark_mode", &self.dark_mode)?;
        state.serialize_field("window_width", &self.window_width)?;
        state.serialize_field("window_height", &self.window_height)?;
        state.serialize_field("sidebar_width", &self.sidebar_width)?;
        state.serialize_field("show_welcome", &self.show_welcome)?;

        // Serialisera genvägar som HashMap<String, String>
        let shortcuts_map: HashMap<String, String> = self
            .shortcuts
            .iter()
            .map(|(action, shortcut)| {
                (action.to_key().to_string(), shortcut.to_string_canonical())
            })
            .collect();
        state.serialize_field("shortcuts", &shortcuts_map)?;

        state.end()
    }
}

impl<'de> Deserialize<'de> for AppSettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct AppSettingsHelper {
            dark_mode: Option<bool>,
            window_width: Option<f32>,
            window_height: Option<f32>,
            sidebar_width: Option<f32>,
            show_welcome: Option<bool>,
            shortcuts: Option<HashMap<String, String>>,
        }

        let helper = AppSettingsHelper::deserialize(deserializer)?;

        // Bygg ShortcutMap: börja med defaults, överskrid med sparade
        let mut shortcuts = default_shortcuts();
        if let Some(saved) = helper.shortcuts {
            for (key, value) in saved {
                if let Some(action) = ShortcutAction::from_key(&key) {
                    if let Some(shortcut) = KeyboardShortcut::from_str(&value) {
                        shortcuts.insert(action, shortcut);
                    }
                }
            }
        }

        Ok(AppSettings {
            dark_mode: helper.dark_mode.unwrap_or(false),
            window_width: helper.window_width.unwrap_or(1280.0),
            window_height: helper.window_height.unwrap_or(800.0),
            sidebar_width: helper.sidebar_width.unwrap_or(200.0),
            show_welcome: helper.show_welcome.unwrap_or(true),
            shortcuts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_serialization_roundtrip() {
        let shortcut = KeyboardShortcut::new(egui::Key::N, true, false, false);
        let s = shortcut.to_string_canonical();
        assert_eq!(s, "Ctrl+N");
        let parsed = KeyboardShortcut::from_str(&s).unwrap();
        assert_eq!(parsed, shortcut);
    }

    #[test]
    fn test_shortcut_with_modifiers() {
        let shortcut = KeyboardShortcut::new(egui::Key::S, true, true, true);
        let s = shortcut.to_string_canonical();
        assert_eq!(s, "Ctrl+Alt+Shift+S");
        let parsed = KeyboardShortcut::from_str(&s).unwrap();
        assert_eq!(parsed, shortcut);
    }

    #[test]
    fn test_shortcut_escape() {
        let shortcut = KeyboardShortcut::new(egui::Key::Escape, false, false, false);
        let s = shortcut.to_string_canonical();
        assert_eq!(s, "Escape");
        let parsed = KeyboardShortcut::from_str(&s).unwrap();
        assert_eq!(parsed, shortcut);
    }

    #[test]
    fn test_shortcut_comma() {
        let shortcut = KeyboardShortcut::new(egui::Key::Comma, true, false, false);
        let s = shortcut.to_string_canonical();
        assert_eq!(s, "Ctrl+Comma");
        let parsed = KeyboardShortcut::from_str(&s).unwrap();
        assert_eq!(parsed, shortcut);
    }

    #[test]
    fn test_default_shortcuts_covers_all_actions() {
        let defaults = default_shortcuts();
        for action in ShortcutAction::ALL {
            assert!(
                defaults.contains_key(action),
                "Missing default for {:?}",
                action
            );
        }
    }

    #[test]
    fn test_app_settings_toml_roundtrip() {
        let settings = AppSettings::default();
        let toml_str = toml::to_string_pretty(&settings).unwrap();
        let loaded: AppSettings = toml::from_str(&toml_str).unwrap();
        assert_eq!(loaded.dark_mode, settings.dark_mode);
        assert_eq!(loaded.shortcuts.len(), settings.shortcuts.len());
        for (action, shortcut) in &settings.shortcuts {
            assert_eq!(loaded.shortcuts.get(action), Some(shortcut));
        }
    }

    #[test]
    fn test_app_settings_backwards_compat() {
        // Gammal TOML utan [shortcuts] — ska ge defaults
        let old_toml = r#"
dark_mode = true
window_width = 1024.0
window_height = 768.0
sidebar_width = 180.0
show_welcome = false
"#;
        let loaded: AppSettings = toml::from_str(old_toml).unwrap();
        assert!(loaded.dark_mode);
        assert_eq!(loaded.window_width, 1024.0);
        assert_eq!(loaded.shortcuts.len(), ShortcutAction::ALL.len());
    }

    #[test]
    fn test_shortcut_action_key_roundtrip() {
        for action in ShortcutAction::ALL {
            let key = action.to_key();
            let parsed = ShortcutAction::from_key(key);
            assert_eq!(parsed, Some(*action), "Roundtrip failed for {:?}", action);
        }
    }
}
