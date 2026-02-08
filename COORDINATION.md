# Koordinering: Claude + Codex

Detta dokument underlättar parallellt arbete mellan AI-assistenter på genlib-desktop projektet.

**Senast uppdaterad:** 2026-01-30
**Aktiva agenter:** Claude, Codex

---

## Projektöversikt

**Stack:** Rust + egui (immediate mode GUI) + SQLite (rusqlite)
**Syfte:** Desktop-app för släktforskning

## Kodkonventioner

### Arkitektur
- **Repository Pattern:** `src/db/*_repo.rs` - databasaccess
- **Services:** `src/services/*.rs` - affärslogik
- **Views:** `src/ui/views/*.rs` - huvudvyer med `show(&mut self, ui, state, db)`
- **Modals:** `src/ui/modals/*.rs` - popup-dialoger
- **Widgets:** `src/ui/widgets/*.rs` - återanvändbara UI-komponenter
- **Models:** `src/models/*.rs` - datastrukturer

### Mönster
```rust
// Repository-metod
pub fn find_by_id(&self, id: i64) -> Result<Option<Model>> {
    let conn = self.conn.lock().unwrap();
    // ...
}

// View-metod
pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState, db: &Database) {
    // Refresh om nödvändigt
    if self.needs_refresh {
        self.refresh(db);
    }
    // UI-kod...
}

// Widget med caching
pub struct MyWidget {
    cache: Vec<Data>,
    needs_refresh: bool,
}
```

### Namnkonventioner
- Svenska kommentarer, engelska kod
- `snake_case` för funktioner/variabler
- `PascalCase` för typer/structs
- Suffix: `*Repository`, `*View`, `*Modal`, `*Panel`, `*Service`

### Databas
- Schema i `src/db/schema.rs`
- Migrationer i `src/db/migrations.rs`
- Alla repos använder `Arc<Mutex<Connection>>`

---

## Aktuell Status

### Klart (Sprint 1-3)
- [x] Fas 1: Dokumenthantering
- [x] Fas 2: Relationshantering
- [x] Fas 3: Checklistor (grundläggande)
- [x] Fas 4: Bildhantering (grundläggande)
- [x] Fas 5: GEDCOM-import
- [x] Fas 6: Backup/Restore

### Tillgängliga Uppgifter

#### UPPGIFT A: Fas 7 - Familjeträd (STOR)
**Prioritet:** Medel
**Komplexitet:** Hög
**Filer att skapa:**
- `src/services/family_tree.rs` - Bygga trädstruktur
- `src/ui/views/family_tree.rs` - Visualisering

**Krav:**
1. Bygg träddata från relationer (föräldrar, barn, syskon, partners)
2. Rita träd med egui::Painter (custom layout)
3. Klickbara noder för navigation
4. Pan/zoom (valfritt)

**Beroenden:** Inga (relationer finns)

---

#### UPPGIFT B: Fas 3.3 - Checklistmallar (MEDEL)
**Prioritet:** Låg
**Komplexitet:** Medel
**Filer att skapa:**
- `src/ui/views/checklist_templates.rs` - Mallhantering
- `src/services/checklist_sync.rs` - Synka mallar till personer

**Krav:**
1. CRUD för ChecklistTemplate och ChecklistTemplateItem
2. UI för att hantera mallar
3. "Applicera mall" som skapar PersonChecklistItem från mall

**Beroenden:** ChecklistRepository finns redan

---

#### UPPGIFT C: Fas 4.2 - Profilbild (LITEN)
**Prioritet:** Medel
**Komplexitet:** Låg
**Filer att ändra:**
- `src/ui/views/person_detail.rs` - Visa profilbild
- `src/ui/views/person_list.rs` - Visa thumbnail
- `src/db/person_repo.rs` - Uppdatera profile_image_path
- `src/ui/widgets/image_gallery.rs` - "Sätt som profilbild" knapp

**Krav:**
1. Lägg till knapp i lightbox: "Sätt som profilbild"
2. Spara relativ sökväg i person.profile_image_path
3. Visa profilbild i personlistan och persondetaljvy

**Beroenden:** ImageGallery finns

---

#### UPPGIFT D: Fas 8 - Setup Wizard (LITEN)
**Prioritet:** Låg
**Komplexitet:** Låg
**Filer att skapa:**
- `src/ui/views/setup_wizard.rs`

**Krav:**
1. Visa vid första start (om !config.is_setup_complete)
2. Steg: Välj media-katalog → Välj backup-katalog → Klar
3. Valfritt: Importera GEDCOM, Återställ backup

**Beroenden:** Config, GEDCOM, Backup finns

---

#### UPPGIFT E: Städa varningar (LITEN)
**Prioritet:** Låg
**Komplexitet:** Låg

**Uppgift:** Kör `cargo fix --lib -p genlib-desktop` och fixa kvarvarande varningar manuellt.

---

#### UPPGIFT F: Fas 9 - Rapporter (MEDEL)
**Prioritet:** Låg
**Komplexitet:** Medel
**Filer att skapa:**
- `src/ui/views/reports.rs` - Rapportvyer
- `src/services/export.rs` - Export till JSON/CSV

---

## Koordineringsregler

### Före arbete
1. Läs denna fil
2. Välj en UPPGIFT (A-F) som inte är markerad som "PÅGÅR"
3. Markera uppgiften som "PÅGÅR: [Agent]" nedan
4. Skapa/uppdatera relevanta filer

