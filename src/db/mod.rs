pub mod schema;
pub mod migrations;
pub mod person_repo;
pub mod document_repo;
pub mod relationship_repo;
pub mod config_repo;
pub mod checklist_repo;

use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub use person_repo::{PersonRepository, SearchFilter};
pub use document_repo::DocumentRepository;
pub use relationship_repo::RelationshipRepository;
pub use config_repo::ConfigRepository;
pub use checklist_repo::ChecklistRepository;

/// Huvuddatabas-wrapper med thread-safe access
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Öppna eller skapa databas
    pub fn open(path: &Path) -> Result<Self> {
        // Skapa katalog om den inte finns
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;

        // Konfigurera SQLite
        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;
            PRAGMA busy_timeout = 5000;
            "
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Öppna in-memory databas (för tester)
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.migrate()?;
        Ok(db)
    }

    /// Kör databasmigrationer
    pub fn migrate(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        migrations::run_migrations(&conn)
    }

    /// Hämta person-repository
    pub fn persons(&self) -> PersonRepository {
        PersonRepository::new(Arc::clone(&self.conn))
    }

    /// Hämta dokument-repository
    pub fn documents(&self) -> DocumentRepository {
        DocumentRepository::new(Arc::clone(&self.conn))
    }

    /// Hämta relations-repository
    pub fn relationships(&self) -> RelationshipRepository {
        RelationshipRepository::new(Arc::clone(&self.conn))
    }

    /// Hämta config-repository
    pub fn config(&self) -> ConfigRepository {
        ConfigRepository::new(Arc::clone(&self.conn))
    }

    /// Hämta checklist-repository
    pub fn checklists(&self) -> ChecklistRepository {
        ChecklistRepository::new(Arc::clone(&self.conn))
    }

    /// Direkt tillgång till connection (för avancerade operationer)
    pub fn with_connection<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let conn = self.conn.lock().unwrap();
        f(&conn)
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
        }
    }
}
