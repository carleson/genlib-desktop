use serde::{Deserialize, Serialize};
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
            Self::FirstnameFirst => "gosta_anders_svensson_1921-12-07",
            Self::SurnameFirst => "svensson_gosta_anders_1921-12-07",
            Self::DateFirst => "1921-12-07_gosta_anders_svensson",
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

/// Applikationstillstånd som inte sparas i databas
#[derive(Debug, Clone, Default)]
pub struct AppSettings {
    pub dark_mode: bool,
    pub window_width: f32,
    pub window_height: f32,
    pub sidebar_width: f32,
    pub show_welcome: bool,
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
        let mut state = serializer.serialize_struct("AppSettings", 5)?;
        state.serialize_field("dark_mode", &self.dark_mode)?;
        state.serialize_field("window_width", &self.window_width)?;
        state.serialize_field("window_height", &self.window_height)?;
        state.serialize_field("sidebar_width", &self.sidebar_width)?;
        state.serialize_field("show_welcome", &self.show_welcome)?;
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
        }

        let helper = AppSettingsHelper::deserialize(deserializer)?;
        Ok(AppSettings {
            dark_mode: helper.dark_mode.unwrap_or(false),
            window_width: helper.window_width.unwrap_or(1280.0),
            window_height: helper.window_height.unwrap_or(800.0),
            sidebar_width: helper.sidebar_width.unwrap_or(200.0),
            show_welcome: helper.show_welcome.unwrap_or(true),
        })
    }
}
