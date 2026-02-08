//! Tjänst för att synka checklistmallar till personer

use anyhow::{anyhow, Result};

use crate::db::Database;
use crate::models::PersonChecklistItem;

/// Resultat av synk
#[derive(Debug, Clone, Default)]
pub struct ChecklistSyncResult {
    pub persons: usize,
    pub created: usize,
    pub skipped: usize,
}

/// Tjänst för att applicera mallar
pub struct ChecklistSyncService<'a> {
    db: &'a Database,
}

impl<'a> ChecklistSyncService<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Applicera mall till en person
    pub fn apply_template_to_person(
        &self,
        template_id: i64,
        person_id: i64,
    ) -> Result<ChecklistSyncResult> {
        let template = self
            .db
            .checklists()
            .find_template_by_id(template_id)?
            .ok_or_else(|| anyhow!("Checklistmall hittades inte"))?;

        if !template.is_active {
            return Err(anyhow!("Checklistmall '{}' är inaktiv", template.name));
        }

        let items = self.db.checklists().list_template_items(template_id)?;
        let existing = self
            .db
            .checklists()
            .template_item_ids_for_person(person_id)?;

        let mut result = ChecklistSyncResult {
            persons: 1,
            created: 0,
            skipped: 0,
        };

        for item in items {
            let item_id = match item.id {
                Some(id) => id,
                None => {
                    result.skipped += 1;
                    continue;
                }
            };

            if existing.contains(&item_id) {
                result.skipped += 1;
                continue;
            }

            let mut person_item = PersonChecklistItem::from_template(person_id, &item);
            if self.db.checklists().create(&mut person_item).is_ok() {
                result.created += 1;
            }
        }

        Ok(result)
    }

    /// Applicera mall till alla personer
    pub fn apply_template_to_all(&self, template_id: i64) -> Result<ChecklistSyncResult> {
        let persons = self.db.persons().find_all()?;
        let mut result = ChecklistSyncResult {
            persons: persons.len(),
            created: 0,
            skipped: 0,
        };

        for person in persons {
            if let Some(person_id) = person.id {
                let partial = self.apply_template_to_person(template_id, person_id)?;
                result.created += partial.created;
                result.skipped += partial.skipped;
            }
        }

        Ok(result)
    }
}
