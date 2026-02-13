//! Datastrukturer för GEDCOM-data

use chrono::NaiveDate;

/// En individ från GEDCOM-fil
#[derive(Debug, Clone)]
pub struct GedcomIndividual {
    /// GEDCOM-ID (t.ex. "@I1@")
    pub id: String,
    /// Förnamn
    pub firstname: Option<String>,
    /// Efternamn
    pub surname: Option<String>,
    /// Kön (M/F/U)
    pub sex: Option<String>,
    /// Födelsedatum
    pub birth_date: Option<GedcomDate>,
    /// Födelseort
    pub birth_place: Option<String>,
    /// Dödsdatum
    pub death_date: Option<GedcomDate>,
    /// Dödsort
    pub death_place: Option<String>,
    /// Anteckningar
    pub notes: Vec<String>,
    /// Familjer där personen är barn (FAMC)
    pub family_child: Vec<String>,
    /// Familjer där personen är förälder/make (FAMS)
    pub family_spouse: Vec<String>,
}

impl Default for GedcomIndividual {
    fn default() -> Self {
        Self {
            id: String::new(),
            firstname: None,
            surname: None,
            sex: None,
            birth_date: None,
            birth_place: None,
            death_date: None,
            death_place: None,
            notes: Vec::new(),
            family_child: Vec::new(),
            family_spouse: Vec::new(),
        }
    }
}

impl GedcomIndividual {
    /// Hämta fullständigt namn
    pub fn full_name(&self) -> String {
        match (&self.firstname, &self.surname) {
            (Some(f), Some(s)) => format!("{} {}", f, s),
            (Some(f), None) => f.clone(),
            (None, Some(s)) => s.clone(),
            (None, None) => "Okänd".to_string(),
        }
    }

    /// Generera ett katalognamn baserat på namn, födelsedatum och format
    pub fn generate_directory_name(&self, format: crate::models::DirNameFormat) -> String {
        let birth_str = self
            .birth_date
            .as_ref()
            .and_then(|d| d.to_naive_date())
            .map(|d| d.format("%Y-%m-%d").to_string());

        crate::models::Person::generate_directory_name(
            &self.firstname,
            &self.surname,
            &birth_str,
            format,
        )
    }
}

/// En familj från GEDCOM-fil
#[derive(Debug, Clone)]
pub struct GedcomFamily {
    /// GEDCOM-ID (t.ex. "@F1@")
    pub id: String,
    /// Make/man (HUSB)
    pub husband_id: Option<String>,
    /// Maka/hustru (WIFE)
    pub wife_id: Option<String>,
    /// Barn (CHIL)
    pub children_ids: Vec<String>,
    /// Vigselsdatum
    pub marriage_date: Option<GedcomDate>,
    /// Vigselort
    pub marriage_place: Option<String>,
}

impl Default for GedcomFamily {
    fn default() -> Self {
        Self {
            id: String::new(),
            husband_id: None,
            wife_id: None,
            children_ids: Vec::new(),
            marriage_date: None,
            marriage_place: None,
        }
    }
}

/// GEDCOM-datum med stöd för modifierare
#[derive(Debug, Clone)]
pub struct GedcomDate {
    /// Modifierare (ABT, BEF, AFT, etc.)
    pub modifier: Option<DateModifier>,
    /// Originalsträng från GEDCOM
    pub original: String,
    /// Parsat datum (om möjligt)
    pub date: Option<NaiveDate>,
}

impl GedcomDate {
    /// Parsa en GEDCOM-datumsträng
    pub fn parse(s: &str) -> Self {
        let s = s.trim();
        let (modifier, date_str) = Self::extract_modifier(s);
        let date = Self::parse_date_string(date_str);

        Self {
            modifier,
            original: s.to_string(),
            date,
        }
    }

    /// Hämta NaiveDate om tillgängligt
    pub fn to_naive_date(&self) -> Option<NaiveDate> {
        self.date
    }

    /// Formatera för visning
    pub fn display(&self) -> String {
        let modifier_str = self
            .modifier
            .as_ref()
            .map(|m| format!("{} ", m.display()))
            .unwrap_or_default();

        if let Some(date) = self.date {
            format!("{}{}", modifier_str, date.format("%Y-%m-%d"))
        } else {
            self.original.clone()
        }
    }

