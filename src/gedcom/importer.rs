//! GEDCOM-importer för att importera data till databasen

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};

use crate::db::Database;
use crate::models::{Person, PersonRelationship, RelationshipType};

use super::models::{GedcomData, GedcomFamily, GedcomIndividual};
use super::parser::GedcomParser;

/// Resultat av en GEDCOM-import
#[derive(Debug, Clone)]
pub struct ImportResult {
    /// Antal importerade personer
    pub persons_imported: usize,
    /// Antal importerade relationer
    pub relations_imported: usize,
    /// Antal överhoppade (duplicerade)
    pub skipped: usize,
    /// Varningar
    pub warnings: Vec<String>,
}

impl ImportResult {
    pub fn new() -> Self {
        Self {
            persons_imported: 0,
            relations_imported: 0,
            skipped: 0,
            warnings: Vec::new(),
        }
    }

    /// Sammanfattning av importen
    pub fn summary(&self) -> String {
        format!(
            "{} personer, {} relationer importerade{}",
            self.persons_imported,
            self.relations_imported,
            if self.skipped > 0 {
                format!(" ({} överhoppade)", self.skipped)
            } else {
                String::new()
            }
        )
    }
}

impl Default for ImportResult {
    fn default() -> Self {
        Self::new()
    }
}

/// GEDCOM-importer
pub struct GedcomImporter<'a> {
    db: &'a Database,
}

