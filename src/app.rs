//! Huvudapplikation för Genlib Desktop

use eframe::egui;
use std::sync::Arc;

use crate::db::Database;
use crate::models::config::{AppSettings, ShortcutAction};
use crate::projects::{Project, ProjectAction, ProjectRegistry};
use crate::ui::{
    modals::{ArchiveModal, ConfirmDialog, DocumentUploadModal, GedcomImportModal, PersonFormModal, RelationshipFormModal, ResourceFormModal},
    shortcuts::ShortcutManager,
    state::AppState,
    theme::configure_style,
    views::{
        BackupView, ChecklistSearchView, ChecklistTemplatesView, DashboardView, DocumentTemplatesView,
        DocumentViewerView, FamilyTreeView, PersonDetailView, PersonListView, ProjectSelectorView,
        ReportsView, ResourceDetailView, ResourceListView, SettingsView, SetupWizardView,
        SplashScreenView,
    },
    View,
};
use crate::utils::path::{get_database_path, get_default_projects_dir};

/// Huvudapplikation
pub struct GenlibApp {
    db: Arc<Database>,
    state: AppState,
    app_settings: AppSettings,
    shortcut_manager: ShortcutManager,

    // Projektstöd
    current_project: Option<Project>,
    project_registry: ProjectRegistry,
    show_project_selector: bool,
    project_selector: ProjectSelectorView,

    // Vyer
    dashboard: DashboardView,
    person_list: PersonListView,
    person_detail: PersonDetailView,
    document_viewer: DocumentViewerView,
    family_tree: FamilyTreeView,
    settings: SettingsView,
    backup_view: BackupView,
    checklist_search: ChecklistSearchView,
    checklist_templates: ChecklistTemplatesView,
    setup_wizard: SetupWizardView,
    reports_view: ReportsView,
    document_templates: DocumentTemplatesView,

    // Resurser
    resource_list: ResourceListView,
    resource_detail: ResourceDetailView,
    resource_form: ResourceFormModal,

    // Modals
    person_form_modal: PersonFormModal,
    document_upload_modal: DocumentUploadModal,
    relationship_form_modal: RelationshipFormModal,
    gedcom_import_modal: GedcomImportModal,
    archive_modal: ArchiveModal,

    // Splash
    splash_screen: SplashScreenView,

    // Intern
    style_initialized: bool,
}

impl GenlibApp {
    /// Skapa ny applikation
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_fonts(&cc.egui_ctx);

        let app_settings = AppSettings::load();
        let shortcut_manager = ShortcutManager::new(app_settings.shortcuts.clone());

        // Ladda projektregister
        let mut project_registry = ProjectRegistry::load();

        // Migration: gammal installation utan projects.toml
        if project_registry.projects.is_empty() {
            let old_db_path = get_database_path();
            if old_db_path.exists() {
                tracing::info!("Migrerar befintlig databas till projektsystemet...");
                let projects_dir = get_default_projects_dir().join("standard");
                if std::fs::create_dir_all(&projects_dir).is_ok() {
                    let new_db_path = projects_dir.join("genlib.db");
                    match std::fs::copy(&old_db_path, &new_db_path) {
                        Ok(_) => {
                            let _ = std::fs::remove_file(&old_db_path);
                            let mut project =
                                Project::new("Standard", "Migrerat projekt", projects_dir);
                            project.is_default = true;
                            project_registry.add(project);
                            let _ = project_registry.save();
                            tracing::info!("Migration klar.");
                        }
                        Err(e) => tracing::error!("Migration misslyckades: {}", e),
                    }
                }
            }
        }

        // Öppna default-projekt eller förbered projektväljar-skärm
        let (db, current_project, show_project_selector, next_view) =
            if let Some(default) = project_registry.default_project().cloned() {
                let db_path = default.db_path();
                match Database::open(&db_path) {
                    Ok(db) => {
                        if let Err(e) = db.migrate() {
                            tracing::error!("Migrering misslyckades: {}", e);
                        }
                        let setup_complete =
                            db.config().is_setup_complete().unwrap_or(false);
                        if let Ok(config) = db.config().get() {
                            if setup_complete {
                                let _ = config.ensure_directories();
                            }
                        }
                        let view = if setup_complete {
                            View::Dashboard
                        } else {
                            View::SetupWizard
                        };
                        (Arc::new(db), Some(default), false, view)
                    }
                    Err(e) => {
                        tracing::error!("Kunde inte öppna databas: {}", e);
                        let db = Database::open_in_memory()
                            .expect("Kunde inte skapa in-memory databas");
                        (Arc::new(db), None, true, View::Dashboard)
                    }
                }
            } else {
                // Inga projekt → visa projektväljar-skärm
                let db = Database::open_in_memory()
                    .expect("Kunde inte skapa in-memory databas");
                let _ = db.migrate();
                (Arc::new(db), None, true, View::Dashboard)
            };

