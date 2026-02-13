//! GEDCOM-parser för GEDCOM 5.5-filer

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::{Context, Result};

use super::models::{GedcomData, GedcomDate, GedcomFamily, GedcomIndividual};

/// GEDCOM-parser
pub struct GedcomParser;

/// En rad i GEDCOM-filen
#[derive(Debug)]
struct GedcomLine {
    level: u32,
    tag: String,
    value: Option<String>,
    xref: Option<String>,
}

impl GedcomParser {
    /// Parsa en GEDCOM-fil
    pub fn parse_file(path: &Path) -> Result<GedcomData> {
        let file = File::open(path).context("Kunde inte öppna GEDCOM-fil")?;
        let reader = BufReader::new(file);
        Self::parse_reader(reader)
    }

    /// Parsa GEDCOM från en sträng
    pub fn parse_string(content: &str) -> Result<GedcomData> {
        let reader = BufReader::new(content.as_bytes());
        Self::parse_reader(reader)
    }

    fn parse_reader<R: BufRead>(reader: R) -> Result<GedcomData> {
        let mut data = GedcomData::new();
        let mut lines: Vec<GedcomLine> = Vec::new();

        // Läs och parsa alla rader
        for line_result in reader.lines() {
            let line = line_result.context("Kunde inte läsa rad")?;
            if let Some(parsed) = Self::parse_line(&line) {
                lines.push(parsed);
            }
        }

        // Processsa raderna
        let mut i = 0;
        while i < lines.len() {
            let line = &lines[i];

            if line.level == 0 {
                match line.tag.as_str() {
                    "HEAD" => {
                        // Parsa header
                        let (header_source, header_charset, consumed) =
                            Self::parse_header(&lines[i..]);
                        data.source = header_source;
                        data.charset = header_charset;
                        i += consumed;
                        continue;
                    }
                    "INDI" => {
                        // Parsa individ
                        let (indi, consumed) = Self::parse_individual(&lines[i..]);
                        data.individuals.push(indi);
                        i += consumed;
                        continue;
                    }
                    "FAM" => {
                        // Parsa familj
                        let (fam, consumed) = Self::parse_family(&lines[i..]);
                        data.families.push(fam);
                        i += consumed;
                        continue;
                    }
                    _ => {}
                }
            }

            i += 1;
        }

        Ok(data)
    }

