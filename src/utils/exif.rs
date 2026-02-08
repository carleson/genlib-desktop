//! EXIF-hantering för bilder
//!
//! Läser EXIF-metadata från bilder med hjälp av kamadak-exif.

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use anyhow::Result;
use chrono::NaiveDateTime;

/// EXIF-data från en bild
#[derive(Debug, Clone, Default)]
pub struct ExifData {
    /// Datum när bilden togs (EXIF DateTimeOriginal)
    pub date_taken: Option<NaiveDateTime>,
    /// Kameramodell
    pub camera_model: Option<String>,
    /// Kameratillverkare
    pub camera_make: Option<String>,
    /// Brännvidd i mm
    pub focal_length: Option<f64>,
    /// Bländare (f-number)
    pub f_number: Option<f64>,
    /// Exponeringstid i sekunder
    pub exposure_time: Option<String>,
    /// ISO-värde
    pub iso: Option<u32>,
    /// Bildbredd i pixlar
    pub width: Option<u32>,
    /// Bildhöjd i pixlar
    pub height: Option<u32>,
    /// GPS-latitud
    pub gps_latitude: Option<f64>,
    /// GPS-longitud
    pub gps_longitude: Option<f64>,
    /// GPS-altitud i meter
    pub gps_altitude: Option<f64>,
    /// Bildorientering (1-8)
    pub orientation: Option<u16>,
    /// Beskrivning/kommentar
    pub description: Option<String>,
    /// Copyright
    pub copyright: Option<String>,
    /// Artist/fotograf
    pub artist: Option<String>,
}

impl ExifData {
    /// Läs EXIF-data från en bildfil
    pub fn from_file(path: &Path) -> Result<Option<Self>> {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(_) => return Ok(None),
        };

        let mut reader = BufReader::new(file);

        let exif = match exif::Reader::new().read_from_container(&mut reader) {
            Ok(e) => e,
            Err(_) => return Ok(None), // Ingen EXIF-data eller format som ej stöds
        };

        let mut data = ExifData::default();

        // Iterera över alla fält
        for field in exif.fields() {
            match field.tag {
                exif::Tag::DateTimeOriginal => {
                    if let Some(dt) = Self::parse_exif_datetime(&field.display_value().to_string()) {
                        data.date_taken = Some(dt);
                    }
                }
                exif::Tag::Model => {
                    data.camera_model = Some(field.display_value().to_string().trim_matches('"').to_string());
                }
                exif::Tag::Make => {
                    data.camera_make = Some(field.display_value().to_string().trim_matches('"').to_string());
                }
                exif::Tag::FocalLength => {
                    if let exif::Value::Rational(ref v) = field.value {
                        if let Some(r) = v.first() {
                            data.focal_length = Some(r.num as f64 / r.denom as f64);
                        }
                    }
                }
                exif::Tag::FNumber => {
                    if let exif::Value::Rational(ref v) = field.value {
                        if let Some(r) = v.first() {
                            data.f_number = Some(r.num as f64 / r.denom as f64);
                        }
                    }
                }
                exif::Tag::ExposureTime => {
                    data.exposure_time = Some(field.display_value().to_string());
                }
                exif::Tag::PhotographicSensitivity => {
                    if let exif::Value::Short(ref v) = field.value {
                        data.iso = v.first().copied().map(|x| x as u32);
                    } else if let exif::Value::Long(ref v) = field.value {
                        data.iso = v.first().copied();
                    }
                }
                exif::Tag::PixelXDimension => {
                    if let exif::Value::Long(ref v) = field.value {
                        data.width = v.first().copied();
                    } else if let exif::Value::Short(ref v) = field.value {
                        data.width = v.first().map(|x| *x as u32);
                    }
                }
                exif::Tag::PixelYDimension => {
                    if let exif::Value::Long(ref v) = field.value {
                        data.height = v.first().copied();
                    } else if let exif::Value::Short(ref v) = field.value {
                        data.height = v.first().map(|x| *x as u32);
                    }
                }
                exif::Tag::GPSLatitude => {
                    if let exif::Value::Rational(ref v) = field.value {
                        data.gps_latitude = Self::parse_gps_coordinate(v);
                    }
                }
                exif::Tag::GPSLongitude => {
                    if let exif::Value::Rational(ref v) = field.value {
                        data.gps_longitude = Self::parse_gps_coordinate(v);
                    }
                }
                exif::Tag::GPSAltitude => {
                    if let exif::Value::Rational(ref v) = field.value {
                        if let Some(r) = v.first() {
                            data.gps_altitude = Some(r.num as f64 / r.denom as f64);
                        }
                    }
                }
                exif::Tag::Orientation => {
                    if let exif::Value::Short(ref v) = field.value {
                        data.orientation = v.first().copied();
                    }
                }
                exif::Tag::ImageDescription => {
                    data.description = Some(field.display_value().to_string().trim_matches('"').to_string());
                }
                exif::Tag::Copyright => {
                    data.copyright = Some(field.display_value().to_string().trim_matches('"').to_string());
                }
                exif::Tag::Artist => {
                    data.artist = Some(field.display_value().to_string().trim_matches('"').to_string());
                }
                _ => {}
            }
        }

