//! Backup-service för att skapa och hantera backuper

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Local;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::db::Database;
use crate::utils::path::get_database_path;

/// Resultat av en backup-operation
#[derive(Debug, Clone)]
pub struct BackupResult {
    /// Sökväg till backup-filen
    pub path: PathBuf,
    /// Storlek i bytes
    pub size: u64,
    /// Antal filer inkluderade
    pub file_count: usize,
    /// Datum för backup
    pub created_at: String,
}

impl BackupResult {
    /// Formatera storlek för visning
    pub fn size_display(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        match self.size {
            b if b >= GB => format!("{:.1} GB", b as f64 / GB as f64),
            b if b >= MB => format!("{:.1} MB", b as f64 / MB as f64),
            b if b >= KB => format!("{:.1} KB", b as f64 / KB as f64),
            b => format!("{} B", b),
        }
    }
}

/// Information om en befintlig backup
#[derive(Debug, Clone)]
pub struct BackupInfo {
    /// Sökväg till backup-filen
    pub path: PathBuf,
    /// Filnamn
    pub filename: String,
    /// Storlek i bytes
    pub size: u64,
    /// Datum (extraherat från filnamn)
    pub date: Option<String>,
}

impl BackupInfo {
    /// Formatera storlek för visning
    pub fn size_display(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        match self.size {
            b if b >= GB => format!("{:.1} GB", b as f64 / GB as f64),
            b if b >= MB => format!("{:.1} MB", b as f64 / MB as f64),
            b if b >= KB => format!("{:.1} KB", b as f64 / KB as f64),
            b => format!("{} B", b),
        }
    }
}

/// Backup-service
pub struct BackupService<'a> {
    db: &'a Database,
}

impl<'a> BackupService<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Skapa en backup
    pub fn create_backup(&self) -> Result<BackupResult> {
        self.create_zip("genlib_backup")
    }

    /// Skapa ett arkiv (samma som backup men med annat prefix)
    pub fn create_archive(&self) -> Result<BackupResult> {
        self.create_zip("genlib_archive")
    }

    fn create_zip(&self, prefix: &str) -> Result<BackupResult> {
        let config = self.db.config().get()?;
        let backup_dir = &config.backup_directory_path;

        fs::create_dir_all(backup_dir).context("Kunde inte skapa backup-katalog")?;

        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{}_{}.zip", prefix, timestamp);
        let backup_path = backup_dir.join(&filename);

        let file = File::create(&backup_path).context("Kunde inte skapa backup-fil")?;
        let mut zip = ZipWriter::new(file);

        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .compression_level(Some(6));

        let mut file_count = 0;

        let db_path = get_database_path();
        if db_path.exists() {
            self.add_file_to_zip(&mut zip, &db_path, "genlib.db", options)?;
            file_count += 1;
        }

        let media_dir = &config.media_directory_path;
        if media_dir.exists() {
            file_count += self.add_directory_to_zip(&mut zip, media_dir, "media", options)?;
        }

        zip.finish().context("Kunde inte avsluta ZIP-fil")?;

        let metadata = fs::metadata(&backup_path)?;

        Ok(BackupResult {
            path: backup_path,
            size: metadata.len(),
            file_count,
            created_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        })
    }

    /// Lista befintliga backuper
    pub fn list_backups(&self) -> Result<Vec<BackupInfo>> {
        let config = self.db.config().get()?;
        let backup_dir = &config.backup_directory_path;

        if !backup_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();

        for entry in fs::read_dir(backup_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "zip").unwrap_or(false) {
                let filename = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let metadata = fs::metadata(&path)?;

                // Försök extrahera datum från filnamn (genlib_backup_YYYYMMDD_HHMMSS.zip)
                let date = Self::extract_date_from_filename(&filename);

                backups.push(BackupInfo {
                    path,
                    filename,
                    size: metadata.len(),
                    date,
                });
            }
        }

        // Sortera med nyaste först
        backups.sort_by(|a, b| b.filename.cmp(&a.filename));

        Ok(backups)
    }

    /// Ta bort en backup
    pub fn delete_backup(&self, path: &Path) -> Result<()> {
        fs::remove_file(path).context("Kunde inte ta bort backup-fil")?;
        Ok(())
    }

    fn add_file_to_zip<W: Write + std::io::Seek>(
        &self,
        zip: &mut ZipWriter<W>,
        file_path: &Path,
        archive_name: &str,
        options: SimpleFileOptions,
    ) -> Result<()> {
        zip.start_file(archive_name, options)?;

        let mut file = File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        zip.write_all(&buffer)?;

        Ok(())
    }

    fn add_directory_to_zip<W: Write + std::io::Seek>(
        &self,
        zip: &mut ZipWriter<W>,
        dir_path: &Path,
        base_name: &str,
        options: SimpleFileOptions,
    ) -> Result<usize> {
        let mut count = 0;

        for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            let relative = path.strip_prefix(dir_path).unwrap_or(path);
            let archive_name = format!("{}/{}", base_name, relative.display());

            if path.is_file() {
                self.add_file_to_zip(zip, path, &archive_name, options)?;
                count += 1;
            } else if path.is_dir() && path != dir_path {
                zip.add_directory(&archive_name, options)?;
            }
        }

        Ok(count)
    }

    fn extract_date_from_filename(filename: &str) -> Option<String> {
        // Format: genlib_backup_YYYYMMDD_HHMMSS.zip
        if filename.starts_with("genlib_backup_") && filename.ends_with(".zip") {
            let date_part = &filename[14..filename.len() - 4];
            if date_part.len() >= 15 {
                let year = &date_part[0..4];
                let month = &date_part[4..6];
                let day = &date_part[6..8];
                let hour = &date_part[9..11];
                let minute = &date_part[11..13];
                let second = &date_part[13..15];
                return Some(format!(
                    "{}-{}-{} {}:{}:{}",
                    year, month, day, hour, minute, second
                ));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_date_from_filename() {
        let date = BackupService::extract_date_from_filename("genlib_backup_20260130_143022.zip");
        assert_eq!(date, Some("2026-01-30 14:30:22".to_string()));

        let date = BackupService::extract_date_from_filename("other_file.zip");
        assert_eq!(date, None);
    }
}
