use chrono::{Datelike, NaiveDate, Utc};

/// Parse ett datum från en sträng (flexibelt format)
pub fn parse_date(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    // Försök olika format
    let formats = [
        "%Y-%m-%d",    // 2024-01-15
        "%Y/%m/%d",    // 2024/01/15
        "%d-%m-%Y",    // 15-01-2024
        "%d/%m/%Y",    // 15/01/2024
        "%Y%m%d",      // 20240115
    ];

    for format in formats {
        if let Ok(date) = NaiveDate::parse_from_str(s, format) {
            return Some(date);
        }
    }

    // Försök tolka endast år
    if s.len() == 4 {
        if let Ok(year) = s.parse::<i32>() {
            return NaiveDate::from_ymd_opt(year, 1, 1);
        }
    }

    None
}

/// Formatera ett datum för visning
pub fn format_date(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// Formatera ett datum med ålder
pub fn format_date_with_age(birth: NaiveDate, death: Option<NaiveDate>) -> String {
    let end = death.unwrap_or_else(|| Utc::now().date_naive());
    let age = calculate_age(birth, end);

    let birth_str = format_date(birth);

    match death {
        Some(d) => format!("{} - {} ({} år)", birth_str, format_date(d), age),
        None => format!("{} ({} år)", birth_str, age),
    }
}

/// Beräkna ålder
pub fn calculate_age(birth: NaiveDate, end: NaiveDate) -> i32 {
    let mut age = end.year() - birth.year();
    if end.ordinal() < birth.ordinal() {
        age -= 1;
    }
    age
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_date() {
        assert_eq!(
            parse_date("2024-01-15"),
            Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
        );
        assert_eq!(
            parse_date("2024"),
            Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())
        );
        assert_eq!(parse_date(""), None);
        assert_eq!(parse_date("invalid"), None);
    }

    #[test]
    fn test_calculate_age() {
        let birth = NaiveDate::from_ymd_opt(1990, 6, 15).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        assert_eq!(calculate_age(birth, end), 33);

        let end2 = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert_eq!(calculate_age(birth, end2), 34);
    }
}
