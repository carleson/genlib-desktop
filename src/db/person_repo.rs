use anyhow::{anyhow, Result};
use chrono::NaiveDate;
use rusqlite::{params, Connection, Row};
use std::sync::{Arc, Mutex};

use crate::models::Person;

/// Avancerade sökfilter för personlistan
#[derive(Default, Clone)]
pub struct SearchFilter {
    /// Fritextsökning (namn, katalog, anteckningar)
    pub query: String,
    /// Filtrera på levande/avlidna (None = alla)
    pub filter_alive: Option<bool>,
    /// Född efter detta datum
    pub birth_after: Option<NaiveDate>,
    /// Född före detta datum
    pub birth_before: Option<NaiveDate>,
    /// Död efter detta datum
    pub death_after: Option<NaiveDate>,
    /// Död före detta datum
    pub death_before: Option<NaiveDate>,
    /// Har minst en relation
    pub has_relations: Option<bool>,
    /// Har minst ett dokument
    pub has_documents: Option<bool>,
    /// Har profilbild
    pub has_profile_image: Option<bool>,
    /// Endast bokmärkta
    pub only_bookmarked: bool,
}

impl SearchFilter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Kolla om några avancerade filter är aktiva
    pub fn has_advanced_filters(&self) -> bool {
        self.birth_after.is_some()
            || self.birth_before.is_some()
            || self.death_after.is_some()
            || self.death_before.is_some()
            || self.has_relations.is_some()
            || self.has_documents.is_some()
            || self.has_profile_image.is_some()
            || self.only_bookmarked
    }

    /// Återställ alla filter
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Återställ endast avancerade filter (behåll query och filter_alive)
    pub fn reset_advanced(&mut self) {
        self.birth_after = None;
        self.birth_before = None;
        self.death_after = None;
        self.death_before = None;
        self.has_relations = None;
        self.has_documents = None;
        self.has_profile_image = None;
        self.only_bookmarked = false;
    }
}

pub struct PersonRepository {
    conn: Arc<Mutex<Connection>>,
}

