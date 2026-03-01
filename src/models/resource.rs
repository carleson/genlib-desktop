use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceType {
    pub id: Option<i64>,
    pub name: String,
    pub directory_name: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl ResourceType {
    pub fn new(name: String) -> Self {
        let directory_name = sanitize_directory_name(&name);
        Self {
            id: None,
            name,
            directory_name,
            created_at: None,
            updated_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub id: Option<i64>,
    pub resource_type_id: i64,
    pub name: String,
    pub directory_name: String,
    pub information: Option<String>,
    pub comment: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub profile_image_path: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl Resource {
    pub fn new(name: String, resource_type_id: i64) -> Self {
        let directory_name = sanitize_directory_name(&name);
        Self {
            id: None,
            resource_type_id,
            name,
            directory_name,
            information: None,
            comment: None,
            lat: None,
            lon: None,
            profile_image_path: None,
            created_at: None,
            updated_at: None,
        }
    }

    pub fn validate(&self) -> Result<(), ResourceValidationError> {
        if self.name.trim().is_empty() {
            return Err(ResourceValidationError::MissingName);
        }
        if self.directory_name.is_empty() {
            return Err(ResourceValidationError::EmptyDirectoryName);
        }
        Ok(())
    }

    /// Absolut sökväg till resurskatalogen
    /// `<media_root>/resurser/<type_dir>/<resource_dir>`
    pub fn full_directory_path(&self, media_root: &str, type_dir: &str) -> std::path::PathBuf {
        std::path::Path::new(media_root)
            .join("resurser")
            .join(type_dir)
            .join(&self.directory_name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAddress {
    pub id: Option<i64>,
    pub resource_id: i64,
    pub street: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub created_at: Option<String>,
}

impl ResourceAddress {
    pub fn new(resource_id: i64) -> Self {
        Self {
            id: None,
            resource_id,
            street: None,
            postal_code: None,
            city: None,
            country: None,
            created_at: None,
        }
    }

    /// Formattera adress för visning
    pub fn display(&self) -> String {
        let mut parts = Vec::new();
        if let Some(s) = &self.street {
            if !s.is_empty() {
                parts.push(s.as_str());
            }
        }
        if let Some(pc) = &self.postal_code {
            if !pc.is_empty() {
                parts.push(pc.as_str());
            }
        }
        if let Some(c) = &self.city {
            if !c.is_empty() {
                parts.push(c.as_str());
            }
        }
        if let Some(co) = &self.country {
            if !co.is_empty() {
                parts.push(co.as_str());
            }
        }
        parts.join(", ")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDocument {
    pub id: Option<i64>,
    pub resource_id: i64,
    pub document_type_id: Option<i64>,
    pub filename: String,
    pub relative_path: String,
    pub file_size: i64,
    pub file_type: Option<String>,
    pub file_modified_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl ResourceDocument {
    pub fn is_image(&self) -> bool {
        if let Some(ref ft) = self.file_type {
            matches!(ft.as_str(), "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "tiff" | "tif")
        } else {
            false
        }
    }

    pub fn full_path(&self, media_root: &str, type_dir: &str, resource_dir: &str) -> std::path::PathBuf {
        std::path::Path::new(media_root)
            .join("resurser")
            .join(type_dir)
            .join(resource_dir)
            .join(&self.relative_path)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ResourceValidationError {
    #[error("Namn krävs")]
    MissingName,
    #[error("Katalognamn får inte vara tomt")]
    EmptyDirectoryName,
}

/// Sanitera ett katalognamn (lowercase, bevarar svenska/tyska/engelska tecken).
/// Tecken som inte är alfanumeriska ersätts med underscore.
pub fn sanitize_directory_name(name: &str) -> String {
    let sanitized: String = name
        .to_lowercase()
        .chars()
        .map(|c| match c {
            ' ' | '-' => '_',
            c if c.is_alphanumeric() || c == '_' => c,
            _ => '_',
        })
        .collect();

    let mut result = String::new();
    let mut last_was_underscore = false;
    for c in sanitized.chars() {
        if c == '_' {
            if !last_was_underscore {
                result.push(c);
            }
            last_was_underscore = true;
        } else {
            result.push(c);
            last_was_underscore = false;
        }
    }

    result.trim_matches('_').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_directory_name() {
        assert_eq!(sanitize_directory_name("Fastigheter"), "fastigheter");
        assert_eq!(sanitize_directory_name("Gamla Stan"), "gamla_stan");
        assert_eq!(sanitize_directory_name("Företag"), "företag");
        assert_eq!(sanitize_directory_name("Åkerström & Co"), "åkerström_co");
        assert_eq!(sanitize_directory_name("Östra Hamnen"), "östra_hamnen");
        assert_eq!(sanitize_directory_name("Über GmbH"), "über_gmbh");
    }

    #[test]
    fn test_resource_validate() {
        let r = Resource::new("Storkyrkan".to_string(), 1);
        assert!(r.validate().is_ok());

        let mut r2 = Resource::new("".to_string(), 1);
        r2.name = "  ".to_string();
        r2.directory_name = String::new();
        assert!(r2.validate().is_err());
    }

    #[test]
    fn test_resource_address_display() {
        let mut a = ResourceAddress::new(1);
        a.street = Some("Storgatan 1".to_string());
        a.city = Some("Stockholm".to_string());
        a.country = Some("Sverige".to_string());
        assert_eq!(a.display(), "Storgatan 1, Stockholm, Sverige");
    }
}
