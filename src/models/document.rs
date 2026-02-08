use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentType {
    pub id: Option<i64>,
    pub name: String,
    pub target_directory: String,
    pub default_filename: Option<String>,
    pub description: Option<String>,
}

impl DocumentType {
    pub fn new(name: String, target_directory: String) -> Self {
        Self {
            id: None,
            name,
            target_directory,
            default_filename: None,
            description: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Option<i64>,
    pub person_id: i64,
    pub document_type_id: Option<i64>,
    pub filename: String,
    pub relative_path: String,
    pub file_size: i64,
    pub file_type: Option<String>,
    pub tags: Option<String>,
    pub file_modified_at: Option<NaiveDateTime>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl Document {
    pub fn new(person_id: i64, filename: String, relative_path: String) -> Self {
        let file_type = std::path::Path::new(&filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());

        Self {
            id: None,
            person_id,
            document_type_id: None,
            filename,
            relative_path,
            file_size: 0,
            file_type,
            tags: None,
            file_modified_at: None,
            created_at: None,
            updated_at: None,
        }
    }

    pub fn get_tags_list(&self) -> Vec<&str> {
        self.tags
            .as_ref()
            .map(|t| t.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect())
            .unwrap_or_default()
    }

    pub fn set_tags(&mut self, tags: Vec<String>) {
        if tags.is_empty() {
            self.tags = None;
        } else {
            self.tags = Some(tags.join(", "));
        }
    }

    pub fn file_size_display(&self) -> String {
        const KB: i64 = 1024;
        const MB: i64 = KB * 1024;
        const GB: i64 = MB * 1024;

        match self.file_size {
            s if s >= GB => format!("{:.1} GB", s as f64 / GB as f64),
            s if s >= MB => format!("{:.1} MB", s as f64 / MB as f64),
            s if s >= KB => format!("{:.1} KB", s as f64 / KB as f64),
            s => format!("{} B", s),
        }
    }

    pub fn is_image(&self) -> bool {
        matches!(
            self.file_type.as_deref(),
            Some("jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp")
        )
    }

    pub fn is_text(&self) -> bool {
        matches!(self.file_type.as_deref(), Some("txt" | "md"))
    }

    pub fn is_pdf(&self) -> bool {
        matches!(self.file_type.as_deref(), Some("pdf"))
    }

    /// Bygg full sökväg given media root och person directory
    pub fn full_path(&self, media_root: &PathBuf, person_dir: &str) -> PathBuf {
        media_root
            .join("persons")
            .join(person_dir)
            .join(&self.relative_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_size_display() {
        let mut doc = Document::new(1, "test.pdf".into(), "dokument/test.pdf".into());

        doc.file_size = 500;
        assert_eq!(doc.file_size_display(), "500 B");

        doc.file_size = 1500;
        assert_eq!(doc.file_size_display(), "1.5 KB");

        doc.file_size = 1_500_000;
        assert_eq!(doc.file_size_display(), "1.4 MB");

        doc.file_size = 1_500_000_000;
        assert_eq!(doc.file_size_display(), "1.4 GB");
    }

    #[test]
    fn test_file_type_detection() {
        let doc = Document::new(1, "photo.jpg".into(), "bilder/photo.jpg".into());
        assert!(doc.is_image());
        assert!(!doc.is_text());
        assert!(!doc.is_pdf());

        let doc2 = Document::new(1, "notes.txt".into(), "anteckningar/notes.txt".into());
        assert!(doc2.is_text());

        let doc3 = Document::new(1, "document.pdf".into(), "dokument/document.pdf".into());
        assert!(doc3.is_pdf());
    }

    #[test]
    fn test_tags() {
        let mut doc = Document::new(1, "test.pdf".into(), "test.pdf".into());

        doc.set_tags(vec!["viktigt".into(), "arkiv".into()]);
        assert_eq!(doc.tags, Some("viktigt, arkiv".to_string()));

        let tags = doc.get_tags_list();
        assert_eq!(tags, vec!["viktigt", "arkiv"]);
    }
}