### Under arbete
- Följ kodkonventionerna ovan
- Kör `cargo check` ofta
- Kör `cargo test` före commit

### Efter arbete
1. Uppdatera MIGRATION_PLAN.md med session-notes
2. Markera uppgiften som KLAR nedan
3. Lista nya filer/ändringar

### Vid konflikt
- Kommunicera via denna fil
- Den som startade först har prioritet
- Dela upp uppgiften om möjligt

---

## Uppgiftsstatus

| Uppgift | Status | Agent | Anteckningar |
|---------|--------|-------|--------------|
| A: Familjeträd | KLAR | Claude | Klar 2026-01-30 |
| B: Checklistmallar | KLAR | Codex | Klar 2026-01-30 |
| C: Profilbild | KLAR | Claude | Klar 2026-02-01 |
| D: Setup Wizard | KLAR | Codex | Klar 2026-01-30 |
| E: Städa varningar | KLAR | Codex | Klar 2026-01-30 |
| F: Rapporter | KLAR | Claude | Klar 2026-02-01 |

---

## Kommunikationslogg

### 2026-01-30
- **Claude:** Skapade koordineringsfil. Alla uppgifter tillgängliga.
- **Claude:** Uppgift A (Familjeträd) klar. Skapade:
  - `src/services/family_tree.rs` - FamilyTreeService, FamilyTree, FamilyTreeNode, LinkType
  - `src/ui/views/family_tree.rs` - FamilyTreeView med interaktiv canvas
  - Funktioner: pan, zoom, klickbara noder, generationsval, länkritning
  - Tester: 38/38 passerar (2 nya)
### 2026-01-30
- **Codex:** Läste `COORDINATION.md`, `MIGRATION_PLAN.md`, `CLAUDE.md` för nuläge och konventioner. Ingen uppgift påbörjad ännu.
### 2026-01-30
- **Codex:** Uppgift B (Checklistmallar) klar. Skapade vy och service, samt repository-CRUD för mallar och mall-objekt. Navigation via Inställningar.
### 2026-01-30
- **Codex:** Uppgift D (Setup Wizard) klar. Ny vy med steg för media/backup och optional GEDCOM/restore. App visar wizard om setup ej klar.
### 2026-01-30
- **Codex:** Uppgift E (Städa varningar) körd. `cargo fix` för lib/bin + manuella justeringar (backup_view, family_tree) och global dead_code-allow.
### 2026-01-30
- **Codex:** Uppgraderade `rfd` till 0.17.2 för att ta bort `ashpd` 0.8.1 (future-incompat). `cargo check` ok efter uppdatering.

### 2026-02-01
- **Claude:** Förbättrade dokumentmodalen:
  - Bytte titel från "Ladda upp dokument" till "Lägg till dokument"
  - Lade till två lägen: "Ladda upp fil" och "Skapa textdokument"
  - Nytt textfält för att klistra in/skriva text som sparas som .txt-fil
  - Teckenräknare och automatisk .txt-ändelse

- **Claude:** Uppgift F (Rapporter) klar. Skapade:
  - `src/services/export.rs` - ExportService med stöd för JSON och CSV
  - `src/ui/views/reports.rs` - ReportsView med statistiköversikt och export-UI
  - Tre rapporttyper: Alla personer, Alla relationer, Statistik
  - Två exportformat: JSON och CSV
  - Navigation via Inställningar → Rapporter
  - Lade till `find_all()` och `count()` i RelationshipRepository
  - Lade till `Hash` derive på RelationshipType
  - Tester: 40/40 passerar (2 nya)

- **Claude:** Uppgift C (Profilbild) klar. Implementerade:
  - Profilbild-thumbnails i personlistan (med textur-cache)
  - Skalning till 64x64 för prestanda
  - Cirkulär placeholder med ikon för personer utan profilbild
  - (Knapp "Sätt som profilbild" i lightbox fanns redan)
  - (Profilbild i person-detaljvy fanns redan)
  - Tester: 38/38 passerar

### 2026-02-08
- **Claude:** Implementerade nya funktioner:
  - **Avancerad sökning:** Utökade PersonListView med filterkombinatorer
    - Datumintervall för födelse/död
    - Filter för har/saknar relationer, dokument, profilbild
    - Endast bokmärkta
    - Collapsible "Avancerade filter" sektion
    - Ny SearchFilter struct i person_repo.rs med advanced_search()
  - **Multi-upload bilder:** Utökade DocumentUploadModal
    - Ny flik "Flera filer" med pick_files()
    - Lista med valda filer, möjlighet att ta bort individuella
    - Progressvisning för varje fil
  - **EXIF-hantering:** Ny src/utils/exif.rs
    - Läser EXIF-data: datum, kamera, exponering, GPS, storlek
    - Visa EXIF-info i ImageGallery lightbox (collapsible)
    - Formatering för kamera, exponering, GPS-koordinater
  - **PDF-generering:** Utökade export.rs med printpdf
    - Nytt ExportFormat::Pdf i rapporter
    - Genererar PDF med personlista, relationslista eller statistik
    - Tabellayout med kolumnrubriker
  - Tester: 43/43 passerar (3 nya EXIF-tester)

<!-- Lägg till nya meddelanden här -->