        let mut state = AppState::new();
        state.current_view = View::Splash;
        state.dark_mode = app_settings.dark_mode;

        Self {
            db,
            state,
            app_settings,
            shortcut_manager,
            current_project,
            project_registry,
            show_project_selector,
            project_selector: ProjectSelectorView::new(),
            dashboard: DashboardView::new(),
            person_list: PersonListView::new(),
            person_detail: PersonDetailView::new(),
            document_viewer: DocumentViewerView::new(),
            family_tree: FamilyTreeView::new(),
            settings: SettingsView::new(),
            backup_view: BackupView::new(),
            checklist_search: ChecklistSearchView::new(),
            checklist_templates: ChecklistTemplatesView::new(),
            setup_wizard: SetupWizardView::new(),
            reports_view: ReportsView::new(),
            document_templates: DocumentTemplatesView::new(),
            splash_screen: SplashScreenView::new(next_view),
            resource_list: ResourceListView::new(),
            resource_detail: ResourceDetailView::new(),
            resource_form: ResourceFormModal::new(),
            person_form_modal: PersonFormModal::new(),
            document_upload_modal: DocumentUploadModal::new(),
            relationship_form_modal: RelationshipFormModal::new(),
            gedcom_import_modal: GedcomImportModal::new(),
            archive_modal: ArchiveModal::new(),
            style_initialized: false,
        }
    }

    /// Öppna ett projekt — byter ut db och återställer vy-tillståndet
    fn open_project(&mut self, project: &Project) {
        let db_path = project.db_path();
        match Database::open(&db_path) {
            Ok(db) => {
                if let Err(e) = db.migrate() {
                    tracing::error!("Migrering misslyckades: {}", e);
                }
                self.db = Arc::new(db);
            }
            Err(e) => {
                tracing::error!("Kunde inte öppna databas för projekt '{}': {}", project.name, e);
                return;
            }
        }

        self.current_project = Some(project.clone());
        self.show_project_selector = false;

        // Återställ AppState men behåll dark_mode
        let dark_mode = self.state.dark_mode;
        self.state = AppState::new();
        self.state.dark_mode = dark_mode;

        // Markera alla vyer som behöver laddas om
        self.dashboard.mark_needs_refresh();
        self.person_list.mark_needs_refresh();
        self.person_detail.mark_needs_refresh();
        self.family_tree.mark_needs_refresh();
        self.backup_view.mark_needs_refresh();
        self.checklist_search.mark_needs_refresh();
        self.checklist_templates.mark_needs_refresh();
        self.reports_view.mark_needs_refresh();
        self.document_templates.mark_needs_refresh();
        self.resource_list.mark_needs_refresh();
        self.resource_detail.mark_needs_refresh();

        let setup_complete = self.db.config().is_setup_complete().unwrap_or(false);
        if let Ok(config) = self.db.config().get() {
            if setup_complete {
                let _ = config.ensure_directories();
            }
        }
        self.state.current_view = if setup_complete {
            View::Dashboard
        } else {
            View::SetupWizard
        };
    }

    /// Hantera åtgärd från projektväljar-vyn
    fn handle_project_action(&mut self, action: ProjectAction) {
        match action {
            ProjectAction::Open(id) => {
                if let Some(project) = self.project_registry.find_by_id(&id).cloned() {
                    self.open_project(&project);
                }
            }
            ProjectAction::CreateNew {
                name,
                description,
                directory,
            } => {
                if let Err(e) = std::fs::create_dir_all(&directory) {
                    tracing::error!("Kunde inte skapa projektmapp: {}", e);
                    return;
                }
                let mut project = Project::new(&name, &description, directory);
                if self.project_registry.projects.is_empty() {
                    project.is_default = true;
                }
                let project_id = project.id.clone();
                self.project_registry.add(project);
                if let Err(e) = self.project_registry.save() {
                    tracing::error!("Kunde inte spara projektregister: {}", e);
                }
                if let Some(project) = self.project_registry.find_by_id(&project_id).cloned() {
                    self.open_project(&project);
                }
            }
            ProjectAction::Delete(id) => {
                self.project_registry.remove(&id);
                let _ = self.project_registry.save();
                // Om det aktiva projektet togs bort, visa projektväljar-skärmen
                if self.current_project.as_ref().map(|p| &p.id) == Some(&id) {
                    self.current_project = None;
                    self.show_project_selector = true;
                }
                if self.project_registry.projects.is_empty() {
                    self.show_project_selector = true;
                }
            }
            ProjectAction::SetDefault(id) => {
                self.project_registry.set_default(&id);
                let _ = self.project_registry.save();
            }
            ProjectAction::Rename(id, new_name) => {
                self.project_registry.rename(&id, &new_name);
                let _ = self.project_registry.save();
                // Uppdatera current_project om det är det aktiva
                if self.current_project.as_ref().map(|p| &p.id) == Some(&id) {
                    self.current_project =
                        self.project_registry.find_by_id(&id).cloned();
                }
            }
        }
    }

    /// Hantera navigation och uppdatera relevanta vyer
    fn handle_view_change(&mut self, new_view: View) {
        match new_view {
            View::Splash => {}
            View::Dashboard => self.dashboard.mark_needs_refresh(),
            View::PersonList => self.person_list.mark_needs_refresh(),
            View::PersonDetail => self.person_detail.mark_needs_refresh(),
            View::DocumentViewer => {
                if let Some(doc_id) = self.state.selected_document_id {
                    self.document_viewer.load_document(doc_id, &self.db);
                }
            }
            View::Settings => {}
            View::FamilyTree => self.family_tree.mark_needs_refresh(),
            View::Backup => self.backup_view.mark_needs_refresh(),
            View::SetupWizard => {}
            View::ChecklistSearch => self.checklist_search.mark_needs_refresh(),
            View::ChecklistTemplates => self.checklist_templates.mark_needs_refresh(),
            View::Reports => self.reports_view.mark_needs_refresh(),
            View::DocumentTemplates => self.document_templates.mark_needs_refresh(),
            View::ResourceList => self.resource_list.mark_needs_refresh(),
            View::ResourceDetail => self.resource_detail.mark_needs_refresh(),
        }
    }

    fn any_modal_open(&self) -> bool {
        self.state.show_person_form
            || self.state.show_confirm_dialog
            || self.state.show_document_upload
            || self.state.show_relationship_form
            || self.state.show_gedcom_import
            || self.state.show_archive_modal
            || self.state.show_resource_form
    }

    fn close_topmost_modal(&mut self) {
        if self.state.show_confirm_dialog {
            self.state.close_confirm();
        } else if self.state.show_person_form {
            self.state.close_person_form();
        } else if self.state.show_resource_form {
            self.state.close_resource_form();
        } else if self.state.show_document_upload {
            self.state.close_document_upload();
        } else if self.state.show_relationship_form {
            self.state.show_relationship_form = false;
        } else if self.state.show_gedcom_import {
            self.state.show_gedcom_import = false;
        } else if self.state.show_archive_modal {
            self.state.show_archive_modal = false;
        }
    }

    fn navigate_to(&mut self, view: View) {
        let old = self.state.current_view;
        self.state.current_view = view;
        if old != view {
            self.handle_view_change(view);
        }
    }

    fn handle_shortcut_action(&mut self, action: ShortcutAction, ctx: &egui::Context) {
        let modal_open = self.any_modal_open();

        if modal_open {
            if action == ShortcutAction::CloseModal {
                self.close_topmost_modal();
            }
            return;
        }

        match action {
            ShortcutAction::NavigateDashboard => self.navigate_to(View::Dashboard),
            ShortcutAction::NavigatePersonList => self.navigate_to(View::PersonList),
            ShortcutAction::NavigateFamilyTree => self.navigate_to(View::FamilyTree),
            ShortcutAction::NavigateChecklistSearch => self.navigate_to(View::ChecklistSearch),
            ShortcutAction::NavigateSettings => self.navigate_to(View::Settings),
            ShortcutAction::NavigateResourceList => self.navigate_to(View::ResourceList),
            ShortcutAction::NewPerson => {
                self.state.open_new_person_form();
            }
            ShortcutAction::FocusSearch => {
                if self.state.current_view != View::PersonList {
                    self.navigate_to(View::PersonList);
                }
                self.state.focus_search = true;
            }
            ShortcutAction::Backup => {
                self.navigate_to(View::Backup);
            }
            ShortcutAction::CloseModal => {}
            ShortcutAction::ToggleDarkMode => {
                self.state.dark_mode = !self.state.dark_mode;
                configure_style(ctx, self.state.dark_mode);
            }
            ShortcutAction::HistoryBack => {
                if let Some(person_id) = self.state.history_back() {
                    self.state.navigate_to_person(person_id);
                    self.person_detail.mark_needs_refresh();
                }
            }
            ShortcutAction::HistoryForward => {
                if let Some(person_id) = self.state.history_forward() {
                    self.state.navigate_to_person(person_id);
                    self.person_detail.mark_needs_refresh();
                }
            }
        }
    }
}

