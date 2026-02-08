//! Filoperationer för dokumenthantering

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Kopiera en fil till destinationskatalog
/// Skapar katalogen om den inte finns
pub fn copy_file_to_directory(source: &Path, dest_dir: &Path, filename: &str) -> Result<PathBuf> {
    // Skapa katalog om den inte finns
    fs::create_dir_all(dest_dir)
        .with_context(|| format!("Kunde inte skapa katalog: {:?}", dest_dir))?;

    let dest_path = dest_dir.join(filename);

    // Kopiera filen
    fs::copy(source, &dest_path)
        .with_context(|| format!("Kunde inte kopiera fil från {:?} till {:?}", source, dest_path))?;

    Ok(dest_path)
}

/// Flytta en fil till ny plats
pub fn move_file(source: &Path, dest: &Path) -> Result<()> {
    // Skapa målkatalog om den inte finns
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    // Försök rename först (snabbast om samma filsystem)
    if fs::rename(source, dest).is_ok() {
        return Ok(());
    }

    // Om rename misslyckas (olika filsystem), kopiera och ta bort
    fs::copy(source, dest)?;
    fs::remove_file(source)?;

    Ok(())
}

/// Ta bort en fil
pub fn delete_file(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_file(path)
            .with_context(|| format!("Kunde inte ta bort fil: {:?}", path))?;
    }
    Ok(())
}

/// Ta bort en katalog rekursivt (endast om tom eller force=true)
pub fn delete_directory(path: &Path, force: bool) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    if force {
        fs::remove_dir_all(path)
            .with_context(|| format!("Kunde inte ta bort katalog: {:?}", path))?;
    } else {
        fs::remove_dir(path)
            .with_context(|| format!("Kunde inte ta bort katalog (ej tom?): {:?}", path))?;
    }

    Ok(())
}

/// Läs textfil till sträng
pub fn read_text_file(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .with_context(|| format!("Kunde inte läsa fil: {:?}", path))
}

/// Skriv text till fil
pub fn write_text_file(path: &Path, content: &str) -> Result<()> {
    // Skapa katalog om den inte finns
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, content)
        .with_context(|| format!("Kunde inte skriva till fil: {:?}", path))
}

/// Hämta filstorlek i bytes
pub fn get_file_size(path: &Path) -> Result<u64> {
    let metadata = fs::metadata(path)
        .with_context(|| format!("Kunde inte läsa metadata för: {:?}", path))?;
    Ok(metadata.len())
}

/// Hämta modifieringstid
pub fn get_modified_time(path: &Path) -> Result<chrono::NaiveDateTime> {
    let metadata = fs::metadata(path)?;
    let modified = metadata.modified()?;
    let duration = modified
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();

    Ok(chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
        .unwrap_or_default()
        .naive_utc())
}

/// Skanna en katalog rekursivt och returnera alla filer
pub fn scan_directory(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();

    for entry in WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }

    Ok(files)
}

/// Skanna katalog och returnera filer med relativ sökväg
pub fn scan_directory_relative(base_dir: &Path) -> Result<Vec<(PathBuf, String)>> {
    if !base_dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();

    for entry in WalkDir::new(base_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            let full_path = entry.path().to_path_buf();
            if let Ok(relative) = full_path.strip_prefix(base_dir) {
                let relative_str = relative.to_string_lossy().to_string();
                // Normalisera till forward slashes
                let relative_normalized = relative_str.replace('\\', "/");
                files.push((full_path, relative_normalized));
            }
        }
    }

    Ok(files)
}

/// Generera unikt filnamn om filen redan finns
pub fn unique_filename(dir: &Path, filename: &str) -> String {
    let path = dir.join(filename);

    if !path.exists() {
        return filename.to_string();
    }

    let stem = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(filename);

    let extension = Path::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .map(|e| format!(".{}", e))
        .unwrap_or_default();

    for i in 2..1000 {
        let new_name = format!("{}_{}{}", stem, i, extension);
        if !dir.join(&new_name).exists() {
            return new_name;
        }
    }

    // Fallback med timestamp
    let timestamp = chrono::Utc::now().timestamp();
    format!("{}_{}{}", stem, timestamp, extension)
}

/// Kontrollera om en sökväg är en bild
pub fn is_image_path(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .as_deref(),
        Some("jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp")
    )
}

/// Kontrollera om en sökväg är en textfil
pub fn is_text_path(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .as_deref(),
        Some("txt" | "md" | "markdown")
    )
}

/// Kontrollera om en sökväg är en PDF
pub fn is_pdf_path(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .as_deref(),
        Some("pdf")
    )
}

/// Hämta filändelse (lowercase)
pub fn get_file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
}

/// Säkerställ att en katalog finns
pub fn ensure_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)
            .with_context(|| format!("Kunde inte skapa katalog: {:?}", path))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_unique_filename() {
        let dir = tempdir().unwrap();

        // Första filen
        assert_eq!(unique_filename(dir.path(), "test.txt"), "test.txt");

        // Skapa filen
        fs::write(dir.path().join("test.txt"), "").unwrap();

        // Nu ska den få suffix
        assert_eq!(unique_filename(dir.path(), "test.txt"), "test_2.txt");
    }

    #[test]
    fn test_is_image_path() {
        assert!(is_image_path(Path::new("photo.jpg")));
        assert!(is_image_path(Path::new("image.PNG")));
        assert!(is_image_path(Path::new("picture.webp")));
        assert!(!is_image_path(Path::new("document.pdf")));
        assert!(!is_image_path(Path::new("notes.txt")));
    }

    #[test]
    fn test_scan_directory_relative() {
        let dir = tempdir().unwrap();

        // Skapa struktur
        let sub = dir.path().join("subdir");
        fs::create_dir_all(&sub).unwrap();
        fs::write(dir.path().join("file1.txt"), "").unwrap();
        fs::write(sub.join("file2.txt"), "").unwrap();

        let files = scan_directory_relative(dir.path()).unwrap();

        assert_eq!(files.len(), 2);

        let relatives: Vec<_> = files.iter().map(|(_, r)| r.as_str()).collect();
        assert!(relatives.contains(&"file1.txt"));
        assert!(relatives.contains(&"subdir/file2.txt"));
    }
}
