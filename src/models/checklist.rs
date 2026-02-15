use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistTemplate {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
}

impl ChecklistTemplate {
    pub fn new(name: String) -> Self {
        Self {
            id: None,
            name,
            description: None,
            is_active: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistTemplateItem {
    pub id: Option<i64>,
    pub template_id: i64,
    pub title: String,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonChecklistItem {
    pub id: Option<i64>,
    pub person_id: i64,
    pub template_item_id: Option<i64>,
    pub title: String,
    pub sort_order: i32,
    pub is_completed: bool,
    pub completed_at: Option<String>,
}

impl PersonChecklistItem {
    pub fn new(person_id: i64, title: String) -> Self {
        Self {
            id: None,
            person_id,
            template_item_id: None,
            title,
            sort_order: 0,
            is_completed: false,
            completed_at: None,
        }
    }

    pub fn from_template(person_id: i64, template_item: &ChecklistTemplateItem) -> Self {
        Self {
            id: None,
            person_id,
            template_item_id: template_item.id,
            title: template_item.title.clone(),
            sort_order: template_item.sort_order,
            is_completed: false,
            completed_at: None,
        }
    }

    pub fn toggle(&mut self) {
        self.is_completed = !self.is_completed;
        if self.is_completed {
            self.completed_at = Some(chrono::Utc::now().to_rfc3339());
        } else {
            self.completed_at = None;
        }
    }
}