impl eframe::App for GenlibApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.style_initialized {
            configure_style(ctx, self.state.dark_mode);
            self.style_initialized = true;
        }

        self.state.clear_old_status();

        // Splash — rendera utan topbar/statusbar
        if self.state.current_view == View::Splash {
            egui::CentralPanel::default().show(ctx, |ui| {
                if self.splash_screen.show(ctx, ui) {
                    let next = self.splash_screen.next_view();
                    self.state.current_view = next;
                    self.handle_view_change(next);
                }
            });
            return;
        }

        // Projektväljar-skärm — rendera utan topbar/statusbar
        if self.show_project_selector {
            egui::CentralPanel::default().show(ctx, |ui| {
                if let Some(action) = self.project_selector.show(ui, &self.project_registry) {
                    self.handle_project_action(action);
                }
            });
            return;
        }

        // Kortkommandon
        if let Some(action) = self.shortcut_manager.check(ctx, self.state.capturing_shortcut) {
            self.handle_shortcut_action(action, ctx);
        }

        // Applicera nya genvägar från inställningsvyn
        if let Some(new_shortcuts) = self.state.shortcuts_to_apply.take() {
            self.shortcut_manager.update_shortcuts(new_shortcuts.clone());
            self.app_settings.shortcuts = new_shortcuts;
            let _ = self.app_settings.save();
        }

        // Topbar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Genlib");
                ui.separator();

                let nav_items = [
                    (View::Dashboard, "📊 Dashboard", ShortcutAction::NavigateDashboard),
                    (View::PersonList, "👥 Personer", ShortcutAction::NavigatePersonList),
                    (View::FamilyTree, "🌳 Släktträd", ShortcutAction::NavigateFamilyTree),
                    (View::ChecklistSearch, "✓ Uppgifter", ShortcutAction::NavigateChecklistSearch),
                    (View::ResourceList, "📍 Resurser", ShortcutAction::NavigateResourceList),
                ];

                for (view, label, shortcut_action) in nav_items {
                    let hint = self
                        .shortcut_manager
                        .shortcut_hint(shortcut_action)
                        .unwrap_or_default();
                    let response =
                        ui.selectable_label(self.state.current_view == view, label);
                    if !hint.is_empty() {
                        response.clone().on_hover_text(&hint);
                    }
                    if response.clicked() {
                        let old_view = self.state.current_view;
                        self.state.current_view = view;
                        if old_view != view {
                            self.handle_view_change(view);
                        }
                    }
                }

                // Höger sida
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Dark mode toggle
                    let mode_icon = if self.state.dark_mode { "🌙" } else { "☀" };
                    if ui
                        .button(mode_icon)
                        .on_hover_text(
                            self.shortcut_manager
                                .shortcut_hint(ShortcutAction::ToggleDarkMode)
                                .unwrap_or_default(),
                        )
                        .clicked()
                    {
                        self.state.dark_mode = !self.state.dark_mode;
                        configure_style(ctx, self.state.dark_mode);
                    }

                    // Inställningar
                    if ui
                        .selectable_label(self.state.current_view == View::Settings, "⚙")
                        .on_hover_text(
                            self.shortcut_manager
                                .shortcut_hint(ShortcutAction::NavigateSettings)
                                .unwrap_or_default(),
                        )
                        .clicked()
                    {
                        self.state.current_view = View::Settings;
                    }

                    ui.separator();

                    // Projektbytesknapp
                    if let Some(ref p) = self.current_project {
                        if ui
                            .button(format!("📁 {}", p.name))
                            .on_hover_text("Byt projekt")
                            .clicked()
                        {
                            self.show_project_selector = true;
                            self.project_selector.reset();
                        }
                    }

                    ui.separator();
                    ui.label(
                        egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                            .small()
                            .weak(),
                    );
                });
            });
        });

        // Statusbar
        if let Some(ref status) = self.state.status_message {
            egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                let color = match status.status_type {
                    crate::ui::StatusType::Success => crate::ui::theme::Colors::SUCCESS,
                    crate::ui::StatusType::Error => crate::ui::theme::Colors::ERROR,
                    crate::ui::StatusType::Warning => crate::ui::theme::Colors::WARNING,
                    crate::ui::StatusType::Info => crate::ui::theme::Colors::INFO,
                };
                ui.colored_label(color, &status.text);
            });
        }

        // Huvudinnehåll
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.state.current_view {
                View::Dashboard => {
                    self.dashboard.show(ui, &mut self.state, &self.db);
                }
                View::PersonList => {
                    if self.person_list.show(ui, &mut self.state, &self.db) {
                        self.person_detail.mark_needs_refresh();
                    }
                }
                View::PersonDetail => {
                    self.person_detail.show(ui, &mut self.state, &self.db);
                }
                View::DocumentViewer => {
                    self.document_viewer.show(ui, &mut self.state, &self.db);
                }
                View::FamilyTree => {
                    self.family_tree.show(ui, &mut self.state, &self.db);
                }
                View::Settings => {
                    self.settings.show(ui, &mut self.state, &self.db, &self.app_settings);
                }
                View::Backup => {
                    self.backup_view.show(ui, &mut self.state, &self.db);
                }
                View::SetupWizard => {
                    self.setup_wizard.show(ui, &mut self.state, &self.db);
                }
                View::ChecklistSearch => {
                    self.checklist_search.show(ui, &mut self.state, &self.db);
                }
                View::ChecklistTemplates => {
                    self.checklist_templates.show(ui, &mut self.state, &self.db);
                }
                View::Reports => {
                    self.reports_view.show(ui, &mut self.state, &self.db);
                }
                View::DocumentTemplates => {
                    self.document_templates.show(ui, &mut self.state, &self.db);
                }
                View::ResourceList => {
                    self.resource_list.show(ui, &mut self.state, &self.db);
                }
                View::ResourceDetail => {
                    self.resource_detail.show(ui, &mut self.state, &self.db);
                }
                View::Splash => {}
            }
        });

        // Modals
        if self.state.show_person_form {
            if self.person_form_modal.show(ctx, &mut self.state, &self.db) {
                self.state.close_person_form();
                self.person_list.mark_needs_refresh();
                self.dashboard.mark_needs_refresh();
                if self.state.current_view == View::PersonDetail {
                    self.person_detail.mark_needs_refresh();
                }
            }
        }

        if self.state.show_confirm_dialog {
            if let Some(confirmed) = ConfirmDialog::show(ctx, &mut self.state, &self.db) {
                if confirmed {
                    self.person_list.mark_needs_refresh();
                    self.dashboard.mark_needs_refresh();
                    if self.state.current_view == View::PersonDetail {
                        self.person_detail.mark_needs_refresh();
                    }
                    if self.state.current_view == View::DocumentViewer {
                        self.state.current_view = View::PersonDetail;
                        self.person_detail.mark_needs_refresh();
                    }
                    if self.state.current_view == View::ResourceDetail {
                        self.resource_detail.mark_needs_refresh();
                    }
                    self.resource_list.mark_needs_refresh();
                }
            }
        }

        if self.state.show_document_upload {
            if let Some(person_id) = self.state.selected_person_id {
                if let Ok(Some(person)) = self.db.persons().find_by_id(person_id) {
                    if self.document_upload_modal.show(ctx, &mut self.state, &self.db, &person) {
                        self.state.close_document_upload();
                        self.person_detail.mark_needs_refresh();
                        self.dashboard.mark_needs_refresh();
                    }
                }
            }
        }

        if self.state.show_relationship_form {
            if let Some(person_id) = self.state.selected_person_id {
                if let Ok(Some(person)) = self.db.persons().find_by_id(person_id) {
                    if self.relationship_form_modal.show(ctx, &mut self.state, &self.db, &person) {
                        self.state.show_relationship_form = false;
                        self.person_detail.mark_needs_refresh();
                    }
                }
            }
        }

        if self.state.show_gedcom_import {
            if self.gedcom_import_modal.show(ctx, &mut self.state, &self.db) {
                self.state.show_gedcom_import = false;
                self.dashboard.mark_needs_refresh();
                self.person_list.mark_needs_refresh();
            }
        }

        if self.state.show_archive_modal {
            if self.archive_modal.show(ctx, &mut self.state, &self.db) {
                self.dashboard.mark_needs_refresh();
                self.person_list.mark_needs_refresh();
            }
        }

        if self.state.show_resource_form {
            if self.resource_form.show(ctx, &mut self.state, &self.db) {
                self.resource_list.mark_needs_refresh();
                self.dashboard.mark_needs_refresh();
                if self.state.current_view == View::ResourceDetail {
                    self.resource_detail.mark_needs_refresh();
                }
            }
        }
    }
}

fn configure_fonts(_ctx: &egui::Context) {}