impl<'a> GedcomImporter<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Importera en GEDCOM-fil
    pub fn import_file(&self, path: &Path) -> Result<ImportResult> {
        let data = GedcomParser::parse_file(path).context("Kunde inte parsa GEDCOM-fil")?;
        self.import_data(&data)
    }

    /// Importera GEDCOM-data
    pub fn import_data(&self, data: &GedcomData) -> Result<ImportResult> {
        let mut result = ImportResult::new();

        // Mappning från GEDCOM-ID till databas-ID
        let mut id_map: HashMap<String, i64> = HashMap::new();

        // Steg 1: Importera alla individer
        for indi in &data.individuals {
            match self.import_individual(indi) {
                Ok(Some(person_id)) => {
                    id_map.insert(indi.id.clone(), person_id);
                    result.persons_imported += 1;
                }
                Ok(None) => {
                    result.skipped += 1;
                }
                Err(e) => {
                    result.warnings.push(format!(
                        "Kunde inte importera {}: {}",
                        indi.full_name(),
                        e
                    ));
                }
            }
        }

        // Steg 2: Importera relationer från familjer
        for family in &data.families {
            match self.import_family_relations(family, &id_map) {
                Ok(count) => {
                    result.relations_imported += count;
                }
                Err(e) => {
                    result.warnings.push(format!(
                        "Kunde inte importera relationer för familj {}: {}",
                        family.id, e
                    ));
                }
            }
        }

        Ok(result)
    }

    /// Förhandsgranska import utan att faktiskt importera
    pub fn preview(&self, data: &GedcomData) -> ImportPreview {
        let mut preview = ImportPreview {
            total_individuals: data.individual_count(),
            total_families: data.family_count(),
            new_persons: 0,
            existing_persons: 0,
            estimated_relations: 0,
            sample_persons: Vec::new(),
        };

        // Räkna nya vs befintliga
        for indi in &data.individuals {
            let fmt = self.db.config().get().map(|c| c.dir_name_format).unwrap_or_default();
            let dir_name = indi.generate_directory_name(fmt);
            if self
                .db
                .persons()
                .find_by_directory(&dir_name)
                .ok()
                .flatten()
                .is_some()
            {
                preview.existing_persons += 1;
            } else {
                preview.new_persons += 1;
            }

            // Lägg till sample
            if preview.sample_persons.len() < 5 {
                preview.sample_persons.push(PersonPreview {
                    name: indi.full_name(),
                    birth_year: indi
                        .birth_date
                        .as_ref()
                        .and_then(|d| d.to_naive_date())
                        .map(|d| d.format("%Y").to_string()),
                    death_year: indi
                        .death_date
                        .as_ref()
                        .and_then(|d| d.to_naive_date())
                        .map(|d| d.format("%Y").to_string()),
                });
            }
        }

        // Uppskatta antal relationer
        for family in &data.families {
            // Förälder-barn relationer
            let parent_count = [&family.husband_id, &family.wife_id]
                .iter()
                .filter(|p| p.is_some())
                .count();
            let child_count = family.children_ids.len();
            preview.estimated_relations += parent_count * child_count;

            // Make/maka relation
            if family.husband_id.is_some() && family.wife_id.is_some() {
                preview.estimated_relations += 1;
            }

            // Syskon-relationer
            if child_count > 1 {
                preview.estimated_relations += child_count * (child_count - 1) / 2;
            }
        }

        preview
    }

    fn import_individual(&self, indi: &GedcomIndividual) -> Result<Option<i64>> {
        let fmt = self.db.config().get().map(|c| c.dir_name_format).unwrap_or_default();
        let dir_name = indi.generate_directory_name(fmt);

        // Kolla om personen redan finns
        if self
            .db
            .persons()
            .find_by_directory(&dir_name)
            .ok()
            .flatten()
            .is_some()
        {
            return Ok(None);
        }

        // Skapa unik katalognamn
        let unique_dir_name = self.generate_unique_directory_name(&dir_name)?;

        let mut person = Person {
            id: None,
            firstname: indi.firstname.clone(),
            surname: indi.surname.clone(),
            birth_place: indi.birth_place.clone(),
            birth_date: indi.birth_date.as_ref().and_then(|d| d.to_naive_date()),
            death_date: indi.death_date.as_ref().and_then(|d| d.to_naive_date()),
            directory_name: unique_dir_name,
            profile_image_path: None,
            created_at: None,
            updated_at: None,
            age: None,
        };

        // Beräkna ålder
        person.calculate_age();

        self.db.persons().create(&mut person)?;

        Ok(person.id)
    }

    fn import_family_relations(
        &self,
        family: &GedcomFamily,
        id_map: &HashMap<String, i64>,
    ) -> Result<usize> {
        let mut count = 0;

        // Hämta föräldra-IDs
        let husband_db_id = family
            .husband_id
            .as_ref()
            .and_then(|id| id_map.get(id))
            .copied();
        let wife_db_id = family
            .wife_id
            .as_ref()
            .and_then(|id| id_map.get(id))
            .copied();

        // Skapa make/maka-relation
        if let (Some(h_id), Some(w_id)) = (husband_db_id, wife_db_id) {
            if self.create_relation_if_not_exists(h_id, w_id, RelationshipType::Spouse)? {
                count += 1;
            }
        }

        // Skapa förälder-barn-relationer
        // Tredje argumentet till PersonRelationship::new() = vad person_1 ÄR till person_2
        // Förälder ÄR Parent TILL barn → RelationshipType::Parent
        for child_gedcom_id in &family.children_ids {
            if let Some(&child_db_id) = id_map.get(child_gedcom_id) {
                // Far är förälder till barn
                if let Some(h_id) = husband_db_id {
                    if self.create_relation_if_not_exists(h_id, child_db_id, RelationshipType::Parent)?
                    {
                        count += 1;
                    }
                }

                // Mor är förälder till barn
                if let Some(w_id) = wife_db_id {
                    if self.create_relation_if_not_exists(w_id, child_db_id, RelationshipType::Parent)?
                    {
                        count += 1;
                    }
                }
            }
        }

        // Skapa syskon-relationer
        let child_db_ids: Vec<i64> = family
            .children_ids
            .iter()
            .filter_map(|id| id_map.get(id).copied())
            .collect();

        for i in 0..child_db_ids.len() {
            for j in (i + 1)..child_db_ids.len() {
                if self.create_relation_if_not_exists(
                    child_db_ids[i],
                    child_db_ids[j],
                    RelationshipType::Sibling,
                )? {
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    fn create_relation_if_not_exists(
        &self,
        person_a_id: i64,
        person_b_id: i64,
        rel_type: RelationshipType,
    ) -> Result<bool> {
        // Kolla om relationen redan finns
        if self.db.relationships().exists(person_a_id, person_b_id)? {
            return Ok(false);
        }

        let mut relationship = PersonRelationship::new(person_a_id, person_b_id, rel_type);

        self.db.relationships().create(&mut relationship)?;

        Ok(true)
    }

    fn generate_unique_directory_name(&self, base_name: &str) -> Result<String> {
        let base_name = if base_name.is_empty() {
            "okand"
        } else {
            base_name
        };

        // Prova originalnamnet först
        if self
            .db
            .persons()
            .find_by_directory(base_name)?
            .is_none()
        {
            return Ok(base_name.to_string());
        }

        // Lägg till nummer
        for i in 2..1000 {
            let candidate = format!("{}_{}", base_name, i);
            if self
                .db
                .persons()
                .find_by_directory(&candidate)?
                .is_none()
            {
                return Ok(candidate);
            }
        }

        // Fallback med timestamp
        Ok(format!(
            "{}_{}",
            base_name,
            chrono::Utc::now().timestamp()
        ))
    }
}

/// Förhandsgranskning av import
#[derive(Debug, Clone)]
pub struct ImportPreview {
    /// Totalt antal individer i GEDCOM
    pub total_individuals: usize,
    /// Totalt antal familjer i GEDCOM
    pub total_families: usize,
    /// Nya personer att importera
    pub new_persons: usize,
    /// Befintliga personer (överhoppas)
    pub existing_persons: usize,
    /// Uppskattat antal relationer
    pub estimated_relations: usize,
    /// Exempel på personer
    pub sample_persons: Vec<PersonPreview>,
}

/// Förhandsgranskning av en person
#[derive(Debug, Clone)]
pub struct PersonPreview {
    pub name: String,
    pub birth_year: Option<String>,
    pub death_year: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_preview() {
        let db = Database::open_in_memory().unwrap();
        let importer = GedcomImporter::new(&db);

        let gedcom = r#"0 HEAD
0 @I1@ INDI
1 NAME Johan /Andersson/
1 BIRT
2 DATE 1850
0 @I2@ INDI
1 NAME Anna /Svensson/
0 @F1@ FAM
1 HUSB @I1@
1 WIFE @I2@
0 TRLR"#;

        let data = GedcomParser::parse_string(gedcom).unwrap();
        let preview = importer.preview(&data);

        assert_eq!(preview.total_individuals, 2);
        assert_eq!(preview.total_families, 1);
        assert_eq!(preview.new_persons, 2);
        assert_eq!(preview.existing_persons, 0);
        assert_eq!(preview.estimated_relations, 1); // make/maka
    }

    #[test]
    fn test_import_data() {
        let db = Database::open_in_memory().unwrap();
        db.migrate().unwrap();

        let importer = GedcomImporter::new(&db);

        let gedcom = r#"0 HEAD
0 @I1@ INDI
1 NAME Johan /Andersson/
1 BIRT
2 DATE 1850
0 @I2@ INDI
1 NAME Anna /Svensson/
0 @I3@ INDI
1 NAME Erik /Andersson/
0 @F1@ FAM
1 HUSB @I1@
1 WIFE @I2@
1 CHIL @I3@
0 TRLR"#;

        let data = GedcomParser::parse_string(gedcom).unwrap();
        let result = importer.import_data(&data).unwrap();

        assert_eq!(result.persons_imported, 3);
        assert_eq!(result.relations_imported, 3); // make/maka + 2 förälder-barn

        // Verifiera att personerna finns
        let persons = db.persons().find_all().unwrap();
        assert_eq!(persons.len(), 3);
    }

    /// Utförligt test: verifierar relationsriktningar, datum och födelseort
    /// genom hela import-pipelinen (GEDCOM → parser → importer → DB → repo-query)
    #[test]
    fn test_import_family_relationships_correct_direction() {
        use chrono::NaiveDate;
        use crate::models::RelationshipType;

        let db = Database::open_in_memory().unwrap();
        db.migrate().unwrap();

        let importer = GedcomImporter::new(&db);

        // Komplett GEDCOM med familj: far + mor + 2 barn, alla med fullständiga datum
        let gedcom = r#"0 HEAD
1 SOUR TestProgram
1 CHAR UTF-8
0 @I1@ INDI
1 NAME Karl /Johansson/
1 SEX M
1 BIRT
2 DATE 12 MAR 1906
2 PLAC Lund, Malmöhus län, Sverige
1 DEAT
2 DATE 3 OCT 1985
1 FAMS @F1@
0 @I2@ INDI
1 NAME Maria /Persson/
1 SEX F
1 BIRT
2 DATE 8 FEB 1911
2 PLAC Stockholm
1 FAMS @F1@
0 @I3@ INDI
1 NAME Erik /Johansson/
1 SEX M
1 BIRT
2 DATE 15 JUN 1935
2 PLAC Malmö
1 FAMC @F1@
0 @I4@ INDI
1 NAME Anna /Johansson/
1 SEX F
1 BIRT
2 DATE 22 DEC 1938
1 FAMC @F1@
0 @F1@ FAM
1 HUSB @I1@
1 WIFE @I2@
1 CHIL @I3@
1 CHIL @I4@
1 MARR
2 DATE 5 MAY 1934
2 PLAC Lund
0 TRLR"#;

        let data = GedcomParser::parse_string(gedcom).unwrap();
        assert_eq!(data.individual_count(), 4);
        assert_eq!(data.family_count(), 1);

        let result = importer.import_data(&data).unwrap();
        assert_eq!(result.persons_imported, 4);
        // Relationer: 1 make/maka + 2 far-barn + 2 mor-barn + 1 syskon = 6
        assert_eq!(result.relations_imported, 6);

        // Hämta alla personer
        let persons = db.persons().find_all().unwrap();
        assert_eq!(persons.len(), 4);

        let karl = persons.iter().find(|p| p.firstname.as_deref() == Some("Karl")).unwrap();
        let maria = persons.iter().find(|p| p.firstname.as_deref() == Some("Maria")).unwrap();
        let erik = persons.iter().find(|p| p.firstname.as_deref() == Some("Erik")).unwrap();
        let anna = persons.iter().find(|p| p.firstname.as_deref() == Some("Anna")).unwrap();

        let karl_id = karl.id.unwrap();
        let maria_id = maria.id.unwrap();
        let erik_id = erik.id.unwrap();
        let anna_id = anna.id.unwrap();

        // --- Verifiera datum ---
        assert_eq!(karl.birth_date, NaiveDate::from_ymd_opt(1906, 3, 12));
        assert_eq!(karl.death_date, NaiveDate::from_ymd_opt(1985, 10, 3));
        assert_eq!(maria.birth_date, NaiveDate::from_ymd_opt(1911, 2, 8));
        assert_eq!(erik.birth_date, NaiveDate::from_ymd_opt(1935, 6, 15));
        assert_eq!(anna.birth_date, NaiveDate::from_ymd_opt(1938, 12, 22));

        // --- Verifiera födelseort ---
        assert_eq!(karl.birth_place, Some("Lund, Malmöhus län, Sverige".to_string()));
        assert_eq!(maria.birth_place, Some("Stockholm".to_string()));
        assert_eq!(erik.birth_place, Some("Malmö".to_string()));
        assert_eq!(anna.birth_place, None); // Anna hade ingen PLAC

        // --- Verifiera relationer från Karls perspektiv (far) ---
        let karl_rels = db.relationships().find_by_person_with_names(karl_id).unwrap();
        // Karl ska ha: Make/Maka(Maria), Barn(Erik), Barn(Anna)
        let karl_spouse: Vec<_> = karl_rels.iter()
            .filter(|r| r.relationship_type == RelationshipType::Spouse)
            .collect();
        assert_eq!(karl_spouse.len(), 1, "Karl ska ha 1 make/maka-relation");
        assert_eq!(karl_spouse[0].other_person_id, maria_id);

        let karl_children: Vec<_> = karl_rels.iter()
            .filter(|r| r.relationship_type == RelationshipType::Child)
            .collect();
        assert_eq!(karl_children.len(), 2, "Karl ska ha 2 barn");
        let karl_child_ids: Vec<i64> = karl_children.iter().map(|r| r.other_person_id).collect();
        assert!(karl_child_ids.contains(&erik_id), "Erik ska vara Karls barn");
        assert!(karl_child_ids.contains(&anna_id), "Anna ska vara Karls barn");

        // Karl ska INTE ha några föräldrar
        let karl_parents: Vec<_> = karl_rels.iter()
            .filter(|r| r.relationship_type == RelationshipType::Parent)
            .collect();
        assert_eq!(karl_parents.len(), 0, "Karl ska INTE ha föräldrar i detta dataset");

        // --- Verifiera relationer från Eriks perspektiv (barn) ---
        let erik_rels = db.relationships().find_by_person_with_names(erik_id).unwrap();
        // Erik ska ha: Förälder(Karl), Förälder(Maria), Syskon(Anna)
        let erik_parents: Vec<_> = erik_rels.iter()
            .filter(|r| r.relationship_type == RelationshipType::Parent)
            .collect();
        assert_eq!(erik_parents.len(), 2, "Erik ska ha 2 föräldrar");
        let erik_parent_ids: Vec<i64> = erik_parents.iter().map(|r| r.other_person_id).collect();
        assert!(erik_parent_ids.contains(&karl_id), "Karl ska vara Eriks förälder");
        assert!(erik_parent_ids.contains(&maria_id), "Maria ska vara Eriks förälder");

        // Erik ska INTE ha barn
        let erik_children: Vec<_> = erik_rels.iter()
            .filter(|r| r.relationship_type == RelationshipType::Child)
            .collect();
        assert_eq!(erik_children.len(), 0, "Erik ska INTE ha barn i detta dataset");

        let erik_siblings: Vec<_> = erik_rels.iter()
            .filter(|r| r.relationship_type == RelationshipType::Sibling)
            .collect();
        assert_eq!(erik_siblings.len(), 1, "Erik ska ha 1 syskon");
        assert_eq!(erik_siblings[0].other_person_id, anna_id);

        // --- Verifiera relationer från Annas perspektiv (barn) ---
        let anna_rels = db.relationships().find_by_person_with_names(anna_id).unwrap();
        let anna_parents: Vec<_> = anna_rels.iter()
            .filter(|r| r.relationship_type == RelationshipType::Parent)
            .collect();
        assert_eq!(anna_parents.len(), 2, "Anna ska ha 2 föräldrar");

        let anna_siblings: Vec<_> = anna_rels.iter()
            .filter(|r| r.relationship_type == RelationshipType::Sibling)
            .collect();
        assert_eq!(anna_siblings.len(), 1, "Anna ska ha 1 syskon");
        assert_eq!(anna_siblings[0].other_person_id, erik_id);
    }

    #[test]
    fn test_import_full_dates_and_birth_place() {
        use chrono::NaiveDate;

        let db = Database::open_in_memory().unwrap();
        db.migrate().unwrap();

        let importer = GedcomImporter::new(&db);

        let gedcom = r#"0 HEAD
0 @I1@ INDI
1 NAME Karl /Johansson/
1 BIRT
2 DATE 12 MAR 1906
2 PLAC Lund, Malmöhus län, Sverige
1 DEAT
2 DATE 3 OCT 1985
0 @I2@ INDI
1 NAME Maria /Persson/
1 BIRT
2 DATE 8 FEB 1911
2 PLAC Stockholm
0 TRLR"#;

        let data = GedcomParser::parse_string(gedcom).unwrap();
        let result = importer.import_data(&data).unwrap();

        assert_eq!(result.persons_imported, 2);

        // Verifiera Karl
        let persons = db.persons().find_all().unwrap();
        let karl = persons.iter().find(|p| p.firstname.as_deref() == Some("Karl")).unwrap();
        assert_eq!(karl.birth_date, NaiveDate::from_ymd_opt(1906, 3, 12));
        assert_eq!(karl.death_date, NaiveDate::from_ymd_opt(1985, 10, 3));
        assert_eq!(karl.birth_place, Some("Lund, Malmöhus län, Sverige".to_string()));

        // Verifiera Maria
        let maria = persons.iter().find(|p| p.firstname.as_deref() == Some("Maria")).unwrap();
        assert_eq!(maria.birth_date, NaiveDate::from_ymd_opt(1911, 2, 8));
        assert_eq!(maria.birth_place, Some("Stockholm".to_string()));
    }
}
