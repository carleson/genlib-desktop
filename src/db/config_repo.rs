use anyhow::Result;
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::models::{DirNameFormat, SystemConfig};

pub struct ConfigRepository {
    conn: Arc<Mutex<Connection>>,
}

impl ConfigRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Hämta systemkonfiguration (skapar default om den inte finns)
    pub fn get(&self) -> Result<SystemConfig> {
        let conn = self.conn.lock().unwrap();

        let result = conn.query_row(
            "SELECT id, media_directory_path, backup_directory_path, dir_name_format, created_at, updated_at
             FROM system_config WHERE id = 1",
            [],
            |row| {
                let format_str: String = row.get(3)?;
                Ok(SystemConfig {
                    id: row.get(0)?,
                    media_directory_path: PathBuf::from(row.get::<_, String>(1)?),
                    backup_directory_path: PathBuf::from(row.get::<_, String>(2)?),
                    dir_name_format: DirNameFormat::from_db_str(&format_str),
                    created_at: row.get(4).ok(),
                    updated_at: row.get(5).ok(),
                })
            },
        );

        match result {
            Ok(config) => Ok(config),
            Err(_) => {
                // Skapa default-konfiguration
                drop(conn);
                let default_config = SystemConfig::default();
                self.save(&default_config)?;
                Ok(default_config)
            }
        }
    }

    /// Spara systemkonfiguration
    pub fn save(&self, config: &SystemConfig) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT OR REPLACE INTO system_config (id, media_directory_path, backup_directory_path, dir_name_format, updated_at)
             VALUES (1, ?1, ?2, ?3, datetime('now'))",
            params![
                config.media_directory_path.to_string_lossy().to_string(),
                config.backup_directory_path.to_string_lossy().to_string(),
                config.dir_name_format.to_string(),
            ],
        )?;

        Ok(())
    }

    /// Kontrollera om initial setup är klar
    pub fn is_setup_complete(&self) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        // Kontrollera om config finns och har giltiga sökvägar
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM system_config WHERE id = 1)",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !exists {
            return Ok(false);
        }

        // Kontrollera att media-katalogen finns
        let _config = drop(conn);
        let config = self.get()?;

        Ok(config.media_directory_path.exists())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    #[test]
    fn test_get_creates_default() {
        let db = Database::open_in_memory().unwrap();
        let repo = db.config();

        let config = repo.get().unwrap();

        // Ska ha skapat default-konfiguration
        assert_eq!(config.id, 1);
        assert!(!config.media_directory_path.as_os_str().is_empty());
    }

    #[test]
    fn test_save_and_get() {
        let db = Database::open_in_memory().unwrap();
        let repo = db.config();

        let mut config = SystemConfig::default();
        config.media_directory_path = PathBuf::from("/custom/media");
        config.backup_directory_path = PathBuf::from("/custom/backup");

        repo.save(&config).unwrap();

        let loaded = repo.get().unwrap();
        assert_eq!(loaded.media_directory_path, PathBuf::from("/custom/media"));
        assert_eq!(loaded.backup_directory_path, PathBuf::from("/custom/backup"));
    }
}