        // Kolla om vi faktiskt hittade någon data
        if data.date_taken.is_none()
            && data.camera_model.is_none()
            && data.gps_latitude.is_none()
        {
            return Ok(None);
        }

        Ok(Some(data))
    }

    /// Parsa EXIF-datumformat (YYYY:MM:DD HH:MM:SS)
    fn parse_exif_datetime(s: &str) -> Option<NaiveDateTime> {
        // Format: "2024:01:15 14:30:00"
        NaiveDateTime::parse_from_str(s, "%Y:%m:%d %H:%M:%S").ok()
    }

    /// Parsa GPS-koordinat från grader, minuter, sekunder
    fn parse_gps_coordinate(rationals: &[exif::Rational]) -> Option<f64> {
        if rationals.len() < 3 {
            return None;
        }

        let degrees = rationals[0].num as f64 / rationals[0].denom as f64;
        let minutes = rationals[1].num as f64 / rationals[1].denom as f64;
        let seconds = rationals[2].num as f64 / rationals[2].denom as f64;

        Some(degrees + minutes / 60.0 + seconds / 3600.0)
    }

    /// Kolla om det finns GPS-data
    pub fn has_gps(&self) -> bool {
        self.gps_latitude.is_some() && self.gps_longitude.is_some()
    }

    /// Formatera kamerainformation
    pub fn camera_info(&self) -> Option<String> {
        match (&self.camera_make, &self.camera_model) {
            (Some(make), Some(model)) => {
                // Undvik dubblering om modellen innehåller tillverkarnamnet
                if model.to_lowercase().contains(&make.to_lowercase()) {
                    Some(model.clone())
                } else {
                    Some(format!("{} {}", make, model))
                }
            }
            (None, Some(model)) => Some(model.clone()),
            (Some(make), None) => Some(make.clone()),
            (None, None) => None,
        }
    }

    /// Formatera exponeringsinformation
    pub fn exposure_info(&self) -> Option<String> {
        let mut parts = Vec::new();

        if let Some(f) = self.f_number {
            parts.push(format!("f/{:.1}", f));
        }

        if let Some(ref exp) = self.exposure_time {
            parts.push(exp.clone());
        }

        if let Some(iso) = self.iso {
            parts.push(format!("ISO {}", iso));
        }

        if let Some(fl) = self.focal_length {
            parts.push(format!("{:.0}mm", fl));
        }

        if parts.is_empty() {
            None
        } else {
            Some(parts.join(" • "))
        }
    }

    /// Formatera GPS-koordinater
    pub fn gps_string(&self) -> Option<String> {
        if let (Some(lat), Some(lon)) = (self.gps_latitude, self.gps_longitude) {
            Some(format!("{:.6}, {:.6}", lat, lon))
        } else {
            None
        }
    }

    /// Formatera bildstorlek
    pub fn dimensions_string(&self) -> Option<String> {
        if let (Some(w), Some(h)) = (self.width, self.height) {
            Some(format!("{}×{}", w, h))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike};

    #[test]
    fn test_parse_exif_datetime() {
        let dt = ExifData::parse_exif_datetime("2024:01:15 14:30:00");
        assert!(dt.is_some());
        let dt = dt.unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.hour(), 14);
        assert_eq!(dt.minute(), 30);
    }

    #[test]
    fn test_camera_info() {
        let mut data = ExifData::default();
        data.camera_make = Some("Canon".to_string());
        data.camera_model = Some("EOS R5".to_string());
        assert_eq!(data.camera_info(), Some("Canon EOS R5".to_string()));

        // Test when model contains make
        data.camera_model = Some("Canon EOS R5".to_string());
        assert_eq!(data.camera_info(), Some("Canon EOS R5".to_string()));
    }

    #[test]
    fn test_exposure_info() {
        let mut data = ExifData::default();
        data.f_number = Some(2.8);
        data.exposure_time = Some("1/125".to_string());
        data.iso = Some(400);
        data.focal_length = Some(50.0);

        let info = data.exposure_info().unwrap();
        assert!(info.contains("f/2.8"));
        assert!(info.contains("ISO 400"));
        assert!(info.contains("50mm"));
    }
}
