use anyhow::{anyhow, Result};
use rusqlite::{params, Connection, Row};
use std::sync::{Arc, Mutex};

use crate::models::{PersonRelationship, RelationshipType, RelationshipView};

pub struct RelationshipRepository {
    conn: Arc<Mutex<Connection>>,
}

impl RelationshipRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Hämta alla relationer
    pub fn find_all(&self) -> Result<Vec<PersonRelationship>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, person_a_id, person_b_id, relationship_a_to_b, relationship_b_to_a, notes, created_at
             FROM person_relationships
             ORDER BY id"
        )?;

        let rels = stmt
            .query_map([], |row| Ok(Self::row_to_relationship(row)))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(rels)
    }

    /// Räkna alla relationer
    pub fn count(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM person_relationships",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Hämta alla relationer för en person
    pub fn find_by_person(&self, person_id: i64) -> Result<Vec<PersonRelationship>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, person_a_id, person_b_id, relationship_a_to_b, relationship_b_to_a, notes, created_at
             FROM person_relationships
             WHERE person_a_id = ? OR person_b_id = ?"
        )?;

        let rels = stmt
            .query_map(params![person_id, person_id], |row| Ok(Self::row_to_relationship(row)))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(rels)
    }

    /// Hämta relationer med namn (för visning)
    pub fn find_by_person_with_names(&self, person_id: i64) -> Result<Vec<RelationshipView>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT r.id, r.person_a_id, r.person_b_id, r.relationship_a_to_b, r.relationship_b_to_a,
                    p.id as other_id, p.firstname, p.surname
             FROM person_relationships r
             JOIN persons p ON (
                 (r.person_a_id = ?1 AND p.id = r.person_b_id) OR
                 (r.person_b_id = ?1 AND p.id = r.person_a_id)
             )
             WHERE r.person_a_id = ?1 OR r.person_b_id = ?1"
        )?;

        let views: Vec<RelationshipView> = stmt
            .query_map([person_id], |row| {
                let rel_id: i64 = row.get(0)?;
                let person_a_id: i64 = row.get(1)?;
                let _person_b_id: i64 = row.get(2)?;
                let rel_a_to_b: i32 = row.get(3)?;
                let rel_b_to_a: i32 = row.get(4)?;
                let other_id: i64 = row.get(5)?;
                let firstname: Option<String> = row.get(6).ok();
                let surname: Option<String> = row.get(7).ok();

                // Bestäm relationstyp från perspektivet av person_id
                // rel_a_to_b = vad person_a är för person_b
                // rel_b_to_a = vad person_b är för person_a
                // Om jag är person_a, vill jag se vad person_b är för mig (rel_b_to_a)
                // Om jag är person_b, vill jag se vad person_a är för mig (rel_a_to_b)
                let relationship_type = if person_id == person_a_id {
                    RelationshipType::from_i32(rel_b_to_a).unwrap_or(RelationshipType::Sibling)
                } else {
                    RelationshipType::from_i32(rel_a_to_b).unwrap_or(RelationshipType::Sibling)
                };

                let other_name = match (firstname, surname) {
                    (Some(f), Some(s)) => format!("{} {}", f, s),
                    (Some(f), None) => f,
                    (None, Some(s)) => s,
                    (None, None) => "Okänd".to_string(),
                };

                Ok(RelationshipView {
                    relationship_id: rel_id,
                    other_person_id: other_id,
                    other_person_name: other_name,
                    relationship_type,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(views)
    }

    /// Hämta relationer grupperade per typ
    pub fn find_by_person_grouped(&self, person_id: i64) -> Result<Vec<(RelationshipType, Vec<RelationshipView>)>> {
        let views = self.find_by_person_with_names(person_id)?;

        let mut grouped: Vec<(RelationshipType, Vec<RelationshipView>)> = Vec::new();

        for rel_type in RelationshipType::all() {
            let type_views: Vec<RelationshipView> = views
                .iter()
                .filter(|v| v.relationship_type == *rel_type)
                .cloned()
                .collect();

            if !type_views.is_empty() {
                grouped.push((*rel_type, type_views));
            }
        }

        Ok(grouped)
    }

    /// Hämta relation via ID
    pub fn find_by_id(&self, id: i64) -> Result<Option<PersonRelationship>> {
        let conn = self.conn.lock().unwrap();
        let result = conn
            .query_row(
                "SELECT id, person_a_id, person_b_id, relationship_a_to_b, relationship_b_to_a, notes, created_at
                 FROM person_relationships WHERE id = ?",
                [id],
                |row| Ok(Self::row_to_relationship(row)),
            )
            .ok();

        Ok(result)
    }

    /// Kontrollera om relation redan finns
    pub fn exists(&self, person_1_id: i64, person_2_id: i64) -> Result<bool> {
        let (a, b) = if person_1_id < person_2_id {
            (person_1_id, person_2_id)
        } else {
            (person_2_id, person_1_id)
        };

        let conn = self.conn.lock().unwrap();
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM person_relationships WHERE person_a_id = ? AND person_b_id = ?)",
            params![a, b],
            |row| row.get(0),
        )?;

        Ok(exists)
    }

    /// Skapa relation
    pub fn create(&self, rel: &mut PersonRelationship) -> Result<i64> {
        // Kontrollera att relation inte redan finns
        if self.exists(rel.person_a_id, rel.person_b_id)? {
            return Err(anyhow!("Relation mellan dessa personer finns redan"));
        }

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO person_relationships (person_a_id, person_b_id, relationship_a_to_b, relationship_b_to_a, notes)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                rel.person_a_id,
                rel.person_b_id,
                rel.relationship_a_to_b as i32,
                rel.relationship_b_to_a as i32,
                rel.notes,
            ],
        )?;

        let id = conn.last_insert_rowid();
        rel.id = Some(id);

        Ok(id)
    }

    /// Ta bort relation
    pub fn delete(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM person_relationships WHERE id = ?", [id])?;
        Ok(())
    }

    /// Ta bort alla relationer för en person
    pub fn delete_by_person(&self, person_id: i64) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "DELETE FROM person_relationships WHERE person_a_id = ? OR person_b_id = ?",
            params![person_id, person_id],
        )?;
        Ok(rows)
    }

    /// Hämta föräldrar till en person
    pub fn get_parents(&self, person_id: i64) -> Result<Vec<RelationshipView>> {
        let all = self.find_by_person_with_names(person_id)?;
        Ok(all
            .into_iter()
            .filter(|v| v.relationship_type == RelationshipType::Parent)
            .collect())
    }

    /// Hämta barn till en person
    pub fn get_children(&self, person_id: i64) -> Result<Vec<RelationshipView>> {
        let all = self.find_by_person_with_names(person_id)?;
        Ok(all
            .into_iter()
            .filter(|v| v.relationship_type == RelationshipType::Child)
            .collect())
    }

    /// Hämta make/maka
    pub fn get_spouses(&self, person_id: i64) -> Result<Vec<RelationshipView>> {
        let all = self.find_by_person_with_names(person_id)?;
        Ok(all
            .into_iter()
            .filter(|v| v.relationship_type == RelationshipType::Spouse)
            .collect())
    }

    /// Hämta syskon
    pub fn get_siblings(&self, person_id: i64) -> Result<Vec<RelationshipView>> {
        let all = self.find_by_person_with_names(person_id)?;
        Ok(all
            .into_iter()
            .filter(|v| v.relationship_type == RelationshipType::Sibling)
            .collect())
    }

    fn row_to_relationship(row: &Row) -> PersonRelationship {
        PersonRelationship {
            id: row.get(0).ok(),
            person_a_id: row.get(1).unwrap_or(0),
            person_b_id: row.get(2).unwrap_or(0),
            relationship_a_to_b: RelationshipType::from_i32(row.get(3).unwrap_or(4))
                .unwrap_or(RelationshipType::Sibling),
            relationship_b_to_a: RelationshipType::from_i32(row.get(4).unwrap_or(4))
                .unwrap_or(RelationshipType::Sibling),
            notes: row.get(5).ok(),
            created_at: row.get(6).ok(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::models::Person;

    fn setup_db() -> Database {
        Database::open_in_memory().unwrap()
    }

    #[test]
    fn test_create_relationship() {
        let db = setup_db();

        // Skapa två personer
        let mut parent = Person::new(Some("Far".into()), Some("Farsson".into()), "far".into());
        let mut child = Person::new(Some("Barn".into()), Some("Barnsson".into()), "barn".into());

        db.persons().create(&mut parent).unwrap();
        db.persons().create(&mut child).unwrap();

        // Skapa relation: parent är förälder till child
        let mut rel = PersonRelationship::new(
            parent.id.unwrap(),
            child.id.unwrap(),
            RelationshipType::Parent,
        );

        let id = db.relationships().create(&mut rel).unwrap();
        assert!(id > 0);

        // Hämta relationer för child
        let child_rels = db.relationships().find_by_person_with_names(child.id.unwrap()).unwrap();
        assert_eq!(child_rels.len(), 1);
        assert_eq!(child_rels[0].relationship_type, RelationshipType::Parent);
        assert_eq!(child_rels[0].other_person_name, "Far Farsson");
    }

    #[test]
    fn test_no_duplicate_relations() {
        let db = setup_db();

        let mut p1 = Person::new(Some("Person".into()), Some("Ett".into()), "p1".into());
        let mut p2 = Person::new(Some("Person".into()), Some("Två".into()), "p2".into());

        db.persons().create(&mut p1).unwrap();
        db.persons().create(&mut p2).unwrap();

        let mut rel1 = PersonRelationship::new(p1.id.unwrap(), p2.id.unwrap(), RelationshipType::Sibling);
        db.relationships().create(&mut rel1).unwrap();

        // Försök skapa samma relation igen
        let mut rel2 = PersonRelationship::new(p2.id.unwrap(), p1.id.unwrap(), RelationshipType::Sibling);
        let result = db.relationships().create(&mut rel2);

        assert!(result.is_err());
    }
}
