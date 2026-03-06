//! Projekthantering — stöd för flera separata genealogiprojekt

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::utils::path::{get_default_projects_dir, get_projects_registry_path};

/// Ett genealogiprojekt med egen databas och mediakatalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub directory: PathBuf,
    pub is_default: bool,
    pub created_at: String,
}

impl Project {
    /// Skapa nytt projekt med genererat ID
    pub fn new(name: &str, description: &str, directory: PathBuf) -> Self {
        Self {
            id: generate_id(),
            name: name.to_string(),
            description: description.to_string(),
            directory,
            is_default: false,
            created_at: chrono::Local::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        }
    }

    /// Sökväg till projektets databas
    pub fn db_path(&self) -> PathBuf {
        self.directory.join("genlib.db")
    }
}

/// Generera ett enkelt unikt ID baserat på tidsstämpel
fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{:016x}{:08x}", d.as_secs(), d.subsec_nanos())
}

/// Register över alla projekt, sparas i projects.toml
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectRegistry {
    #[serde(default)]
    pub projects: Vec<Project>,
}

impl ProjectRegistry {
    /// Ladda projektregistret från disk (returnerar tomt register om filen saknas)
    pub fn load() -> Self {
        let path = get_projects_registry_path();
        if !path.exists() {
            return Self::default();
        }
        match std::fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content).unwrap_or_default(),
            Err(e) => {
                tracing::warn!("Kunde inte läsa projects.toml: {}", e);
                Self::default()
            }
        }
    }

    /// Spara projektregistret till disk
    pub fn save(&self) -> anyhow::Result<()> {
        let path = get_projects_registry_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Hämta default-projekt (det med is_default=true, annars det första)
    pub fn default_project(&self) -> Option<&Project> {
        self.projects
            .iter()
            .find(|p| p.is_default)
            .or_else(|| self.projects.first())
    }

    /// Hitta projekt med givet ID
    pub fn find_by_id(&self, id: &str) -> Option<&Project> {
        self.projects.iter().find(|p| p.id == id)
    }

    fn find_by_id_mut(&mut self, id: &str) -> Option<&mut Project> {
        self.projects.iter_mut().find(|p| p.id == id)
    }

    /// Lägg till projekt i registret
    pub fn add(&mut self, project: Project) {
        self.projects.push(project);
    }

    /// Ta bort projekt med givet ID från registret (filer på disk lämnas)
    pub fn remove(&mut self, id: &str) {
        self.projects.retain(|p| p.id != id);
    }

    /// Sätt ett projekt som default
    pub fn set_default(&mut self, id: &str) {
        for p in &mut self.projects {
            p.is_default = p.id == id;
        }
    }

    /// Byt namn på ett projekt
    pub fn rename(&mut self, id: &str, new_name: &str) {
        if let Some(p) = self.find_by_id_mut(id) {
            p.name = new_name.to_string();
        }
    }

    /// Returnera standardkatalogen för ett nytt projekt baserat på namn
    pub fn suggested_dir(name: &str) -> PathBuf {
        let safe_name = name
            .to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' {
                    c
                } else if c == ' ' || c == '-' {
                    '_'
                } else {
                    '_'
                }
            })
            .collect::<String>();
        get_default_projects_dir().join(safe_name)
    }
}

/// Åtgärder som projektväljar-vyn returnerar till anroparen
#[derive(Debug, Clone)]
pub enum ProjectAction {
    Open(String),
    Delete(String),
    SetDefault(String),
    Rename(String, String),
    CreateNew {
        name: String,
        description: String,
        directory: PathBuf,
    },
}
