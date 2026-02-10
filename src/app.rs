//! Huvudapplikation för Genlib Desktop

use eframe::egui;
use std::sync::Arc;

use crate::db::Database;
use crate::ui::{
    modals::{ConfirmDialog, DocumentUploadModal, GedcomImportModal, PersonFormModal, RelationshipFormModal},
    state::AppState,
    theme::configure_style,
    views::{
        BackupView, ChecklistSearchView, ChecklistTemplatesView, DashboardView, DocumentTemplatesView,
        DocumentViewerView, FamilyTreeView, PersonDetailView, PersonListView, ReportsView, SettingsView,
        SetupWizardView, SplashScreenView,
    },
    View,
};
use crate::utils::path::get_database_path;

/// Huvudapplikation
pub struct GenlibApp {
    db: Arc<Database>,
    state: AppState,

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

    // Modals
    person_form_modal: PersonFormModal,
    document_upload_modal: DocumentUploadModal,
    relationship_form_modal: RelationshipFormModal,
    gedcom_import_modal: GedcomImportModal,

    // Splash
    splash_screen: SplashScreenView,

    // Intern
    style_initialized: bool,
}

impl GenlibApp {
    /// Skapa ny applikation
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Konfigurera fonts
        configure_fonts(&cc.egui_ctx);

        // Öppna databas
        let db_path = get_database_path();
        tracing::info!("Öppnar databas: {:?}", db_path);

        let db = match Database::open(&db_path) {
            Ok(db) => {
                // Kör migrationer
                if let Err(e) = db.migrate() {
                    tracing::error!("Migrering misslyckades: {}", e);
                }
                Arc::new(db)
            }
            Err(e) => {
                tracing::error!("Kunde inte öppna databas: {}", e);
                // Försök med in-memory som fallback
                Arc::new(Database::open_in_memory().expect("Kunde inte skapa in-memory databas"))
            }
        };

        // Skapa initial konfiguration om den inte finns
        let setup_complete = db.config().is_setup_complete().unwrap_or(false);
        if let Ok(config) = db.config().get() {
            if setup_complete {
                let _ = config.ensure_directories();
            }
        }

        let next_view = if setup_complete {
            View::Dashboard
        } else {
            View::SetupWizard
        };

        let mut state = AppState::new();
        state.current_view = View::Splash;

        Self {
            db,
            state,
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
            person_form_modal: PersonFormModal::new(),
            document_upload_modal: DocumentUploadModal::new(),
            relationship_form_modal: RelationshipFormModal::new(),
            gedcom_import_modal: GedcomImportModal::new(),
            style_initialized: false,
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
                // Ladda dokument om ett är valt
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
        }
    }
}

impl eframe::App for GenlibApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Konfigurera stil (endast första gången eller vid ändring)
        if !self.style_initialized {
            configure_style(ctx, self.state.dark_mode);
            self.style_initialized = true;
        }

        // Rensa gamla statusmeddelanden
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

        // Topbar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Genlib");
                ui.separator();

                // Navigation
                let nav_items = [
                    (View::Dashboard, "📊 Dashboard"),
                    (View::PersonList, "👥 Personer"),
                    (View::FamilyTree, "🌳 Släktträd"),
                    (View::ChecklistSearch, "✓ Uppgifter"),
                ];

                for (view, label) in nav_items {
                    if ui
                        .selectable_label(self.state.current_view == view, label)
                        .clicked()
                    {
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
                    if ui.button(mode_icon).clicked() {
                        self.state.dark_mode = !self.state.dark_mode;
                        configure_style(ctx, self.state.dark_mode);
                    }

                    // Inställningar
                    if ui
                        .selectable_label(self.state.current_view == View::Settings, "⚙")
                        .clicked()
                    {
                        self.state.current_view = View::Settings;
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
                        self.state.current_view = View::PersonDetail;
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
                    self.settings.show(ui, &mut self.state, &self.db);
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
                View::Splash => {} // Hanteras ovan med early return
            }
        });

        // Modals
        if self.state.show_person_form {
            if self.person_form_modal.show(ctx, &mut self.state, &self.db) {
                self.state.close_person_form();
                // Uppdatera vyer
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
                    // Uppdatera vyer efter radering
                    self.person_list.mark_needs_refresh();
                    self.dashboard.mark_needs_refresh();
                    // Om vi är i persondetalj, uppdatera den (för relationer/dokument)
                    if self.state.current_view == View::PersonDetail {
                        self.person_detail.mark_needs_refresh();
                    }
                    // Om vi raderade ett dokument, gå tillbaka till persondetalj
                    if self.state.current_view == View::DocumentViewer {
                        self.state.current_view = View::PersonDetail;
                        self.person_detail.mark_needs_refresh();
                    }
                }
            }
        }

        // Dokumentuppladdning modal
        if self.state.show_document_upload {
            // Hämta aktuell person för upload
            if let Some(person_id) = self.state.selected_person_id {
                if let Ok(Some(person)) = self.db.persons().find_by_id(person_id) {
                    if self.document_upload_modal.show(ctx, &mut self.state, &self.db, &person) {
                        self.state.close_document_upload();
                        // Uppdatera persondetalj
                        self.person_detail.mark_needs_refresh();
                        self.dashboard.mark_needs_refresh();
                    }
                }
            }
        }

        // Relationsformulär modal
        if self.state.show_relationship_form {
            if let Some(person_id) = self.state.selected_person_id {
                if let Ok(Some(person)) = self.db.persons().find_by_id(person_id) {
                    if self.relationship_form_modal.show(ctx, &mut self.state, &self.db, &person) {
                        self.state.show_relationship_form = false;
                        // Uppdatera persondetalj för att visa nya relationer
                        self.person_detail.mark_needs_refresh();
                    }
                }
            }
        }

        // GEDCOM-import modal
        if self.state.show_gedcom_import {
            if self.gedcom_import_modal.show(ctx, &mut self.state, &self.db) {
                self.state.show_gedcom_import = false;
                // Uppdatera alla vyer efter import
                self.dashboard.mark_needs_refresh();
                self.person_list.mark_needs_refresh();
            }
        }
    }
}

/// Konfigurera fonts
fn configure_fonts(_ctx: &egui::Context) {
    // Använder standardfonterna som har bra Unicode-stöd
    // Om du vill använda anpassade fonts senare, lägg till dem här
}
