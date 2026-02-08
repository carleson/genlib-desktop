use std::path::{Path, PathBuf};

/// Hämta databassökväg
pub fn get_database_path() -> PathBuf {
    directories::ProjectDirs::from("se", "genlib", "Genlib")
        .map(|dirs| dirs.data_dir().join("genlib.db"))
        .unwrap_or_else(|| PathBuf::from("genlib.db"))
}

/// Hämta konfigurationssökväg
pub fn get_config_path() -> PathBuf {
    directories::ProjectDirs::from("se", "genlib", "Genlib")
        .map(|dirs| dirs.config_dir().join("config.toml"))
        .unwrap_or_else(|| PathBuf::from("config.toml"))
}

/// Normalisera sökväg för visning
pub fn display_path(path: &Path) -> String {
    // Förkorta hemkatalogen till ~
    if let Some(home) = dirs::home_dir() {
        if let Ok(stripped) = path.strip_prefix(&home) {
            return format!("~/{}", stripped.display());
        }
    }
    path.display().to_string()
}

/// Skapa en säker filnamn från en sträng
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// Hämta filändelse
pub fn get_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
}

/// Kontrollera om en fil är en bild
pub fn is_image_file(path: &Path) -> bool {
    matches!(
        get_extension(path).as_deref(),
        Some("jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp")
    )
}

/// Kontrollera om en fil är en textfil
pub fn is_text_file(path: &Path) -> bool {
    matches!(
        get_extension(path).as_deref(),
        Some("txt" | "md" | "markdown")
    )
}

/// Kontrollera om en fil är PDF
pub fn is_pdf_file(path: &Path) -> bool {
    matches!(get_extension(path).as_deref(), Some("pdf"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("hello world"), "hello world");
        assert_eq!(sanitize_filename("hello/world"), "hello_world");
        assert_eq!(sanitize_filename("file:name"), "file_name");
        assert_eq!(sanitize_filename("test<>file"), "test__file");
    }

    #[test]
    fn test_is_image_file() {
        assert!(is_image_file(Path::new("photo.jpg")));
        assert!(is_image_file(Path::new("image.PNG")));
        assert!(!is_image_file(Path::new("document.pdf")));
    }
}
