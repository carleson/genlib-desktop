//! Restore-service för att återställa från backup

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

use anyhow::{Context, Result};
use zip::ZipArchive;

use crate::db::Database;
use crate::utils::path::get_database_path;

/// Resultat av en restore-operation
#[derive(Debug, Clone)]
pub struct RestoreResult {
    /// Antal filer återställda
    pub files_restored: usize,
    /// Om databasen återställdes
    pub database_restored: bool,
    /// Om media återställdes
    pub media_restored: bool,
}

/// Förhandsgranskning av restore
#[derive(Debug, Clone)]
pub struct RestorePreview {
    /// Innehåller databas
    pub has_database: bool,
    /// Innehåller media
    pub has_media: bool,
    /// Antal filer i backup
    pub file_count: usize,
    /// Total storlek (okomprimerad)
    pub total_size: u64,
}

/// Restore-service
pub struct RestoreService<'a> {
    db: &'a Database,
}

impl<'a> RestoreService<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Förhandsgranska en backup
    pub fn preview(&self, backup_path: &Path) -> Result<RestorePreview> {
        let file = File::open(backup_path).context("Kunde inte öppna backup-fil")?;
        let mut archive = ZipArchive::new(file).context("Kunde inte läsa ZIP-arkiv")?;

        let mut has_database = false;
        let mut has_media = false;
        let mut total_size = 0u64;

        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let name = file.name();

            if name == "genlib.db" {
                has_database = true;
            } else if name.starts_with("media/") {
                has_media = true;
            }

            total_size += file.size();
        }

        Ok(RestorePreview {
            has_database,
            has_media,
            file_count: archive.len(),
            total_size,
        })
    }

    /// Återställ från backup
    pub fn restore(&self, backup_path: &Path, restore_db: bool, restore_media: bool) -> Result<RestoreResult> {
        let file = File::open(backup_path).context("Kunde inte öppna backup-fil")?;
        let mut archive = ZipArchive::new(file).context("Kunde inte läsa ZIP-arkiv")?;

        let config = self.db.config().get()?;
        let media_dir = &config.media_directory_path;

        let mut files_restored = 0;
        let mut database_restored = false;
        let mut media_restored = false;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            // Återställ databas
            if name == "genlib.db" && restore_db {
                let db_path = get_database_path();

                // Skapa backup av befintlig databas
                if db_path.exists() {
                    let backup_existing = db_path.with_extension("db.bak");
                    let _ = fs::copy(&db_path, &backup_existing);
                }

                // Skriv ny databas
                let mut outfile = File::create(&db_path)?;
                io::copy(&mut file, &mut outfile)?;
                database_restored = true;
                files_restored += 1;
            }
            // Återställ media
            else if name.starts_with("media/") && restore_media {
                let relative_path = &name[6..]; // Ta bort "media/"
                if relative_path.is_empty() {
                    continue;
                }

                let target_path = media_dir.join(relative_path);

                if file.is_dir() {
                    fs::create_dir_all(&target_path)?;
                } else {
                    // Skapa föräldrakatalog om nödvändigt
                    if let Some(parent) = target_path.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    let mut outfile = File::create(&target_path)?;
                    io::copy(&mut file, &mut outfile)?;
                    files_restored += 1;
                    media_restored = true;
                }
            }
        }

        Ok(RestoreResult {
            files_restored,
            database_restored,
            media_restored,
        })
    }

    /// Validera en backup-fil
    pub fn validate(&self, backup_path: &Path) -> Result<bool> {
        let file = File::open(backup_path)?;
        let archive = ZipArchive::new(file);

        match archive {
            Ok(mut a) => {
                // Kontrollera att vi kan läsa alla filer
                for i in 0..a.len() {
                    let mut file = a.by_index(i)?;
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer)?;
                }
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    }
}

impl RestorePreview {
    /// Formatera total storlek för visning
    pub fn size_display(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        match self.total_size {
            b if b >= GB => format!("{:.1} GB", b as f64 / GB as f64),
            b if b >= MB => format!("{:.1} MB", b as f64 / MB as f64),
            b if b >= KB => format!("{:.1} KB", b as f64 / KB as f64),
            b => format!("{} B", b),
        }
    }
}
