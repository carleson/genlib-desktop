//! Repository för checklisthantering

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
use rusqlite::{params, Connection, Row};

use crate::models::{
    ChecklistCategory, ChecklistPriority, ChecklistTemplate, ChecklistTemplateItem,
    PersonChecklistItem,
};

/// Filter för global checklistsökning
#[derive(Debug, Clone, Default)]
pub struct ChecklistSearchFilter {
    pub query: String,
    pub birth_after: Option<chrono::NaiveDate>,
    pub birth_before: Option<chrono::NaiveDate>,
    pub death_after: Option<chrono::NaiveDate>,
    pub death_before: Option<chrono::NaiveDate>,
    pub filter_alive: Option<bool>,
}

/// Resultat från global checklistsökning (item + personnamn)
#[derive(Debug, Clone)]
pub struct ChecklistSearchResult {
    pub item: PersonChecklistItem,
    pub person_name: String,
}

/// Repository för checklistobjekt
pub struct ChecklistRepository {
    conn: Arc<Mutex<Connection>>,
}

impl ChecklistRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Hämta alla checklistobjekt för en person
    pub fn find_by_person(&self, person_id: i64) -> Result<Vec<PersonChecklistItem>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, person_id, template_item_id, title, description,
                    category, priority, sort_order, is_completed, completed_at, notes
             FROM person_checklist_items
             WHERE person_id = ?
             ORDER BY sort_order, category, priority DESC",
        )?;

        let items = stmt
            .query_map([person_id], |row| Ok(Self::row_to_item(row)))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(items)
    }

    /// Hämta checklistmallar
    pub fn list_templates(&self, include_inactive: bool) -> Result<Vec<ChecklistTemplate>> {
        let conn = self.conn.lock().unwrap();
        let mut sql = String::from(
            "SELECT id, name, description, is_active FROM checklist_templates",
        );
        if !include_inactive {
            sql.push_str(" WHERE is_active = 1");
        }
        sql.push_str(" ORDER BY name");

        let mut stmt = conn.prepare(&sql)?;
        let templates = stmt
            .query_map([], |row| Ok(Self::row_to_template(row)))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(templates)
    }

    /// Hämta mall via ID
    pub fn find_template_by_id(&self, id: i64) -> Result<Option<ChecklistTemplate>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, is_active FROM checklist_templates WHERE id = ?",
        )?;

        let template = stmt
            .query_row([id], |row| Ok(Self::row_to_template(row)))
            .ok();

        Ok(template)
    }

    /// Skapa ny mall
    pub fn create_template(&self, template: &mut ChecklistTemplate) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO checklist_templates (name, description, is_active)
             VALUES (?1, ?2, ?3)",
            params![template.name, template.description, template.is_active],
        )?;

        let id = conn.last_insert_rowid();
        template.id = Some(id);
        Ok(id)
    }

    /// Uppdatera mall
    pub fn update_template(&self, template: &ChecklistTemplate) -> Result<()> {
        let id = template
            .id
            .ok_or_else(|| anyhow!("Checklist template har inget ID"))?;

        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "UPDATE checklist_templates
             SET name = ?1, description = ?2, is_active = ?3
             WHERE id = ?4",
            params![template.name, template.description, template.is_active, id],
        )?;

        if rows == 0 {
            return Err(anyhow!("Checklist template med ID {} hittades inte", id));
        }

        Ok(())
    }

    /// Ta bort mall
    pub fn delete_template(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM checklist_templates WHERE id = ?", [id])?;

        if rows == 0 {
            return Err(anyhow!("Checklist template med ID {} hittades inte", id));
        }

        Ok(())
    }

    /// Hämta items för en mall
    pub fn list_template_items(&self, template_id: i64) -> Result<Vec<ChecklistTemplateItem>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, template_id, title, description, category, priority, sort_order
             FROM checklist_template_items
             WHERE template_id = ?
             ORDER BY sort_order, category, priority DESC",
        )?;

        let items = stmt
            .query_map([template_id], |row| Ok(Self::row_to_template_item(row)))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(items)
    }

    /// Skapa nytt mall-item
    pub fn create_template_item(&self, item: &mut ChecklistTemplateItem) -> Result<i64> {
        let conn = self.conn.lock().unwrap();

        let max_order: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1)
                 FROM checklist_template_items
                 WHERE template_id = ?",
                [item.template_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        item.sort_order = (max_order + 1) as i32;

        conn.execute(
            "INSERT INTO checklist_template_items
             (template_id, title, description, category, priority, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                item.template_id,
                item.title,
                item.description,
                item.category as i32,
                item.priority as i32,
                item.sort_order,
            ],
        )?;

        let id = conn.last_insert_rowid();
        item.id = Some(id);
        Ok(id)
    }

    /// Uppdatera mall-item
    pub fn update_template_item(&self, item: &ChecklistTemplateItem) -> Result<()> {
        let id = item
            .id
            .ok_or_else(|| anyhow!("Checklist template item har inget ID"))?;

        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "UPDATE checklist_template_items SET
                title = ?1, description = ?2, category = ?3, priority = ?4, sort_order = ?5
             WHERE id = ?6",
            params![
                item.title,
                item.description,
                item.category as i32,
                item.priority as i32,
                item.sort_order,
                id,
            ],
        )?;

        if rows == 0 {
            return Err(anyhow!("Checklist template item med ID {} hittades inte", id));
        }

        Ok(())
    }

    /// Ta bort mall-item
    pub fn delete_template_item(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM checklist_template_items WHERE id = ?", [id])?;

        if rows == 0 {
            return Err(anyhow!("Checklist template item med ID {} hittades inte", id));
        }

        Ok(())
    }

    /// Hämta mall-item via ID
    pub fn find_template_item_by_id(
        &self,
        id: i64,
    ) -> Result<Option<ChecklistTemplateItem>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, template_id, title, description, category, priority, sort_order
             FROM checklist_template_items
             WHERE id = ?",
        )?;

        let item = stmt
            .query_row([id], |row| Ok(Self::row_to_template_item(row)))
            .ok();

        Ok(item)
    }

    /// Hämta template_item_id som redan finns för en person
    pub fn template_item_ids_for_person(&self, person_id: i64) -> Result<HashSet<i64>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT template_item_id
             FROM person_checklist_items
             WHERE person_id = ? AND template_item_id IS NOT NULL",
        )?;

        let ids = stmt
            .query_map([person_id], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect::<HashSet<i64>>();

        Ok(ids)
    }

    /// Hämta checklistobjekt grupperade per kategori
    pub fn find_by_person_grouped(
        &self,
        person_id: i64,
    ) -> Result<HashMap<ChecklistCategory, Vec<PersonChecklistItem>>> {
        let items = self.find_by_person(person_id)?;
        let mut grouped: HashMap<ChecklistCategory, Vec<PersonChecklistItem>> = HashMap::new();

        for item in items {
            grouped.entry(item.category).or_default().push(item);
        }

        Ok(grouped)
    }

    /// Hämta progress för en person (completed, total)
    pub fn get_progress(&self, person_id: i64) -> Result<(i64, i64)> {
        let conn = self.conn.lock().unwrap();

        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM person_checklist_items WHERE person_id = ?",
            [person_id],
            |row| row.get(0),
        )?;

        let completed: i64 = conn.query_row(
            "SELECT COUNT(*) FROM person_checklist_items WHERE person_id = ? AND is_completed = 1",
            [person_id],
            |row| row.get(0),
        )?;

        Ok((completed, total))
    }

    /// Skapa nytt checklistobjekt
    pub fn create(&self, item: &mut PersonChecklistItem) -> Result<i64> {
        let conn = self.conn.lock().unwrap();

        // Hämta nästa sort_order
        let max_order: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) FROM person_checklist_items WHERE person_id = ?",
                [item.person_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        item.sort_order = (max_order + 1) as i32;

        conn.execute(
            "INSERT INTO person_checklist_items
             (person_id, template_item_id, title, description, category, priority, sort_order, is_completed, completed_at, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                item.person_id,
                item.template_item_id,
                item.title,
                item.description,
                item.category as i32,
                item.priority as i32,
                item.sort_order,
                item.is_completed,
                item.completed_at,
                item.notes,
            ],
        )?;

        let id = conn.last_insert_rowid();
        item.id = Some(id);

        Ok(id)
    }

    /// Uppdatera checklistobjekt
    pub fn update(&self, item: &PersonChecklistItem) -> Result<()> {
        let id = item.id.ok_or_else(|| anyhow!("Checklist item har inget ID"))?;

        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "UPDATE person_checklist_items SET
                title = ?1, description = ?2, category = ?3, priority = ?4,
                sort_order = ?5, is_completed = ?6, completed_at = ?7, notes = ?8
             WHERE id = ?9",
            params![
                item.title,
                item.description,
                item.category as i32,
                item.priority as i32,
                item.sort_order,
                item.is_completed,
                item.completed_at,
                item.notes,
                id,
            ],
        )?;

        if rows == 0 {
            return Err(anyhow!("Checklist item med ID {} hittades inte", id));
        }

        Ok(())
    }

    /// Ta bort checklistobjekt
    pub fn delete(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM person_checklist_items WHERE id = ?", [id])?;

        if rows == 0 {
            return Err(anyhow!("Checklist item med ID {} hittades inte", id));
        }

        Ok(())
    }

    /// Växla completed-status
    pub fn toggle_completed(&self, id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        // Hämta nuvarande status
        let is_completed: bool = conn.query_row(
            "SELECT is_completed FROM person_checklist_items WHERE id = ?",
            [id],
            |row| row.get(0),
        )?;

        let new_status = !is_completed;
        let completed_at = if new_status {
            Some(chrono::Utc::now().to_rfc3339())
        } else {
            None
        };

        conn.execute(
            "UPDATE person_checklist_items SET is_completed = ?, completed_at = ? WHERE id = ?",
            params![new_status, completed_at, id],
        )?;

        Ok(new_status)
    }

    /// Hämta ett checklistobjekt
    pub fn find_by_id(&self, id: i64) -> Result<Option<PersonChecklistItem>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, person_id, template_item_id, title, description,
                    category, priority, sort_order, is_completed, completed_at, notes
             FROM person_checklist_items
             WHERE id = ?",
        )?;

        let item = stmt
            .query_row([id], |row| Ok(Self::row_to_item(row)))
            .ok();

        Ok(item)
    }

    /// Hämta global progress (completed, total) för alla personer
    pub fn get_global_progress(&self) -> Result<(i64, i64)> {
        let conn = self.conn.lock().unwrap();
        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM person_checklist_items",
            [],
            |row| row.get(0),
        )?;
        let completed: i64 = conn.query_row(
            "SELECT COUNT(*) FROM person_checklist_items WHERE is_completed = 1",
            [],
            |row| row.get(0),
        )?;
        Ok((completed, total))
    }

    /// Räkna checklistobjekt för en person
    pub fn count_by_person(&self, person_id: i64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM person_checklist_items WHERE person_id = ?",
            [person_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Sök checklistobjekt globalt med personnamn- och datumfilter
    pub fn search_items_with_person(&self, filter: &ChecklistSearchFilter) -> Result<Vec<ChecklistSearchResult>> {
        let conn = self.conn.lock().unwrap();
        let like_pattern = format!("%{}%", filter.query);

        let mut sql = String::from(
            "SELECT pci.id, pci.person_id, pci.template_item_id, pci.title, pci.description,
                    pci.category, pci.priority, pci.sort_order, pci.is_completed, pci.completed_at, pci.notes,
                    p.firstname, p.surname
             FROM person_checklist_items pci
             JOIN persons p ON p.id = pci.person_id
             WHERE (?1 = '' OR p.firstname LIKE ?2 OR p.surname LIKE ?2)",
        );

        if let Some(ref d) = filter.birth_after {
            sql.push_str(&format!(" AND p.birth_date >= '{}'", d.format("%Y-%m-%d")));
        }
        if let Some(ref d) = filter.birth_before {
            sql.push_str(&format!(" AND p.birth_date <= '{}'", d.format("%Y-%m-%d")));
        }
        if let Some(ref d) = filter.death_after {
            sql.push_str(&format!(" AND p.death_date >= '{}'", d.format("%Y-%m-%d")));
        }
        if let Some(ref d) = filter.death_before {
            sql.push_str(&format!(" AND p.death_date <= '{}'", d.format("%Y-%m-%d")));
        }
        if let Some(alive) = filter.filter_alive {
            if alive {
                sql.push_str(" AND p.death_date IS NULL");
            } else {
                sql.push_str(" AND p.death_date IS NOT NULL");
            }
        }

        sql.push_str(" ORDER BY pci.is_completed, p.surname, p.firstname, pci.priority DESC");

        let mut stmt = conn.prepare(&sql)?;

        let results = stmt
            .query_map(params![filter.query, like_pattern], |row| {
                let item = Self::row_to_item(row);
                let firstname: String = row.get(11).unwrap_or_default();
                let surname: String = row.get(12).unwrap_or_default();
                let person_name = match (firstname.is_empty(), surname.is_empty()) {
                    (false, false) => format!("{} {}", firstname, surname),
                    (false, true) => firstname,
                    (true, false) => surname,
                    (true, true) => "Okänd".to_string(),
                };
                Ok(ChecklistSearchResult { item, person_name })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(results)
    }

    fn row_to_item(row: &Row) -> PersonChecklistItem {
        PersonChecklistItem {
            id: row.get(0).ok(),
            person_id: row.get(1).unwrap_or(0),
            template_item_id: row.get(2).ok(),
            title: row.get(3).unwrap_or_default(),
            description: row.get(4).ok(),
            category: ChecklistCategory::from_i32(row.get(5).unwrap_or(0))
                .unwrap_or(ChecklistCategory::default()),
            priority: ChecklistPriority::from_i32(row.get(6).unwrap_or(1))
                .unwrap_or(ChecklistPriority::default()),
            sort_order: row.get(7).unwrap_or(0),
            is_completed: row.get(8).unwrap_or(false),
            completed_at: row.get(9).ok(),
            notes: row.get(10).ok(),
        }
    }

    fn row_to_template(row: &Row) -> ChecklistTemplate {
        ChecklistTemplate {
            id: row.get(0).ok(),
            name: row.get(1).unwrap_or_default(),
            description: row.get(2).ok(),
            is_active: row.get::<_, i64>(3).unwrap_or(1) != 0,
        }
    }

    fn row_to_template_item(row: &Row) -> ChecklistTemplateItem {
        ChecklistTemplateItem {
            id: row.get(0).ok(),
            template_id: row.get(1).unwrap_or(0),
            title: row.get(2).unwrap_or_default(),
            description: row.get(3).ok(),
            category: ChecklistCategory::from_i32(row.get(4).unwrap_or(0))
                .unwrap_or(ChecklistCategory::default()),
            priority: ChecklistPriority::from_i32(row.get(5).unwrap_or(1))
                .unwrap_or(ChecklistPriority::default()),
            sort_order: row.get(6).unwrap_or(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    #[test]
    fn test_create_and_find() {
        let db = Database::open_in_memory().unwrap();

        // Skapa en person först
        let mut person = crate::models::Person::default();
        person.firstname = Some("Test".to_string());
        person.directory_name = "test_person".to_string();
        db.persons().create(&mut person).unwrap();
        let person_id = person.id.unwrap();

        // Skapa checklistobjekt
        let mut item = PersonChecklistItem::new(person_id, "Hitta födelsebevis".to_string());
        item.category = ChecklistCategory::Documents;
        item.priority = ChecklistPriority::High;

        let id = db.checklists().create(&mut item).unwrap();
        assert!(id > 0);

        // Hämta
        let found = db.checklists().find_by_id(id).unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.title, "Hitta födelsebevis");
        assert_eq!(found.category, ChecklistCategory::Documents);
    }

    #[test]
    fn test_toggle_completed() {
        let db = Database::open_in_memory().unwrap();

        // Skapa person
        let mut person = crate::models::Person::default();
        person.firstname = Some("Test".to_string());
        person.directory_name = "test_toggle".to_string();
        db.persons().create(&mut person).unwrap();
        let person_id = person.id.unwrap();

        // Skapa checklistobjekt
        let mut item = PersonChecklistItem::new(person_id, "Test item".to_string());
        let id = db.checklists().create(&mut item).unwrap();

        // Toggle till completed
        let new_status = db.checklists().toggle_completed(id).unwrap();
        assert!(new_status);

        // Verifiera
        let found = db.checklists().find_by_id(id).unwrap().unwrap();
        assert!(found.is_completed);
        assert!(found.completed_at.is_some());

        // Toggle tillbaka
        let new_status = db.checklists().toggle_completed(id).unwrap();
        assert!(!new_status);
    }

    #[test]
    fn test_progress() {
        let db = Database::open_in_memory().unwrap();

        // Skapa person
        let mut person = crate::models::Person::default();
        person.firstname = Some("Test".to_string());
        person.directory_name = "test_progress".to_string();
        db.persons().create(&mut person).unwrap();
        let person_id = person.id.unwrap();

        // Skapa 3 objekt
        for i in 0..3 {
            let mut item = PersonChecklistItem::new(person_id, format!("Item {}", i));
            db.checklists().create(&mut item).unwrap();
        }

        // Progress: 0 av 3
        let (completed, total) = db.checklists().get_progress(person_id).unwrap();
        assert_eq!(completed, 0);
        assert_eq!(total, 3);

        // Markera 2 som klara
        let items = db.checklists().find_by_person(person_id).unwrap();
        db.checklists().toggle_completed(items[0].id.unwrap()).unwrap();
        db.checklists().toggle_completed(items[1].id.unwrap()).unwrap();

        // Progress: 2 av 3
        let (completed, total) = db.checklists().get_progress(person_id).unwrap();
        assert_eq!(completed, 2);
        assert_eq!(total, 3);
    }
}