impl PersonRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Hämta alla personer
    pub fn find_all(&self) -> Result<Vec<Person>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, firstname, surname, birth_date, death_date, age,
                    directory_name, profile_image_path, created_at, updated_at
             FROM persons
             ORDER BY surname, firstname"
        )?;

        let persons = stmt
            .query_map([], |row| Ok(Self::row_to_person(row)))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(persons)
    }

    /// Hämta person via ID
    pub fn find_by_id(&self, id: i64) -> Result<Option<Person>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, firstname, surname, birth_date, death_date, age,
                    directory_name, profile_image_path, created_at, updated_at
             FROM persons
             WHERE id = ?"
        )?;

        let person = stmt
            .query_row([id], |row| Ok(Self::row_to_person(row)))
            .ok();

        Ok(person)
    }

    /// Hämta person via katalognamn
    pub fn find_by_directory(&self, directory_name: &str) -> Result<Option<Person>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, firstname, surname, birth_date, death_date, age,
                    directory_name, profile_image_path, created_at, updated_at
             FROM persons
             WHERE directory_name = ?"
        )?;

        let person = stmt
            .query_row([directory_name], |row| Ok(Self::row_to_person(row)))
            .ok();

        Ok(person)
    }

    /// Sök personer (enkel sökning)
    pub fn search(&self, query: &str, filter_alive: Option<bool>) -> Result<Vec<Person>> {
        let filter = SearchFilter {
            query: query.to_string(),
            filter_alive,
            ..Default::default()
        };
        self.advanced_search(&filter)
    }

    /// Avancerad sökning med flera filterkriterier
    pub fn advanced_search(&self, filter: &SearchFilter) -> Result<Vec<Person>> {
        let conn = self.conn.lock().unwrap();

        let mut sql = String::from(
            "SELECT DISTINCT p.id, p.firstname, p.surname, p.birth_date, p.death_date, p.age,
                    p.directory_name, p.profile_image_path, p.created_at, p.updated_at
             FROM persons p"
        );

        // Join för bokmärken om filtret är aktivt
        if filter.only_bookmarked {
            sql.push_str(" INNER JOIN bookmarked_persons bp ON p.id = bp.person_id");
        }

        sql.push_str(" WHERE 1=1");

        let mut param_index = 1;
        let mut params_vec: Vec<String> = Vec::new();

        // Fritextsökning
        if !filter.query.is_empty() {
            sql.push_str(&format!(
                " AND (p.firstname LIKE ?{0} OR p.surname LIKE ?{0} OR p.directory_name LIKE ?{0})",
                param_index
            ));
            params_vec.push(format!("%{}%", filter.query));
            param_index += 1;
        }

        // Levande/avliden
        if let Some(alive) = filter.filter_alive {
            if alive {
                sql.push_str(" AND p.death_date IS NULL");
            } else {
                sql.push_str(" AND p.death_date IS NOT NULL");
            }
        }

        // Födelsedatum efter
        if let Some(date) = filter.birth_after {
            sql.push_str(&format!(" AND p.birth_date >= ?{}", param_index));
            params_vec.push(date.to_string());
            param_index += 1;
        }

        // Födelsedatum före
        if let Some(date) = filter.birth_before {
            sql.push_str(&format!(" AND p.birth_date <= ?{}", param_index));
            params_vec.push(date.to_string());
            param_index += 1;
        }

        // Dödsdatum efter
        if let Some(date) = filter.death_after {
            sql.push_str(&format!(" AND p.death_date >= ?{}", param_index));
            params_vec.push(date.to_string());
            param_index += 1;
        }

        // Dödsdatum före
        if let Some(date) = filter.death_before {
            sql.push_str(&format!(" AND p.death_date <= ?{}", param_index));
            params_vec.push(date.to_string());
        }

        // Har relationer
        if let Some(has_rel) = filter.has_relations {
            if has_rel {
                sql.push_str(
                    " AND (EXISTS (SELECT 1 FROM person_relationships WHERE person_a_id = p.id OR person_b_id = p.id))"
                );
            } else {
                sql.push_str(
                    " AND NOT EXISTS (SELECT 1 FROM person_relationships WHERE person_a_id = p.id OR person_b_id = p.id)"
                );
            }
        }

        // Har dokument
        if let Some(has_doc) = filter.has_documents {
            if has_doc {
                sql.push_str(" AND EXISTS (SELECT 1 FROM documents WHERE person_id = p.id)");
            } else {
                sql.push_str(" AND NOT EXISTS (SELECT 1 FROM documents WHERE person_id = p.id)");
            }
        }

        // Har profilbild
        if let Some(has_img) = filter.has_profile_image {
            if has_img {
                sql.push_str(" AND p.profile_image_path IS NOT NULL");
            } else {
                sql.push_str(" AND p.profile_image_path IS NULL");
            }
        }

        sql.push_str(" ORDER BY p.surname, p.firstname");

        let mut stmt = conn.prepare(&sql)?;

        // Bind parameters dynamiskt
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec
            .iter()
            .map(|s| s as &dyn rusqlite::ToSql)
            .collect();

        let persons: Vec<Person> = stmt
            .query_map(rusqlite::params_from_iter(params_refs), |row| {
                Ok(Self::row_to_person(row))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(persons)
    }

    /// Skapa ny person
    pub fn create(&self, person: &mut Person) -> Result<i64> {
        person.validate()?;
        person.calculate_age();

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO persons (firstname, surname, birth_date, death_date, age,
                                  directory_name, profile_image_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                person.firstname,
                person.surname,
                person.birth_date.map(|d| d.to_string()),
                person.death_date.map(|d| d.to_string()),
                person.age,
                person.directory_name,
                person.profile_image_path,
            ],
        )?;

        let id = conn.last_insert_rowid();
        person.id = Some(id);

        Ok(id)
    }

    /// Uppdatera person
    pub fn update(&self, person: &mut Person) -> Result<()> {
        let id = person.id.ok_or_else(|| anyhow!("Person har inget ID"))?;
        person.validate()?;
        person.calculate_age();

        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "UPDATE persons SET
                firstname = ?1, surname = ?2, birth_date = ?3, death_date = ?4,
                age = ?5, directory_name = ?6, profile_image_path = ?7,
                updated_at = datetime('now')
             WHERE id = ?8",
            params![
                person.firstname,
                person.surname,
                person.birth_date.map(|d| d.to_string()),
                person.death_date.map(|d| d.to_string()),
                person.age,
                person.directory_name,
                person.profile_image_path,
                id,
            ],
        )?;

        if rows == 0 {
            return Err(anyhow!("Person med ID {} hittades inte", id));
        }

        Ok(())
    }

    /// Ta bort person
    pub fn delete(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM persons WHERE id = ?", [id])?;

        if rows == 0 {
            return Err(anyhow!("Person med ID {} hittades inte", id));
        }

        Ok(())
    }

    /// Kontrollera om katalognamn är unikt
    pub fn is_directory_name_unique(&self, directory_name: &str, exclude_id: Option<i64>) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        let count: i64 = if let Some(id) = exclude_id {
            conn.query_row(
                "SELECT COUNT(*) FROM persons WHERE directory_name = ? AND id != ?",
                params![directory_name, id],
                |row| row.get(0),
            )?
        } else {
            conn.query_row(
                "SELECT COUNT(*) FROM persons WHERE directory_name = ?",
                [directory_name],
                |row| row.get(0),
            )?
        };

        Ok(count == 0)
    }

    /// Generera unikt katalognamn
    pub fn generate_unique_directory_name(&self, base_name: &str) -> Result<String> {
        let base = Person::sanitize_directory_name(base_name);

        if self.is_directory_name_unique(&base, None)? {
            return Ok(base);
        }

        // Lägg till suffix
        for i in 2..1000 {
            let name = format!("{}_{}", base, i);
            if self.is_directory_name_unique(&name, None)? {
                return Ok(name);
            }
        }

        Err(anyhow!("Kunde inte generera unikt katalognamn"))
    }

    /// Räkna antal personer
    pub fn count(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM persons", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Hämta bokmärkta personer
    pub fn get_bookmarked(&self) -> Result<Vec<Person>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT p.id, p.firstname, p.surname, p.birth_date, p.death_date, p.age,
                    p.directory_name, p.profile_image_path, p.created_at, p.updated_at
             FROM persons p
             INNER JOIN bookmarked_persons bp ON p.id = bp.person_id
             ORDER BY p.surname, p.firstname"
        )?;

        let persons = stmt
            .query_map([], |row| Ok(Self::row_to_person(row)))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(persons)
    }

    /// Lägg till/ta bort bokmärke
    pub fn toggle_bookmark(&self, person_id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        let is_bookmarked: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM bookmarked_persons WHERE person_id = ?)",
                [person_id],
                |row| row.get(0),
            )?;

        if is_bookmarked {
            conn.execute("DELETE FROM bookmarked_persons WHERE person_id = ?", [person_id])?;
            Ok(false)
        } else {
            conn.execute("INSERT INTO bookmarked_persons (person_id) VALUES (?)", [person_id])?;
            Ok(true)
        }
    }

    /// Kontrollera om person är bokmärkt
    pub fn is_bookmarked(&self, person_id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let is_bookmarked: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM bookmarked_persons WHERE person_id = ?)",
            [person_id],
            |row| row.get(0),
        )?;
        Ok(is_bookmarked)
    }

    /// Uppdatera profilbild för person
    pub fn set_profile_image(&self, person_id: i64, image_path: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "UPDATE persons SET profile_image_path = ?, updated_at = datetime('now') WHERE id = ?",
            params![image_path, person_id],
        )?;

        if rows == 0 {
            return Err(anyhow!("Person med ID {} hittades inte", person_id));
        }

        Ok(())
    }

    fn row_to_person(row: &Row) -> Person {
        Person {
            id: row.get(0).ok(),
            firstname: row.get(1).ok(),
            surname: row.get(2).ok(),
            birth_date: row
                .get::<_, Option<String>>(3)
                .ok()
                .flatten()
                .and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
            death_date: row
                .get::<_, Option<String>>(4)
                .ok()
                .flatten()
                .and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
            age: row.get(5).ok(),
            directory_name: row.get(6).unwrap_or_default(),
            profile_image_path: row.get(7).ok(),
            created_at: row.get(8).ok(),
            updated_at: row.get(9).ok(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    fn setup_db() -> Database {
        Database::open_in_memory().unwrap()
    }

    #[test]
    fn test_create_and_find() {
        let db = setup_db();
        let repo = db.persons();

        let mut person = Person::new(
            Some("Johan".into()),
            Some("Andersson".into()),
            "johan_andersson".into(),
        );

        let id = repo.create(&mut person).unwrap();
        assert!(id > 0);

        let found = repo.find_by_id(id).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().firstname, Some("Johan".into()));
    }

    #[test]
    fn test_search() {
        let db = setup_db();
        let repo = db.persons();

        let mut p1 = Person::new(Some("Johan".into()), Some("Andersson".into()), "johan_a".into());
        let mut p2 = Person::new(Some("Maria".into()), Some("Andersson".into()), "maria_a".into());
        let mut p3 = Person::new(Some("Erik".into()), Some("Svensson".into()), "erik_s".into());

        repo.create(&mut p1).unwrap();
        repo.create(&mut p2).unwrap();
        repo.create(&mut p3).unwrap();

        let results = repo.search("Andersson", None).unwrap();
        assert_eq!(results.len(), 2);

        let results = repo.search("Johan", None).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_bookmark() {
        let db = setup_db();
        let repo = db.persons();

        let mut person = Person::new(Some("Test".into()), None, "test".into());
        let id = repo.create(&mut person).unwrap();

        assert!(!repo.is_bookmarked(id).unwrap());

        let is_now_bookmarked = repo.toggle_bookmark(id).unwrap();
        assert!(is_now_bookmarked);
        assert!(repo.is_bookmarked(id).unwrap());

        let is_still_bookmarked = repo.toggle_bookmark(id).unwrap();
        assert!(!is_still_bookmarked);
        assert!(!repo.is_bookmarked(id).unwrap());
    }
}
