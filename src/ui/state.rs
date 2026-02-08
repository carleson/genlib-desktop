use std::collections::HashSet;

/// Aktuell vy i applikationen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    Dashboard,
    PersonList,
    PersonDetail,
    DocumentViewer,
    FamilyTree,
    Settings,
    Backup,
    SetupWizard,
    ChecklistTemplates,
    Reports,
    DocumentTemplates,
}

/// Centraliserat applikationstillstånd
#[derive(Debug, Default)]
pub struct AppState {
    /// Aktuell vy
    pub current_view: View,

    /// Vald person (för detaljvy)
    pub selected_person_id: Option<i64>,

    /// Valt dokument (för dokumentvy)
    pub selected_document_id: Option<i64>,

    /// Bokmärkta personer (cache)
    pub bookmarked_persons: HashSet<i64>,

    /// Visar personformulär
    pub show_person_form: bool,

    /// Person som redigeras (None = ny person)
    pub editing_person_id: Option<i64>,

    /// Visar relationsformulär
    pub show_relationship_form: bool,

    /// Visar dokumentuppladdningsmodal
    pub show_document_upload: bool,

    /// Läge för dokumentmodal (import eller skapa)
    pub document_upload_mode: DocumentUploadMode,

    /// Visar GEDCOM-importmodal
    pub show_gedcom_import: bool,

    /// Visar bekräftelsedialog
    pub show_confirm_dialog: bool,
    pub confirm_dialog_message: String,
    pub confirm_dialog_action: Option<ConfirmAction>,

    /// Statusmeddelande
    pub status_message: Option<StatusMessage>,

    /// Sökfråga (global)
    pub search_query: String,

    /// Dark mode
    pub dark_mode: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Navigera till vy
    pub fn navigate(&mut self, view: View) {
        self.current_view = view;
    }

    /// Navigera till persondetalj
    pub fn navigate_to_person(&mut self, person_id: i64) {
        self.selected_person_id = Some(person_id);
        self.current_view = View::PersonDetail;
    }

    /// Navigera till dokumentvy
    pub fn navigate_to_document(&mut self, document_id: i64) {
        self.selected_document_id = Some(document_id);
        self.current_view = View::DocumentViewer;
    }

    /// Öppna dokumentimport (en eller flera filer)
    pub fn open_document_import(&mut self) {
        self.document_upload_mode = DocumentUploadMode::Import;
        self.show_document_upload = true;
    }

    /// Öppna skapa nytt dokument
    pub fn open_document_create(&mut self) {
        self.document_upload_mode = DocumentUploadMode::Create;
        self.show_document_upload = true;
    }

    /// Stäng dokumentuppladdning
    pub fn close_document_upload(&mut self) {
        self.show_document_upload = false;
    }

    /// Öppna personformulär för ny person
    pub fn open_new_person_form(&mut self) {
        self.editing_person_id = None;
        self.show_person_form = true;
    }

    /// Öppna personformulär för redigering
    pub fn open_edit_person_form(&mut self, person_id: i64) {
        self.editing_person_id = Some(person_id);
        self.show_person_form = true;
    }

    /// Stäng personformulär
    pub fn close_person_form(&mut self) {
        self.show_person_form = false;
        self.editing_person_id = None;
    }

    /// Visa bekräftelsedialog
    pub fn show_confirm(&mut self, message: &str, action: ConfirmAction) {
        self.confirm_dialog_message = message.to_string();
        self.confirm_dialog_action = Some(action);
        self.show_confirm_dialog = true;
    }

    /// Stäng bekräftelsedialog
    pub fn close_confirm(&mut self) {
        self.show_confirm_dialog = false;
        self.confirm_dialog_action = None;
    }

    /// Visa statusmeddelande
    pub fn show_status(&mut self, message: &str, status_type: StatusType) {
        self.status_message = Some(StatusMessage {
            text: message.to_string(),
            status_type,
            created_at: std::time::Instant::now(),
        });
    }

    /// Visa framgångsmeddelande
    pub fn show_success(&mut self, message: &str) {
        self.show_status(message, StatusType::Success);
    }

    /// Visa felmeddelande
    pub fn show_error(&mut self, message: &str) {
        self.show_status(message, StatusType::Error);
    }

    /// Rensa statusmeddelande om det är för gammalt
    pub fn clear_old_status(&mut self) {
        if let Some(ref status) = self.status_message {
            if status.created_at.elapsed().as_secs() > 5 {
                self.status_message = None;
            }
        }
    }
}

/// Typ av bekräftelseåtgärd
#[derive(Debug, Clone)]
pub enum ConfirmAction {
    DeletePerson(i64),
    DeleteRelationship(i64),
    DeleteDocument(i64),
}

/// Statusmeddelande
#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub text: String,
    pub status_type: StatusType,
    pub created_at: std::time::Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusType {
    Success,
    Error,
    Info,
    Warning,
}

/// Läge för dokumentmodal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DocumentUploadMode {
    #[default]
    Import,
    Create,
}

/// Formulärdata för person
#[derive(Debug, Default, Clone)]
pub struct PersonFormData {
    pub firstname: String,
    pub surname: String,
    pub birth_date: String,
    pub death_date: String,
    pub directory_name: String,
}

impl PersonFormData {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn from_person(person: &crate::models::Person) -> Self {
        Self {
            firstname: person.firstname.clone().unwrap_or_default(),
            surname: person.surname.clone().unwrap_or_default(),
            birth_date: person
                .birth_date
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default(),
            death_date: person
                .death_date
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default(),
            directory_name: person.directory_name.clone(),
        }
    }
}

/// Formulärdata för relation
#[derive(Debug, Default, Clone)]
pub struct RelationshipFormData {
    pub other_person_id: Option<i64>,
    pub relationship_type: Option<crate::models::RelationshipType>,
}

impl RelationshipFormData {
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}