    fn parse_line(line: &str) -> Option<GedcomLine> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }

        // Ta bort BOM om det finns
        let line = line.trim_start_matches('\u{feff}');

        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.is_empty() {
            return None;
        }

        let level = parts[0].parse::<u32>().ok()?;

        if parts.len() < 2 {
            return None;
        }

        // Kolla om det är en xref (t.ex. @I1@)
        let (xref, tag, value) = if parts[1].starts_with('@') && parts[1].ends_with('@') {
            let xref = Some(parts[1].to_string());
            let tag = parts.get(2).map(|s| s.to_string()).unwrap_or_default();
            (xref, tag, None)
        } else {
            let tag = parts[1].to_string();
            let value = if parts.len() > 2 {
                Some(parts[2].to_string())
            } else {
                None
            };
            (None, tag, value)
        };

        Some(GedcomLine {
            level,
            tag,
            value,
            xref,
        })
    }

    fn parse_header(lines: &[GedcomLine]) -> (Option<String>, Option<String>, usize) {
        let mut source = None;
        let mut charset = None;
        let mut i = 1; // Hoppa över HEAD-raden

        while i < lines.len() {
            let line = &lines[i];

            if line.level == 0 {
                break;
            }

            match line.tag.as_str() {
                "SOUR" => source = line.value.clone(),
                "CHAR" => charset = line.value.clone(),
                _ => {}
            }

            i += 1;
        }

        (source, charset, i)
    }

    fn parse_individual(lines: &[GedcomLine]) -> (GedcomIndividual, usize) {
        let mut indi = GedcomIndividual::default();
        let mut i = 0;

        // Första raden har xref
        if let Some(xref) = &lines[0].xref {
            indi.id = xref.clone();
        }

        i += 1;

        while i < lines.len() {
            let line = &lines[i];

            if line.level == 0 {
                break;
            }

            match line.tag.as_str() {
                "NAME" => {
                    // Bara använda första NAME-posten (efterföljande kan vara
                    // alternativa namn som TYPE aka utan förnamn)
                    if indi.firstname.is_none() && indi.surname.is_none() {
                        if let Some(ref name) = line.value {
                            let (firstname, surname) = Self::parse_name(name);
                            indi.firstname = firstname;
                            indi.surname = surname;
                        }
                    }
                }
                "SEX" => {
                    indi.sex = line.value.clone();
                }
                "BIRT" => {
                    // Parsa födelseinformation
                    let (date, place, consumed) = Self::parse_event(&lines[i..]);
                    indi.birth_date = date;
                    indi.birth_place = place;
                    i += consumed;
                    continue;
                }
                "DEAT" => {
                    // Parsa dödsinformation
                    let (date, place, consumed) = Self::parse_event(&lines[i..]);
                    indi.death_date = date;
                    indi.death_place = place;
                    i += consumed;
                    continue;
                }
                "NOTE" => {
                    if let Some(ref note) = line.value {
                        indi.notes.push(note.clone());
                    }
                }
                "FAMC" => {
                    if let Some(ref fam_id) = line.value {
                        indi.family_child.push(fam_id.clone());
                    }
                }
                "FAMS" => {
                    if let Some(ref fam_id) = line.value {
                        indi.family_spouse.push(fam_id.clone());
                    }
                }
                _ => {}
            }

            i += 1;
        }

        (indi, i)
    }

    fn parse_family(lines: &[GedcomLine]) -> (GedcomFamily, usize) {
        let mut fam = GedcomFamily::default();
        let mut i = 0;

        // Första raden har xref
        if let Some(xref) = &lines[0].xref {
            fam.id = xref.clone();
        }

        i += 1;

        while i < lines.len() {
            let line = &lines[i];

            if line.level == 0 {
                break;
            }

            match line.tag.as_str() {
                "HUSB" => {
                    fam.husband_id = line.value.clone();
                }
                "WIFE" => {
                    fam.wife_id = line.value.clone();
                }
                "CHIL" => {
                    if let Some(ref child_id) = line.value {
                        fam.children_ids.push(child_id.clone());
                    }
                }
                "MARR" => {
                    let (date, place, consumed) = Self::parse_event(&lines[i..]);
                    fam.marriage_date = date;
                    fam.marriage_place = place;
                    i += consumed;
                    continue;
                }
                _ => {}
            }

            i += 1;
        }

        (fam, i)
    }

    fn parse_event(lines: &[GedcomLine]) -> (Option<GedcomDate>, Option<String>, usize) {
        let mut date = None;
        let mut place = None;
        let base_level = lines[0].level;
        let event_level = base_level + 1; // DATE och PLAC ligger direkt under eventet
        let mut i = 1;

        while i < lines.len() {
            let line = &lines[i];

            if line.level <= base_level {
                break;
            }

            // Matcha bara taggar på direkt undernivå (t.ex. level 2 under level 1 BIRT)
            // Djupare nivåer (SOUR→DATA→DATE) ska ignoreras
            if line.level == event_level {
                match line.tag.as_str() {
                    "DATE" => {
                        if let Some(ref date_str) = line.value {
                            date = Some(GedcomDate::parse(date_str));
                        }
                    }
                    "PLAC" => {
                        place = line.value.clone();
                    }
                    _ => {}
                }
            }

            i += 1;
        }

        (date, place, i)
    }

    fn parse_name(name: &str) -> (Option<String>, Option<String>) {
        // GEDCOM-namn är i formatet "Förnamn /Efternamn/"
        let name = name.trim();

        if let Some(slash_pos) = name.find('/') {
            let firstname = name[..slash_pos].trim();
            let rest = &name[slash_pos + 1..];

            let surname = if let Some(end_slash) = rest.find('/') {
                rest[..end_slash].trim()
            } else {
                rest.trim()
            };

            let firstname = if firstname.is_empty() {
                None
            } else {
                Some(firstname.to_string())
            };

            let surname = if surname.is_empty() {
                None
            } else {
                Some(surname.to_string())
            };

            (firstname, surname)
        } else {
            // Inget efternamn markerat
            if name.is_empty() {
                (None, None)
            } else {
                (Some(name.to_string()), None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_gedcom() {
        let gedcom = r#"0 HEAD
1 SOUR Test
1 CHAR UTF-8
0 @I1@ INDI
1 NAME Johan /Andersson/
1 SEX M
1 BIRT
2 DATE 23 MAY 1850
2 PLAC Stockholm
1 DEAT
2 DATE 1920
0 @I2@ INDI
1 NAME Anna /Svensson/
1 SEX F
0 @F1@ FAM
1 HUSB @I1@
1 WIFE @I2@
1 MARR
2 DATE 1875
0 TRLR"#;

        let data = GedcomParser::parse_string(gedcom).unwrap();

        assert_eq!(data.individual_count(), 2);
        assert_eq!(data.family_count(), 1);
        assert_eq!(data.source, Some("Test".to_string()));

        let johan = data.find_individual("@I1@").unwrap();
        assert_eq!(johan.firstname, Some("Johan".to_string()));
        assert_eq!(johan.surname, Some("Andersson".to_string()));
        assert_eq!(johan.sex, Some("M".to_string()));
        assert!(johan.birth_date.is_some());
        assert_eq!(johan.birth_place, Some("Stockholm".to_string()));

        let fam = data.find_family("@F1@").unwrap();
        assert_eq!(fam.husband_id, Some("@I1@".to_string()));
        assert_eq!(fam.wife_id, Some("@I2@".to_string()));
    }

    #[test]
    fn test_parse_name() {
        let (first, last) = GedcomParser::parse_name("Johan /Andersson/");
        assert_eq!(first, Some("Johan".to_string()));
        assert_eq!(last, Some("Andersson".to_string()));

        let (first, last) = GedcomParser::parse_name("/Andersson/");
        assert_eq!(first, None);
        assert_eq!(last, Some("Andersson".to_string()));

        let (first, last) = GedcomParser::parse_name("Johan");
        assert_eq!(first, Some("Johan".to_string()));
        assert_eq!(last, None);
    }

    /// Test: djupt nästade SOUR/DATA/DATE-taggar under BIRT/DEAT ska INTE
    /// skriva över det faktiska datumet. Bara DATE på direkt undernivå (level 2
    /// under level 1) ska matchas.
    #[test]
    fn test_parse_event_ignores_nested_dates() {
        let gedcom = r#"0 HEAD
0 @I1@ INDI
1 NAME Gunnar Reinhold /Carleson/
1 SEX M
1 BIRT
2 DATE 12 MAR 1906
2 PLAC Örkened församling, Kristianstads län, Sverige
2 SOUR @S1104929828@
3 PAGE Örkened (L) CI:8 (1895-1913) Bild 2240 / Sida 216
3 QUAY 3
3 DATA
4 DATE 1895-1913
3 NOTE @N0081@
1 DEAT
2 DATE 19 JAN 1971
2 PLAC Växjö, Kronobergs län, Småland, Sverige
2 SOUR @S-898380968@
3 PAGE Begravning
3 DATA
4 DATE 23 SEP 2008
4 TEXT Carlesson, Gunnar Reinhold f. 12/3 1906
0 TRLR"#;

        let data = GedcomParser::parse_string(gedcom).unwrap();
        let gunnar = data.find_individual("@I1@").unwrap();

        // Födelsedatum ska vara 12 MAR 1906, INTE 1895-1913
        let birth = gunnar.birth_date.as_ref().expect("birth_date ska finnas");
        assert_eq!(
            birth.to_naive_date(),
            chrono::NaiveDate::from_ymd_opt(1906, 3, 12),
            "Födelsedatum ska vara 1906-03-12, inte överskrivna av nästade SOUR/DATA/DATE"
        );

        // Födelseort
        assert_eq!(
            gunnar.birth_place,
            Some("Örkened församling, Kristianstads län, Sverige".to_string())
        );

        // Dödsdatum ska vara 19 JAN 1971, INTE 23 SEP 2008
        let death = gunnar.death_date.as_ref().expect("death_date ska finnas");
        assert_eq!(
            death.to_naive_date(),
            chrono::NaiveDate::from_ymd_opt(1971, 1, 19),
            "Dödsdatum ska vara 1971-01-19, inte överskrivna av nästade SOUR/DATA/DATE"
        );

        assert_eq!(
            gunnar.death_place,
            Some("Växjö, Kronobergs län, Småland, Sverige".to_string())
        );
    }

    /// Test: NAME-taggens undertaggar (SOUR med nästade DATE) ska inte
    /// störa parsning av BIRT längre ner i posten.
    #[test]
    fn test_parse_complex_indi_with_name_sources() {
        let gedcom = r#"0 HEAD
0 @I1@ INDI
1 NAME Gunnar Reinhold /Carleson/
2 TYPE birth
2 GIVN Gunnar Reinhold
2 SURN Carleson
2 SOUR @S-1391455793@
3 DATA
4 TEXT Födelsedatum: 12 Mar 1906
2 SOUR @S1104929828@
3 DATA
4 DATE 1895-1913
1 SEX M
1 BIRT
2 DATE 12 MAR 1906
2 PLAC Örkened, Kristianstads län
1 DEAT
2 DATE 19 JAN 1971
1 BAPM
2 DATE 15 APR 1906
2 PLAC Växjö
0 TRLR"#;

        let data = GedcomParser::parse_string(gedcom).unwrap();
        let gunnar = data.find_individual("@I1@").unwrap();

        assert_eq!(gunnar.firstname, Some("Gunnar Reinhold".to_string()));
        assert_eq!(gunnar.surname, Some("Carleson".to_string()));

        // Födelsedatum: 12 MAR 1906
        assert_eq!(
            gunnar.birth_date.as_ref().unwrap().to_naive_date(),
            chrono::NaiveDate::from_ymd_opt(1906, 3, 12)
        );
        assert_eq!(gunnar.birth_place, Some("Örkened, Kristianstads län".to_string()));

        // Dödsdatum: 19 JAN 1971
        assert_eq!(
            gunnar.death_date.as_ref().unwrap().to_naive_date(),
            chrono::NaiveDate::from_ymd_opt(1971, 1, 19)
        );
    }

    /// Test: Multipla NAME-poster – andra NAME (TYPE aka) utan förnamn
    /// ska inte skriva över förnamnet från första NAME-posten.
    #[test]
    fn test_parse_multiple_name_records() {
        let gedcom = r#"0 HEAD
0 @P33@ INDI
1 NAME Johan Peter /Carleson/
2 TYPE birth
2 GIVN Johan Peter
2 SURN Carleson
1 NAME  /Carlsson/
2 TYPE aka
2 SURN Carlsson
1 SEX M
1 BIRT
2 DATE 15 NOV 1875
2 PLAC Virestad, Kronobergs län
0 TRLR"#;

        let data = GedcomParser::parse_string(gedcom).unwrap();
        let person = data.find_individual("@P33@").unwrap();

        assert_eq!(person.firstname, Some("Johan Peter".to_string()),
            "Förnamnet från första NAME ska bevaras trots andra NAME utan förnamn");
        assert_eq!(person.surname, Some("Carleson".to_string()),
            "Efternamnet från första NAME ska bevaras");

        assert_eq!(person.sex, Some("M".to_string()));
        assert_eq!(
            person.birth_date.as_ref().unwrap().to_naive_date(),
            chrono::NaiveDate::from_ymd_opt(1875, 11, 15)
        );
    }
}