    fn extract_modifier(s: &str) -> (Option<DateModifier>, &str) {
        let upper = s.to_uppercase();
        if upper.starts_with("ABT. ") {
            (Some(DateModifier::About), s[5..].trim())
        } else if upper.starts_with("ABT ") {
            (Some(DateModifier::About), s[4..].trim())
        } else if upper.starts_with("ABOUT ") {
            (Some(DateModifier::About), s[6..].trim())
        } else if upper.starts_with("BEF. ") {
            (Some(DateModifier::Before), s[5..].trim())
        } else if upper.starts_with("BEF ") {
            (Some(DateModifier::Before), s[4..].trim())
        } else if upper.starts_with("BEFORE ") {
            (Some(DateModifier::Before), s[7..].trim())
        } else if upper.starts_with("AFT. ") {
            (Some(DateModifier::After), s[5..].trim())
        } else if upper.starts_with("AFT ") {
            (Some(DateModifier::After), s[4..].trim())
        } else if upper.starts_with("AFTER ") {
            (Some(DateModifier::After), s[6..].trim())
        } else if upper.starts_with("EST. ") {
            (Some(DateModifier::Estimated), s[5..].trim())
        } else if upper.starts_with("EST ") {
            (Some(DateModifier::Estimated), s[4..].trim())
        } else if upper.starts_with("CAL. ") {
            (Some(DateModifier::Calculated), s[5..].trim())
        } else if upper.starts_with("CAL ") {
            (Some(DateModifier::Calculated), s[4..].trim())
        } else if upper.starts_with("BET ") {
            (Some(DateModifier::Between), s[4..].trim())
        } else if upper.starts_with("FROM ") {
            (Some(DateModifier::From), s[5..].trim())
        } else if upper.starts_with("TO ") {
            (Some(DateModifier::To), s[3..].trim())
        } else {
            (None, s)
        }
    }

    fn parse_date_string(s: &str) -> Option<NaiveDate> {
        let s = s.trim();

        // Försök olika format

        // ISO-format: 1850-05-23
        if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            return Some(date);
        }

        // Svenska format: 1850-05-23, 23/5 1850
        if let Ok(date) = NaiveDate::parse_from_str(s, "%d/%m %Y") {
            return Some(date);
        }

        // GEDCOM-format: 23 MAY 1850
        if let Some(date) = Self::parse_gedcom_date(s) {
            return Some(date);
        }

        // Bara år: 1850
        if s.len() == 4 {
            if let Ok(year) = s.parse::<i32>() {
                if (1000..=2100).contains(&year) {
                    return NaiveDate::from_ymd_opt(year, 1, 1);
                }
            }
        }

        None
    }

    fn parse_gedcom_date(s: &str) -> Option<NaiveDate> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        match parts.len() {
            // "1850" - bara år
            1 => {
                let year = parts[0].parse::<i32>().ok()?;
                NaiveDate::from_ymd_opt(year, 1, 1)
            }
            // "MAY 1850" eller "1850 MAY"
            2 => {
                let (month_str, year_str) = if parts[0].parse::<i32>().is_ok() {
                    (parts[1], parts[0])
                } else {
                    (parts[0], parts[1])
                };

                let month = Self::parse_month(month_str)?;
                let year = year_str.parse::<i32>().ok()?;
                NaiveDate::from_ymd_opt(year, month, 1)
            }
            // "23 MAY 1850"
            3 => {
                let day = parts[0].parse::<u32>().ok()?;
                let month = Self::parse_month(parts[1])?;
                let year = parts[2].parse::<i32>().ok()?;
                NaiveDate::from_ymd_opt(year, month, day)
            }
            _ => None,
        }
    }

    fn parse_month(s: &str) -> Option<u32> {
        match s.to_uppercase().as_str() {
            "JAN" | "JANUARY" => Some(1),
            "FEB" | "FEBRUARY" => Some(2),
            "MAR" | "MARCH" => Some(3),
            "APR" | "APRIL" => Some(4),
            "MAY" => Some(5),
            "JUN" | "JUNE" => Some(6),
            "JUL" | "JULY" => Some(7),
            "AUG" | "AUGUST" => Some(8),
            "SEP" | "SEPTEMBER" => Some(9),
            "OCT" | "OCTOBER" => Some(10),
            "NOV" | "NOVEMBER" => Some(11),
            "DEC" | "DECEMBER" => Some(12),
            _ => None,
        }
    }
}

/// Datummodifierare
#[derive(Debug, Clone, PartialEq)]
pub enum DateModifier {
    /// Omkring (ABT)
    About,
    /// Före (BEF)
    Before,
    /// Efter (AFT)
    After,
    /// Uppskattat (EST)
    Estimated,
    /// Beräknat (CAL)
    Calculated,
    /// Mellan (BET ... AND ...)
    Between,
    /// Från (FROM)
    From,
    /// Till (TO)
    To,
}

