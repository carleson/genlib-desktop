//! Export-tjänst för att exportera data till olika format (JSON, CSV, PDF)

use anyhow::{Context, Result};
use chrono::Utc;
use printpdf::{BuiltinFont, Mm, PdfDocument};
use serde::Serialize;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use crate::db::Database;
use crate::models::{Person, RelationshipType};

/// Exportformat
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    Csv,
    Pdf,
}

impl ExportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Json => "json",
            ExportFormat::Csv => "csv",
            ExportFormat::Pdf => "pdf",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ExportFormat::Json => "JSON",
            ExportFormat::Csv => "CSV",
            ExportFormat::Pdf => "PDF",
        }
    }
}

/// Typ av rapport
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportType {
    /// Alla personer
    AllPersons,
    /// Alla relationer
    AllRelationships,
    /// Statistiksammanfattning
    Statistics,
}

impl ReportType {
    pub fn display_name(&self) -> &'static str {
        match self {
            ReportType::AllPersons => "Alla personer",
            ReportType::AllRelationships => "Alla relationer",
            ReportType::Statistics => "Statistik",
        }
    }

    pub fn filename_prefix(&self) -> &'static str {
        match self {
            ReportType::AllPersons => "personer",
            ReportType::AllRelationships => "relationer",
            ReportType::Statistics => "statistik",
        }
    }
}

/// Exporterbar persondata (förenklad för export)
#[derive(Debug, Serialize)]
pub struct PersonExport {
    pub id: i64,
    pub firstname: Option<String>,
    pub surname: Option<String>,
    pub full_name: String,
    pub birth_date: Option<String>,
    pub death_date: Option<String>,
    pub age: Option<i32>,
    pub is_alive: bool,
    pub directory_name: String,
}

impl From<&Person> for PersonExport {
    fn from(p: &Person) -> Self {
        Self {
            id: p.id.unwrap_or(0),
            firstname: p.firstname.clone(),
            surname: p.surname.clone(),
            full_name: p.full_name(),
            birth_date: p.birth_date.map(|d| d.format("%Y-%m-%d").to_string()),
            death_date: p.death_date.map(|d| d.format("%Y-%m-%d").to_string()),
            age: p.age,
            is_alive: p.is_alive(),
            directory_name: p.directory_name.clone(),
        }
    }
}

/// Exporterbar relationsdata
#[derive(Debug, Serialize)]
pub struct RelationshipExport {
    pub id: i64,
    pub person_a_id: i64,
    pub person_a_name: String,
    pub person_b_id: i64,
    pub person_b_name: String,
    pub relationship_type: String,
}

/// Statistikexport
#[derive(Debug, Serialize)]
pub struct StatisticsExport {
    pub generated_at: String,
    pub total_persons: i64,
    pub living_persons: i64,
    pub deceased_persons: i64,
    pub total_relationships: i64,
    pub total_documents: i64,
    pub relationships_by_type: Vec<RelationshipTypeCount>,
    pub persons_by_birth_decade: Vec<DecadeCount>,
}

#[derive(Debug, Serialize)]
pub struct RelationshipTypeCount {
    pub relationship_type: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct DecadeCount {
    pub decade: String,
    pub count: i64,
}

/// Export-tjänst
pub struct ExportService<'a> {
    db: &'a Database,
}

