//! Rapportvy för export av data

use egui::{self, RichText};

use crate::db::Database;
use crate::services::export::{ExportFormat, ExportService, ReportType};
use crate::ui::{
    state::AppState,
    theme::{Colors, Icons},
    View,
};

pub struct ReportsView {
    /// Vald rapporttyp
    selected_report: ReportType,
    /// Valt exportformat
    selected_format: ExportFormat,
    /// Senaste exportresultat (meddelande)
    last_result: Option<String>,
    /// Statistik-cache
    stats_cache: Option<StatsCache>,
    /// Behöver refresh
    needs_refresh: bool,
}

struct StatsCache {
    total_persons: i64,
    living_persons: i64,
    deceased_persons: i64,
    total_relationships: i64,
    total_documents: i64,
}

impl Default for ReportsView {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportsView {
    pub fn new() -> Self {
        Self {
            selected_report: ReportType::AllPersons,
            selected_format: ExportFormat::Json,
            last_result: None,
            stats_cache: None,
            needs_refresh: true,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) {
        // Refresh statistik om nödvändigt
        if self.needs_refresh {
            self.refresh_stats(db);
            self.needs_refresh = false;
        }

        // Header
        ui.horizontal(|ui| {
            if ui
                .button(format!("{} Tillbaka", Icons::ARROW_LEFT))
                .clicked()
            {
                state.navigate(View::Settings);
            }
            ui.separator();
            ui.heading(format!("{} Rapporter & Export", Icons::DOCUMENT));
        });

        ui.add_space(16.0);

        // Statistiköversikt
        self.show_statistics_overview(ui);

        ui.add_space(16.0);

        // Export-sektion
        self.show_export_section(ui, state, db);

        ui.add_space(16.0);

        // Senaste resultat
        if let Some(ref result) = self.last_result {
            egui::Frame::none()
                .fill(Colors::SUCCESS.gamma_multiply(0.2))
                .rounding(8.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(Icons::CHECK).color(Colors::SUCCESS));
                        ui.label(result);
                    });
                });
        }
    }

    fn show_statistics_overview(&self, ui: &mut egui::Ui) {
        egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .rounding(8.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.heading("Databasöversikt");
                ui.add_space(8.0);

                if let Some(ref stats) = self.stats_cache {
                    ui.columns(5, |columns| {
                        Self::stat_card(&mut columns[0], Icons::PEOPLE, "Personer", stats.total_persons);
                        Self::stat_card(&mut columns[1], Icons::HEART, "Levande", stats.living_persons);
                        Self::stat_card(&mut columns[2], "†", "Avlidna", stats.deceased_persons);
                        Self::stat_card(&mut columns[3], Icons::LINK, "Relationer", stats.total_relationships);
                        Self::stat_card(&mut columns[4], Icons::DOCUMENT, "Dokument", stats.total_documents);
                    });
                } else {
                    ui.label("Laddar statistik...");
                }
            });
    }

    fn stat_card(ui: &mut egui::Ui, icon: &str, label: &str, value: i64) {
        egui::Frame::none()
            .fill(ui.visuals().faint_bg_color)
            .rounding(4.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(icon).size(20.0));
                    ui.label(RichText::new(value.to_string()).size(24.0).strong());
                    ui.label(RichText::new(label).small().color(Colors::TEXT_SECONDARY));
                });
            });
    }

    fn show_export_section(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) {
        egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .rounding(8.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.heading("Exportera data");
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    // Rapporttyp
                    ui.label("Rapport:");
                    egui::ComboBox::from_id_salt("report_type")
                        .selected_text(self.selected_report.display_name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.selected_report,
                                ReportType::AllPersons,
                                ReportType::AllPersons.display_name(),
                            );
                            ui.selectable_value(
                                &mut self.selected_report,
                                ReportType::AllRelationships,
                                ReportType::AllRelationships.display_name(),
                            );
                            ui.selectable_value(
                                &mut self.selected_report,
                                ReportType::Statistics,
                                ReportType::Statistics.display_name(),
                            );
                        });

                    ui.separator();

                    // Format
                    ui.label("Format:");
                    egui::ComboBox::from_id_salt("export_format")
                        .selected_text(self.selected_format.display_name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.selected_format,
                                ExportFormat::Json,
                                ExportFormat::Json.display_name(),
                            );
                            ui.selectable_value(
                                &mut self.selected_format,
                                ExportFormat::Csv,
                                ExportFormat::Csv.display_name(),
                            );
                            ui.selectable_value(
                                &mut self.selected_format,
                                ExportFormat::Pdf,
                                ExportFormat::Pdf.display_name(),
                            );
                        });
                });

                ui.add_space(12.0);

                // Beskrivning av vald rapport
                let description = match self.selected_report {
                    ReportType::AllPersons => "Exporterar alla personer med namn, datum, ålder och anteckningar.",
                    ReportType::AllRelationships => "Exporterar alla relationer mellan personer med namn och relationstyp.",
                    ReportType::Statistics => "Exporterar en sammanfattning med statistik om databasen.",
                };
                ui.label(RichText::new(description).small().color(Colors::TEXT_SECONDARY));

                ui.add_space(12.0);

                // Export-knapp
                if ui
                    .button(format!("{} Exportera till fil", Icons::DOWNLOAD))
                    .clicked()
                {
                    self.do_export(state, db);
                }
            });
    }

    fn do_export(&mut self, state: &mut AppState, db: &Database) {
        // Generera filnamn
        let filename = ExportService::generate_filename(self.selected_report, self.selected_format);

        // Öppna fildialog för att välja var filen ska sparas
        let file_dialog = rfd::FileDialog::new()
            .set_file_name(&filename)
            .add_filter(
                self.selected_format.display_name(),
                &[self.selected_format.extension()],
            );

        if let Some(path) = file_dialog.save_file() {
            let export_service = ExportService::new(db);

            match export_service.export_to_file(self.selected_report, self.selected_format, &path) {
                Ok(result) => {
                    let msg = format!(
                        "{} sparad till {}",
                        result.summary(),
                        path.display()
                    );
                    self.last_result = Some(msg.clone());
                    state.show_success(&msg);
                }
                Err(e) => {
                    state.show_error(&format!("Export misslyckades: {}", e));
                }
            }
        }
    }

    fn refresh_stats(&mut self, db: &Database) {
        let total_persons = db.persons().count().unwrap_or(0);
        let all_persons = db.persons().find_all().unwrap_or_default();
        let living_persons = all_persons.iter().filter(|p| p.is_alive()).count() as i64;
        let deceased_persons = total_persons - living_persons;
        let total_relationships = db.relationships().find_all().map(|r| r.len()).unwrap_or(0) as i64;
        let total_documents = db.documents().count().unwrap_or(0);

        self.stats_cache = Some(StatsCache {
            total_persons,
            living_persons,
            deceased_persons,
            total_relationships,
            total_documents,
        });
    }

    pub fn mark_needs_refresh(&mut self) {
        self.needs_refresh = true;
    }
}
