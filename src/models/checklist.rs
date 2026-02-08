use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[repr(i32)]
pub enum ChecklistCategory {
    #[default]
    Research = 0,
    Documents = 1,
    Sources = 2,
    Verification = 3,
    Other = 4,
}

impl ChecklistCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Research => "Forskning",
            Self::Documents => "Dokument",
            Self::Sources => "Källor",
            Self::Verification => "Verifiering",
            Self::Other => "Övrigt",
        }
    }

    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Research),
            1 => Some(Self::Documents),
            2 => Some(Self::Sources),
            3 => Some(Self::Verification),
            4 => Some(Self::Other),
            _ => None,
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Research,
            Self::Documents,
            Self::Sources,
            Self::Verification,
            Self::Other,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[repr(i32)]
pub enum ChecklistPriority {
    Low = 0,
    #[default]
    Medium = 1,
    High = 2,
    Critical = 3,
}

impl ChecklistPriority {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Low => "Låg",
            Self::Medium => "Medel",
            Self::High => "Hög",
            Self::Critical => "Kritisk",
        }
    }

    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Low),
            1 => Some(Self::Medium),
            2 => Some(Self::High),
            3 => Some(Self::Critical),
            _ => None,
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Low, Self::Medium, Self::High, Self::Critical]
    }
}

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
    pub description: Option<String>,
    pub category: ChecklistCategory,
    pub priority: ChecklistPriority,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonChecklistItem {
    pub id: Option<i64>,
    pub person_id: i64,
    pub template_item_id: Option<i64>,
    pub title: String,
    pub description: Option<String>,
    pub category: ChecklistCategory,
    pub priority: ChecklistPriority,
    pub sort_order: i32,
    pub is_completed: bool,
    pub completed_at: Option<String>,
    pub notes: Option<String>,
}

impl PersonChecklistItem {
    pub fn new(person_id: i64, title: String) -> Self {
        Self {
            id: None,
            person_id,
            template_item_id: None,
            title,
            description: None,
            category: ChecklistCategory::default(),
            priority: ChecklistPriority::default(),
            sort_order: 0,
            is_completed: false,
            completed_at: None,
            notes: None,
        }
    }

    pub fn from_template(person_id: i64, template_item: &ChecklistTemplateItem) -> Self {
        Self {
            id: None,
            person_id,
            template_item_id: template_item.id,
            title: template_item.title.clone(),
            description: template_item.description.clone(),
            category: template_item.category,
            priority: template_item.priority,
            sort_order: template_item.sort_order,
            is_completed: false,
            completed_at: None,
            notes: None,
        }
    }

    pub fn is_custom(&self) -> bool {
        self.template_item_id.is_none()
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