impl DateModifier {
    pub fn display(&self) -> &'static str {
        match self {
            Self::About => "ca",
            Self::Before => "före",
            Self::After => "efter",
            Self::Estimated => "uppsk.",
            Self::Calculated => "ber.",
            Self::Between => "mellan",
            Self::From => "från",
            Self::To => "till",
        }
    }
}

/// Resultat av GEDCOM-parsning
#[derive(Debug, Clone)]
pub struct GedcomData {
    /// Alla individer
    pub individuals: Vec<GedcomIndividual>,
    /// Alla familjer
    pub families: Vec<GedcomFamily>,
    /// Metadata från HEAD
    pub source: Option<String>,
    /// Charset
    pub charset: Option<String>,
}

impl GedcomData {
    pub fn new() -> Self {
        Self {
            individuals: Vec::new(),
            families: Vec::new(),
            source: None,
            charset: None,
        }
    }

    /// Hitta individ med ID
    pub fn find_individual(&self, id: &str) -> Option<&GedcomIndividual> {
        self.individuals.iter().find(|i| i.id == id)
    }

    /// Hitta familj med ID
    pub fn find_family(&self, id: &str) -> Option<&GedcomFamily> {
        self.families.iter().find(|f| f.id == id)
    }

    /// Antal individer
    pub fn individual_count(&self) -> usize {
        self.individuals.len()
    }

    /// Antal familjer
    pub fn family_count(&self) -> usize {
        self.families.len()
    }
}

impl Default for GedcomData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gedcom_date() {
        // ISO-format
        let date = GedcomDate::parse("1850-05-23");
        assert_eq!(date.date, NaiveDate::from_ymd_opt(1850, 5, 23));

        // GEDCOM-format: dag månad år
        let date = GedcomDate::parse("23 MAY 1850");
        assert_eq!(date.date, NaiveDate::from_ymd_opt(1850, 5, 23));

        // GEDCOM-format: enkel dag (8 FEB 1911)
        let date = GedcomDate::parse("8 FEB 1911");
        assert_eq!(date.date, NaiveDate::from_ymd_opt(1911, 2, 8));
        assert_eq!(date.modifier, None);

        // Med modifierare (utan punkt)
        let date = GedcomDate::parse("ABT 1850");
        assert_eq!(date.modifier, Some(DateModifier::About));
        assert_eq!(date.date, NaiveDate::from_ymd_opt(1850, 1, 1));

        // Med modifierare (med punkt)
        let date = GedcomDate::parse("ABT. 1850");
        assert_eq!(date.modifier, Some(DateModifier::About));
        assert_eq!(date.date, NaiveDate::from_ymd_opt(1850, 1, 1));

        let date = GedcomDate::parse("BEF. 15 MAR 1900");
        assert_eq!(date.modifier, Some(DateModifier::Before));
        assert_eq!(date.date, NaiveDate::from_ymd_opt(1900, 3, 15));

        let date = GedcomDate::parse("AFT. 1920");
        assert_eq!(date.modifier, Some(DateModifier::After));
        assert_eq!(date.date, NaiveDate::from_ymd_opt(1920, 1, 1));

        let date = GedcomDate::parse("EST. JUN 1875");
        assert_eq!(date.modifier, Some(DateModifier::Estimated));
        assert_eq!(date.date, NaiveDate::from_ymd_opt(1875, 6, 1));

        let date = GedcomDate::parse("CAL. 1800");
        assert_eq!(date.modifier, Some(DateModifier::Calculated));
        assert_eq!(date.date, NaiveDate::from_ymd_opt(1800, 1, 1));

        // Bara år
        let date = GedcomDate::parse("1850");
        assert_eq!(date.date, NaiveDate::from_ymd_opt(1850, 1, 1));

        // Månad + år
        let date = GedcomDate::parse("FEB 1911");
        assert_eq!(date.date, NaiveDate::from_ymd_opt(1911, 2, 1));
    }

    #[test]
    fn test_generate_directory_name() {
        use crate::models::DirNameFormat;

        let mut indi = GedcomIndividual::default();
        indi.firstname = Some("Johan".to_string());
        indi.surname = Some("Andersson".to_string());
        indi.birth_date = Some(GedcomDate::parse("1850"));

        // SurnameFirst
        assert_eq!(
            indi.generate_directory_name(DirNameFormat::SurnameFirst),
            "andersson_johan_1850_01_01"
        );
        // FirstnameFirst
        assert_eq!(
            indi.generate_directory_name(DirNameFormat::FirstnameFirst),
            "johan_andersson_1850_01_01"
        );
    }
}