impl<'a> ExportService<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Generera filnamn för export
    pub fn generate_filename(report_type: ReportType, format: ExportFormat) -> String {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        format!(
            "genlib_{}_{}.{}",
            report_type.filename_prefix(),
            timestamp,
            format.extension()
        )
    }

    /// Exportera rapport till fil
    pub fn export_to_file(
        &self,
        report_type: ReportType,
        format: ExportFormat,
        path: &Path,
    ) -> Result<ExportResult> {
        // PDF hanteras separat
        if format == ExportFormat::Pdf {
            return self.export_to_pdf(report_type, path);
        }

        let content = match report_type {
            ReportType::AllPersons => self.export_persons(format)?,
            ReportType::AllRelationships => self.export_relationships(format)?,
            ReportType::Statistics => self.export_statistics(format)?,
        };

        std::fs::write(path, &content).context("Kunde inte skriva fil")?;

        Ok(ExportResult {
            report_type,
            format,
            row_count: self.count_rows(report_type)?,
            file_size: content.len(),
        })
    }

    /// Exportera till PDF
    fn export_to_pdf(&self, report_type: ReportType, path: &Path) -> Result<ExportResult> {
        let title = match report_type {
            ReportType::AllPersons => "Personlista - Genlib",
            ReportType::AllRelationships => "Relationslista - Genlib",
            ReportType::Statistics => "Statistik - Genlib",
        };

        let (doc, page1, layer1) = PdfDocument::new(title, Mm(210.0), Mm(297.0), "Lager 1");
        let current_layer = doc.get_page(page1).get_layer(layer1);

        // Ladda typsnitt
        let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
        let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;

        // Sidkonfiguration
        let margin_left = Mm(20.0);
        let margin_top = Mm(280.0);
        let line_height = Mm(5.0);
        let mut y_pos = margin_top;

        // Titel
        current_layer.use_text(title, 16.0, margin_left, y_pos, &font_bold);
        y_pos = y_pos - Mm(8.0);

        // Datum
        let date_str = Utc::now().format("Genererad: %Y-%m-%d %H:%M").to_string();
        current_layer.use_text(&date_str, 10.0, margin_left, y_pos, &font);
        y_pos = y_pos - Mm(10.0);

        // Separator
        y_pos = y_pos - line_height;

        let row_count = match report_type {
            ReportType::AllPersons => {
                let persons = self.db.persons().find_all()?;

                // Kolumnrubriker
                current_layer.use_text("Namn", 10.0, margin_left, y_pos, &font_bold);
                current_layer.use_text("Född", 10.0, Mm(90.0), y_pos, &font_bold);
                current_layer.use_text("Död", 10.0, Mm(120.0), y_pos, &font_bold);
                current_layer.use_text("Ålder", 10.0, Mm(150.0), y_pos, &font_bold);
                y_pos = y_pos - line_height;

                // Data
                let mut count = 0;
                for person in &persons {
                    if y_pos < Mm(20.0) {
                        // Ny sida behövs (förenklad - hoppar över i denna implementation)
                        break;
                    }

                    let name = person.full_name();
                    let birth = person
                        .birth_date
                        .map(|d| d.format("%Y-%m-%d").to_string())
                        .unwrap_or_default();
                    let death = person
                        .death_date
                        .map(|d| d.format("%Y-%m-%d").to_string())
                        .unwrap_or_default();
                    let age = person
                        .age
                        .map(|a| a.to_string())
                        .unwrap_or_default();

                    current_layer.use_text(&name, 9.0, margin_left, y_pos, &font);
                    current_layer.use_text(&birth, 9.0, Mm(90.0), y_pos, &font);
                    current_layer.use_text(&death, 9.0, Mm(120.0), y_pos, &font);
                    current_layer.use_text(&age, 9.0, Mm(150.0), y_pos, &font);

                    y_pos = y_pos - line_height;
                    count += 1;
                }
                count
            }
            ReportType::AllRelationships => {
                let relationships = self.db.relationships().find_all()?;

                // Kolumnrubriker
                current_layer.use_text("Person A", 10.0, margin_left, y_pos, &font_bold);
                current_layer.use_text("Relation", 10.0, Mm(80.0), y_pos, &font_bold);
                current_layer.use_text("Person B", 10.0, Mm(120.0), y_pos, &font_bold);
                y_pos = y_pos - line_height;

                let mut count = 0;
                for rel in &relationships {
                    if y_pos < Mm(20.0) {
                        break;
                    }

                    let person_a = self
                        .db
                        .persons()
                        .find_by_id(rel.person_a_id)?
                        .map(|p| p.full_name())
                        .unwrap_or_default();
                    let person_b = self
                        .db
                        .persons()
                        .find_by_id(rel.person_b_id)?
                        .map(|p| p.full_name())
                        .unwrap_or_default();
                    let rel_type = rel.relationship_a_to_b.display_name();

                    current_layer.use_text(&person_a, 9.0, margin_left, y_pos, &font);
                    current_layer.use_text(rel_type, 9.0, Mm(80.0), y_pos, &font);
                    current_layer.use_text(&person_b, 9.0, Mm(120.0), y_pos, &font);

                    y_pos = y_pos - line_height;
                    count += 1;
                }
                count
            }
            ReportType::Statistics => {
                let persons = self.db.persons().find_all()?;
                let relationships = self.db.relationships().find_all()?;
                let documents_count = self.db.documents().count()?;

                let total = persons.len();
                let living = persons.iter().filter(|p| p.is_alive()).count();
                let deceased = total - living;

                let stats = [
                    ("Antal personer:", total.to_string()),
                    ("  - Levande:", living.to_string()),
                    ("  - Avlidna:", deceased.to_string()),
                    ("Antal relationer:", relationships.len().to_string()),
                    ("Antal dokument:", documents_count.to_string()),
                ];

                for (label, value) in &stats {
                    current_layer.use_text(*label, 10.0, margin_left, y_pos, &font);
                    current_layer.use_text(value.clone(), 10.0, Mm(60.0), y_pos, &font);
                    y_pos = y_pos - line_height;
                }

                1
            }
        };

        // Spara PDF
        let file = File::create(path).context("Kunde inte skapa PDF-fil")?;
        let mut writer = BufWriter::new(file);
        doc.save(&mut writer).context("Kunde inte spara PDF")?;

        let file_size = std::fs::metadata(path)?.len() as usize;

        Ok(ExportResult {
            report_type,
            format: ExportFormat::Pdf,
            row_count,
            file_size,
        })
    }

    /// Exportera personer
    fn export_persons(&self, format: ExportFormat) -> Result<String> {
        let persons = self.db.persons().find_all()?;
        let exports: Vec<PersonExport> = persons.iter().map(PersonExport::from).collect();

        match format {
            ExportFormat::Json => {
                serde_json::to_string_pretty(&exports).context("JSON serialisering misslyckades")
            }
            ExportFormat::Csv => self.persons_to_csv(&exports),
            ExportFormat::Pdf => unreachable!("PDF hanteras separat i export_to_pdf"),
        }
    }

    /// Exportera relationer
    fn export_relationships(&self, format: ExportFormat) -> Result<String> {
        let relationships = self.db.relationships().find_all()?;
        let mut exports = Vec::new();

        for rel in relationships {
            let person_a = self.db.persons().find_by_id(rel.person_a_id)?;
            let person_b = self.db.persons().find_by_id(rel.person_b_id)?;

            // Format: "Person A är [relationship_a_to_b] till Person B"
            exports.push(RelationshipExport {
                id: rel.id.unwrap_or(0),
                person_a_id: rel.person_a_id,
                person_a_name: person_a.map(|p| p.full_name()).unwrap_or_default(),
                person_b_id: rel.person_b_id,
                person_b_name: person_b.map(|p| p.full_name()).unwrap_or_default(),
                relationship_type: rel.relationship_a_to_b.display_name().to_string(),
            });
        }

        match format {
            ExportFormat::Json => {
                serde_json::to_string_pretty(&exports).context("JSON serialisering misslyckades")
            }
            ExportFormat::Csv => self.relationships_to_csv(&exports),
            ExportFormat::Pdf => unreachable!("PDF hanteras separat i export_to_pdf"),
        }
    }

    /// Exportera statistik
    fn export_statistics(&self, format: ExportFormat) -> Result<String> {
        let persons = self.db.persons().find_all()?;
        let relationships = self.db.relationships().find_all()?;

        let total_persons = persons.len() as i64;
        let living_persons = persons.iter().filter(|p| p.is_alive()).count() as i64;
        let deceased_persons = total_persons - living_persons;
        let total_relationships = relationships.len() as i64;
        let total_documents = self.db.documents().count()?;

        // Räkna relationer per typ (använder relationship_a_to_b)
        let mut rel_counts: std::collections::HashMap<RelationshipType, i64> =
            std::collections::HashMap::new();
        for rel in &relationships {
            *rel_counts.entry(rel.relationship_a_to_b).or_insert(0) += 1;
        }

        let relationships_by_type: Vec<RelationshipTypeCount> = rel_counts
            .into_iter()
            .map(|(rt, count)| RelationshipTypeCount {
                relationship_type: rt.display_name().to_string(),
                count,
            })
            .collect();

        // Räkna personer per födelsedecennium
        let mut decade_counts: std::collections::HashMap<String, i64> =
            std::collections::HashMap::new();
        for person in &persons {
            if let Some(birth_date) = person.birth_date {
                let decade = (birth_date.format("%Y").to_string().parse::<i32>().unwrap_or(0) / 10)
                    * 10;
                let decade_str = format!("{}-tal", decade);
                *decade_counts.entry(decade_str).or_insert(0) += 1;
            }
        }

        let mut persons_by_birth_decade: Vec<DecadeCount> = decade_counts
            .into_iter()
            .map(|(decade, count)| DecadeCount { decade, count })
            .collect();
        persons_by_birth_decade.sort_by(|a, b| a.decade.cmp(&b.decade));

        let stats = StatisticsExport {
            generated_at: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            total_persons,
            living_persons,
            deceased_persons,
            total_relationships,
            total_documents,
            relationships_by_type,
            persons_by_birth_decade,
        };

        match format {
            ExportFormat::Json => {
                serde_json::to_string_pretty(&stats).context("JSON serialisering misslyckades")
            }
            ExportFormat::Csv => self.statistics_to_csv(&stats),
            ExportFormat::Pdf => unreachable!("PDF hanteras separat i export_to_pdf"),
        }
    }

    /// Konvertera personer till CSV
    fn persons_to_csv(&self, persons: &[PersonExport]) -> Result<String> {
        let mut csv = String::new();

        // Header
        csv.push_str("id,firstname,surname,full_name,birth_date,death_date,age,is_alive,directory_name\n");

        // Rader
        for p in persons {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{}\n",
                p.id,
                Self::csv_escape(p.firstname.as_deref().unwrap_or("")),
                Self::csv_escape(p.surname.as_deref().unwrap_or("")),
                Self::csv_escape(&p.full_name),
                p.birth_date.as_deref().unwrap_or(""),
                p.death_date.as_deref().unwrap_or(""),
                p.age.map(|a| a.to_string()).unwrap_or_default(),
                p.is_alive,
                Self::csv_escape(&p.directory_name),
            ));
        }

        Ok(csv)
    }

    /// Konvertera relationer till CSV
    fn relationships_to_csv(&self, relationships: &[RelationshipExport]) -> Result<String> {
        let mut csv = String::new();

        // Header
        csv.push_str(
            "id,person_a_id,person_a_name,person_b_id,person_b_name,relationship_type\n",
        );

        // Rader
        for r in relationships {
            csv.push_str(&format!(
                "{},{},{},{},{},{}\n",
                r.id,
                r.person_a_id,
                Self::csv_escape(&r.person_a_name),
                r.person_b_id,
                Self::csv_escape(&r.person_b_name),
                Self::csv_escape(&r.relationship_type),
            ));
        }

        Ok(csv)
    }

    /// Konvertera statistik till CSV
    fn statistics_to_csv(&self, stats: &StatisticsExport) -> Result<String> {
        let mut csv = String::new();

        csv.push_str(&format!("Genererad,{}\n", stats.generated_at));
        csv.push_str(&format!("Totalt antal personer,{}\n", stats.total_persons));
        csv.push_str(&format!("Levande,{}\n", stats.living_persons));
        csv.push_str(&format!("Avlidna,{}\n", stats.deceased_persons));
        csv.push_str(&format!(
            "Totalt antal relationer,{}\n",
            stats.total_relationships
        ));
        csv.push_str(&format!(
            "Totalt antal dokument,{}\n",
            stats.total_documents
        ));

        csv.push_str("\nRelationer per typ\n");
        csv.push_str("Typ,Antal\n");
        for rt in &stats.relationships_by_type {
            csv.push_str(&format!("{},{}\n", rt.relationship_type, rt.count));
        }

        csv.push_str("\nPersoner per födelsedecennium\n");
        csv.push_str("Decennium,Antal\n");
        for dc in &stats.persons_by_birth_decade {
            csv.push_str(&format!("{},{}\n", dc.decade, dc.count));
        }

        Ok(csv)
    }

    /// Escape CSV-värde
    fn csv_escape(value: &str) -> String {
        if value.contains(',') || value.contains('"') || value.contains('\n') {
            format!("\"{}\"", value.replace('"', "\"\""))
        } else {
            value.to_string()
        }
    }

    /// Räkna rader för rapport
    fn count_rows(&self, report_type: ReportType) -> Result<usize> {
        Ok(match report_type {
            ReportType::AllPersons => self.db.persons().find_all()?.len(),
            ReportType::AllRelationships => self.db.relationships().find_all()?.len(),
            ReportType::Statistics => 1,
        })
    }
}

/// Resultat av export
#[derive(Debug)]
pub struct ExportResult {
    pub report_type: ReportType,
    pub format: ExportFormat,
    pub row_count: usize,
    pub file_size: usize,
}

impl ExportResult {
    pub fn summary(&self) -> String {
        format!(
            "{} exporterad: {} rader, {} bytes",
            self.report_type.display_name(),
            self.row_count,
            self.file_size
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_escape() {
        assert_eq!(ExportService::csv_escape("hello"), "hello");
        assert_eq!(ExportService::csv_escape("hello,world"), "\"hello,world\"");
        assert_eq!(
            ExportService::csv_escape("say \"hello\""),
            "\"say \"\"hello\"\"\""
        );
    }

    #[test]
    fn test_generate_filename() {
        let filename = ExportService::generate_filename(ReportType::AllPersons, ExportFormat::Json);
        assert!(filename.starts_with("genlib_personer_"));
        assert!(filename.ends_with(".json"));

        let filename = ExportService::generate_filename(ReportType::Statistics, ExportFormat::Csv);
        assert!(filename.starts_with("genlib_statistik_"));
        assert!(filename.ends_with(".csv"));
    }
}
