use chrono::{Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: Option<i64>,
    pub firstname: Option<String>,
    pub surname: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub death_date: Option<NaiveDate>,
    pub age: Option<i32>,
    pub directory_name: String,
    pub profile_image_path: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl Default for Person {
    fn default() -> Self {
        Self {
            id: None,
            firstname: None,
            surname: None,
            birth_date: None,
            death_date: None,
            age: None,
            directory_name: String::new(),
            profile_image_path: None,
            created_at: None,
            updated_at: None,
        }
    }
}

impl Person {
    pub fn new(firstname: Option<String>, surname: Option<String>, directory_name: String) -> Self {
        Self {
            firstname,
            surname,
            directory_name,
            ..Default::default()
        }
    }

    pub fn full_name(&self) -> String {
        match (&self.firstname, &self.surname) {
            (Some(f), Some(s)) => format!("{} {}", f, s),
            (Some(f), None) => f.clone(),
            (None, Some(s)) => s.clone(),
            (None, None) => "Okänd".to_string(),
        }
    }

    pub fn years_display(&self) -> String {
        let birth = self
            .birth_date
            .map(|d| d.format("%Y").to_string())
            .unwrap_or_default();
        let death = self
            .death_date
            .map(|d| d.format("%Y").to_string())
            .unwrap_or_default();

        match (self.birth_date, self.death_date, self.age) {
            (Some(_), Some(_), Some(age)) => format!("{}-{} ({} år)", birth, death, age),
            (Some(_), None, Some(age)) => format!("{}- ({} år)", birth, age),
            (Some(_), Some(_), None) => format!("{}-{}", birth, death),
            (Some(_), None, None) => format!("{}-", birth),
            _ => String::new(),
        }
    }

    pub fn calculate_age(&mut self) {
        let Some(birth) = self.birth_date else {
            self.age = None;
            return;
        };

        let end_date = self.death_date.unwrap_or_else(|| Utc::now().date_naive());

        let years_since_birth = end_date.year() - birth.year();
        if years_since_birth > 150 || years_since_birth < 0 {
            self.age = None;
            return;
        }

        let mut age = years_since_birth;
        if end_date.ordinal() < birth.ordinal() {
            age -= 1;
        }

        self.age = Some(age);
    }

    pub fn is_alive(&self) -> bool {
        self.death_date.is_none()
    }

    pub fn validate(&self) -> Result<(), PersonValidationError> {
        if self.firstname.is_none() && self.surname.is_none() {
            return Err(PersonValidationError::MissingName);
        }

        if let (Some(birth), Some(death)) = (self.birth_date, self.death_date) {
            if death < birth {
                return Err(PersonValidationError::DeathBeforeBirth);
            }
        }

        if self.directory_name.is_empty() {
            return Err(PersonValidationError::EmptyDirectoryName);
        }

        Ok(())
    }

    /// Generera ett katalognamn baserat på namn
    pub fn generate_directory_name(firstname: &Option<String>, surname: &Option<String>) -> String {
        let name = match (firstname, surname) {
            (Some(f), Some(s)) => format!("{}_{}", f, s),
            (Some(f), None) => f.clone(),
            (None, Some(s)) => s.clone(),
            (None, None) => "okand".to_string(),
        };

        // Rensa och normalisera
        name.to_lowercase()
            .chars()
            .map(|c| match c {
                'å' | 'ä' => 'a',
                'ö' => 'o',
                'é' | 'è' => 'e',
                ' ' | '-' => '_',
                c if c.is_alphanumeric() || c == '_' => c,
                _ => '_',
            })
            .collect::<String>()
            .trim_matches('_')
            .to_string()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PersonValidationError {
    #[error("Minst förnamn eller efternamn krävs")]
    MissingName,
    #[error("Dödsdatum kan inte vara före födelsedatum")]
    DeathBeforeBirth,
    #[error("Katalognamn får inte vara tomt")]
    EmptyDirectoryName,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_name() {
        let person = Person::new(Some("Johan".into()), Some("Andersson".into()), "johan_andersson".into());
        assert_eq!(person.full_name(), "Johan Andersson");

        let person2 = Person::new(Some("Johan".into()), None, "johan".into());
        assert_eq!(person2.full_name(), "Johan");

        let person3 = Person::new(None, Some("Andersson".into()), "andersson".into());
        assert_eq!(person3.full_name(), "Andersson");
    }

    #[test]
    fn test_generate_directory_name() {
        assert_eq!(
            Person::generate_directory_name(&Some("Johan".into()), &Some("Åkerström".into())),
            "johan_akerstrom"
        );
        assert_eq!(
            Person::generate_directory_name(&Some("Märta".into()), &None),
            "marta"
        );
    }

    #[test]
    fn test_validation() {
        let valid = Person::new(Some("Johan".into()), None, "johan".into());
        assert!(valid.validate().is_ok());

        let invalid = Person {
            firstname: None,
            surname: None,
            directory_name: "test".into(),
            ..Default::default()
        };
        assert!(matches!(
            invalid.validate(),
            Err(PersonValidationError::MissingName)
        ));
    }
}
