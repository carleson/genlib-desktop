# Migreringsplan: Django → Rust

Detta dokument beskriver stegen för att migrera all funktionalitet från Django-projektet (genlib) till Rust desktop-applikationen (genlib-desktop).

**Senast uppdaterad:** 2026-01-30
**Status:** Sprint 4 klar

---

## Översikt

### Redan implementerat i Rust
- [x] Person CRUD (skapa, visa, redigera, radera)
- [x] Personsökning och filtrering
- [x] Bokmärken
- [x] Relationer (visning, kanonisk ordning)
- [x] Dashboard med statistik
- [x] Databasschema och migrationer
- [x] Dark/light mode
- [x] Grundläggande dokumentvisning (i persondetaljvy)
- [x] Dokumenthantering (uppladdning, visning, redigering, synkronisering)
- [x] Relationshantering (skapa, radera)
- [x] GEDCOM-import
- [x] Backup/Restore
- [x] Checklistor (grundläggande)
- [x] Bildhantering med galleri (grundläggande)
- [x] Familjeträd (interaktiv visualisering)

### Kvar att migrera
- [ ] Checklistor (mallar)
- [ ] Bildhantering (profilbild, multi-upload, EXIF)
- [ ] Setup Wizard
- [ ] Rapporter och export

---

## Fas 1: Dokumenthantering
**Prioritet:** Hög
**Beroenden:** Inga (grundläggande funktionalitet)
**Estimerad omfattning:** Medel

### 1.1 Dokumentuppladdning
**Vad:** Modal för att ladda upp dokument till en person.

**Funktioner:**
- Välj fil(er) via fildialogruta (rfd crate)
- Välj dokumenttyp från dropdown
- Kopiera fil till rätt katalog (`{media}/persons/{dir}/{target_directory}/`)
- Skapa Document-post i databas
- Visa framstegsindikator vid uppladdning

**Filer att skapa/ändra:**
- `src/ui/modals/document_upload.rs` - Ny modal
- `src/ui/modals/mod.rs` - Lägg till export
- `src/db/document_repo.rs` - Redan finns, ev. utöka
- `src/utils/file_ops.rs` - Ny fil för filoperationer

**Tekniska detaljer:**
```rust
// Kopiera fil och skapa post
fn upload_document(person: &Person, doc_type: &DocumentType, source_path: &Path) -> Result<Document>
```

### 1.2 Dokumentvisning
**Vad:** Visa dokument i applikationen (bilder, PDF, text).

**Funktioner:**
- Visa bilder inline med egui (via egui_extras/image)
- Visa textfiler (.txt, .md) med TextEdit
- PDF: Öppna i extern applikation (eller embedded om möjligt)
- Metadata-panel (storlek, typ, datum)

**Filer att skapa/ändra:**
- `src/ui/views/document_viewer.rs` - Ny vy
- `src/ui/views/mod.rs` - Lägg till export
- `src/utils/file_ops.rs` - Läs textfiler

### 1.3 Dokumentredigering
**Vad:** Redigera dokumentmetadata och textinnehåll.

**Funktioner:**
- Redigera textfiler inline
- Byta dokumenttyp
- Döpa om fil
- Ta bort dokument (fil + databaspost)

**Filer att skapa/ändra:**
- `src/ui/modals/document_edit.rs` - Modal för metadata
- `src/ui/views/document_viewer.rs` - Inline-redigering för text

### 1.4 Dokumentsynkronisering
**Vad:** Synka filsystem med databas (som PersonDocumentSyncView i Django).

**Funktioner:**
- Skanna personens katalog rekursivt
- Hitta nya filer → skapa Document-poster
- Hitta raderade filer → ta bort Document-poster
- Uppdatera metadata för ändrade filer
- Visa rapport: X tillagda, Y borttagna, Z uppdaterade

**Filer att skapa/ändra:**
- `src/services/document_sync.rs` - Ny modul
- `src/services/mod.rs` - Ny katalog för tjänster

---

## Fas 2: Relationshantering
**Prioritet:** Hög
**Beroenden:** Inga
**Estimerad omfattning:** Liten

### 2.1 Skapa relation
**Vad:** Modal för att lägga till relation mellan personer.

**Funktioner:**
- Dropdown för att välja "andra personen"
- Dropdown för relationstyp (förälder, barn, make/maka, syskon)
- Automatisk reciprok relation
- Validering (ingen dublett, inte sig själv)

**Filer att skapa/ändra:**
- `src/ui/modals/relationship_form.rs` - Ny modal
- `src/db/relationship_repo.rs` - Lägg till `create()`

### 2.2 Ta bort relation
**Vad:** Ta bort en befintlig relation.

**Funktioner:**
- Bekräftelsedialog
- Ta bort från databas

**Filer att ändra:**
- `src/ui/views/person_detail.rs` - Lägg till delete-knapp
- `src/db/relationship_repo.rs` - Lägg till `delete()`

---

## Fas 3: Checklistor
**Prioritet:** Medel
**Beroenden:** Inga
**Estimerad omfattning:** Medel

### 3.1 Visa checklistor
**Vad:** Visa checklistobjekt för en person.

**Funktioner:**
- Lista checklistobjekt grupperade per kategori
- Visa progress (X av Y klara)
- Filtrera på kategori och status
- Visa prioritet med färgkodning

**Filer att skapa/ändra:**
- `src/ui/views/checklist_view.rs` - Ny vy
- `src/db/checklist_repo.rs` - Ny repository

### 3.2 Hantera checklistobjekt
**Vad:** Skapa, redigera, bocka av checklistobjekt.

**Funktioner:**
- Toggle completion (checkbox)
- Skapa anpassat objekt
- Redigera titel/beskrivning
- Ta bort objekt

**Filer att skapa/ändra:**
- `src/ui/modals/checklist_item_form.rs` - Modal
- `src/db/checklist_repo.rs` - CRUD-metoder

### 3.3 Checklistmallar
**Vad:** Hantera mallar som synkas till personer.

**Funktioner:**
- Lista mallar
- Skapa/redigera mall
- Synka mall till alla personer

**Filer att skapa/ändra:**
- `src/ui/views/checklist_templates.rs` - Mallhantering
- `src/services/checklist_sync.rs` - Synkronisering

---

## Fas 4: Bildhantering
**Prioritet:** Medel
**Beroenden:** Fas 1 (Dokumenthantering)
**Estimerad omfattning:** Medel

### 4.1 Bildgalleri
**Vad:** Visa alla bilder för en person i ett galleri.

**Funktioner:**
- Thumbnail-grid
- Klicka för fullstorlek
- Lightbox-läge

**Filer att skapa/ändra:**
- `src/ui/widgets/image_gallery.rs` - Galleri-widget
- `src/ui/modals/image_lightbox.rs` - Fullskärmsvisning

### 4.2 Profilbild
**Vad:** Sätta och visa profilbild för person.

**Funktioner:**
- Välj bild som profilbild
- Visa profilbild i personlistan
- Visa i persondetaljvy

**Filer att ändra:**
- `src/ui/views/person_detail.rs` - Visa profilbild
- `src/ui/views/person_list.rs` - Visa thumbnail
- `src/db/person_repo.rs` - Uppdatera profile_image_path

### 4.3 Bilduppladdning (multi)
**Vad:** Ladda upp flera bilder samtidigt.

**Funktioner:**
- Välj flera filer
- Drag-and-drop (om möjligt med egui)
- Visa förhandsgranskning före uppladdning

**Filer att skapa/ändra:**
- `src/ui/modals/image_upload.rs` - Utökad modal

### 4.4 EXIF-hantering (valfri)
**Vad:** Läsa och visa EXIF-metadata från bilder.

**Funktioner:**
- Läs EXIF-data (datum, kamera, plats)
- Visa i dokumentvy

**Filer att skapa/ändra:**
- `src/utils/exif.rs` - EXIF-hantering (använd kamadak-exif crate)

---

## Fas 5: GEDCOM-import
**Prioritet:** Hög
**Beroenden:** Fas 2 (Relationer)
**Estimerad omfattning:** Stor

### 5.1 GEDCOM-parser
**Vad:** Parsa GEDCOM 5.5-filer.

**Funktioner:**
- Läs GEDCOM-fil
- Parsa INDI (individer)
- Parsa FAM (familjer)
- Hantera olika datumformat (ABT, BEF, AFT, etc.)

**Filer att skapa:**
- `src/gedcom/mod.rs` - Modul för GEDCOM
- `src/gedcom/parser.rs` - Parser
- `src/gedcom/models.rs` - GEDCOM-datastrukturer

**Tekniska alternativ:**
- Använda `gedcom` crate (om lämplig)
- Implementera egen parser (mer kontroll)

### 5.2 Import-logik
**Vad:** Importera parsad data till databasen.

**Funktioner:**
- Skapa Person-poster från INDI
- Generera unika katalognamn
- Skapa relationer från FAM
- Hantera duplicering/konflikter
- Visa importstatistik

**Filer att skapa:**
- `src/gedcom/importer.rs` - Importlogik
- `src/services/gedcom_service.rs` - Tjänst för import

### 5.3 Import-UI
**Vad:** Användargränssnitt för GEDCOM-import.

**Funktioner:**
- Välj GEDCOM-fil
- Förhandsgranskning (antal personer, familjer)
- Progressindikator
- Resultatrapport

**Filer att skapa:**
- `src/ui/modals/gedcom_import.rs` - Import-modal

---

## Fas 6: Backup & Restore
**Prioritet:** Medel
**Beroenden:** Inga
**Estimerad omfattning:** Medel

### 6.1 Skapa backup
**Vad:** Skapa ZIP-backup av databas och filer.

**Funktioner:**
- Skapa ZIP med databas + media-katalog
- Tidsstämplat filnamn
- Spara till backup-katalog
- Visa progress

**Filer att skapa:**
- `src/services/backup.rs` - Backup-logik (använd zip crate)

### 6.2 Återställ backup
**Vad:** Återställ från backup-fil.

**Funktioner:**
- Välj backup-fil
- Validera innehåll
- Extrahera och ersätt
- Starta om databaskoppling

**Filer att skapa:**
- `src/services/restore.rs` - Restore-logik

### 6.3 Backup-hantering UI
**Vad:** Lista och hantera backuper.

**Funktioner:**
- Lista befintliga backuper
- Skapa ny backup
- Ta bort gammal backup
- Återställ vald backup

**Filer att skapa:**
- `src/ui/views/backup_view.rs` - Backup-vy

---

## Fas 7: Familjeträd
**Prioritet:** Medel
**Beroenden:** Fas 2 (Relationer)
**Estimerad omfattning:** Stor

### 7.1 Träddatastruktur
**Vad:** Bygg trädstruktur från relationer.

**Funktioner:**
- Hitta föräldrar rekursivt (anor)
- Hitta barn rekursivt (ättlingar)
- Hitta syskon
- Hitta make/maka

**Filer att skapa:**
- `src/services/family_tree.rs` - Trädbyggande

### 7.2 Trädvisualisering
**Vad:** Rita interaktivt familjeträd.

**Funktioner:**
- Visa person i centrum
- Visa föräldrar ovanför
- Visa barn nedanför
- Visa syskon bredvid
- Klickbara noder för navigation
- Pan och zoom

**Filer att skapa:**
- `src/ui/views/family_tree.rs` - Trädvy
- `src/ui/widgets/tree_node.rs` - Nod-widget

**Tekniska utmaningar:**
- egui har inget inbyggt trädlayout
- Alternativ: Custom painting med egui::Painter
- Alternativ: Extern layout-algoritm

---

## Fas 8: Setup Wizard
**Prioritet:** Låg (kan köras manuellt)
**Beroenden:** Fas 5, 6 (GEDCOM, Backup)
**Estimerad omfattning:** Liten

### 8.1 Första start
**Vad:** Guide för ny installation.

**Funktioner:**
- Välj media-katalog
- Välj backup-katalog
- Valfri GEDCOM-import
- Valfri restore från backup
- Skapa standarddata

**Filer att skapa:**
- `src/ui/views/setup_wizard.rs` - Wizard-vy

---

## Fas 9: Rapporter och Export
**Prioritet:** Låg
**Beroenden:** Fas 1, 3 (Dokument, Checklistor)
**Estimerad omfattning:** Medel

### 9.1 Kronologisk rapport
**Vad:** Visa alla källor för en person sorterat på år.

**Filer att skapa:**
- `src/ui/views/chronological_report.rs`

### 9.2 Checklistrapport
**Vad:** Visa checklistframsteg för alla personer.

**Filer att skapa:**
- `src/ui/views/checklist_report.rs`

### 9.3 Personexport
**Vad:** Exportera persondata till JSON/CSV.

**Funktioner:**
- Välj format (JSON/CSV)
- Välj vad som ska inkluderas
- Spara fil

**Filer att skapa:**
- `src/services/export.rs` - Export-logik
- `src/ui/modals/export_dialog.rs` - Export-dialog

---

## Fas 10: Filhanteringsintegration
**Prioritet:** Låg
**Beroenden:** Fas 1
**Estimerad omfattning:** Liten

### 10.1 Öppna i filhanterare
**Vad:** Öppna personens katalog i systemets filhanterare.

**Funktioner:**
- Detektera OS
- Öppna med xdg-open (Linux), open (macOS), explorer (Windows)

**Filer att skapa:**
- `src/utils/os_integration.rs`

---

## Sammanfattning av faser

| Fas | Namn | Prioritet | Beroenden | Omfattning |
|-----|------|-----------|-----------|------------|
| 1 | Dokumenthantering | Hög | - | Medel |
| 2 | Relationshantering | Hög | - | Liten |
| 3 | Checklistor | Medel | - | Medel |
| 4 | Bildhantering | Medel | Fas 1 | Medel |
| 5 | GEDCOM-import | Hög | Fas 2 | Stor |
| 6 | Backup & Restore | Medel | - | Medel |
| 7 | Familjeträd | Medel | Fas 2 | Stor |
| 8 | Setup Wizard | Låg | Fas 5, 6 | Liten |
| 9 | Rapporter & Export | Låg | Fas 1, 3 | Medel |
| 10 | Filhanteringsintegration | Låg | Fas 1 | Liten |

---

## Rekommenderad ordning

**Sprint 1:** Fas 1 (Dokumenthantering) + Fas 2 (Relationer)
- Grundläggande funktionalitet som krävs för daglig användning

**Sprint 2:** Fas 5 (GEDCOM) + Fas 6 (Backup)
- Importera befintlig data, säkerställ att data inte förloras

**Sprint 3:** Fas 3 (Checklistor) + Fas 4 (Bilder)
- Utökad funktionalitet

**Sprint 4:** Fas 7 (Familjeträd)
- Visuell representation av släktdata

**Sprint 5:** Fas 8-10 (Övrigt)
- Nice-to-have funktioner

---

## Tekniska anteckningar

### Crates att lägga till
```toml
# För GEDCOM (alternativ)
# gedcom = "1.0"  # Eller implementera egen parser

# För ZIP (redan tillagd)
zip = "2.2"

# För EXIF (redan tillagd)
kamadak-exif = "0.5"

# För fildialoger (redan tillagd)
rfd = "0.14"

# För filsystemoperationer (redan tillagd)
walkdir = "2.5"
```

### Trådsäkerhet
- Långa operationer (backup, import) bör köras i bakgrunden
- Använd `std::thread::spawn` eller `tokio` för async
- Kommunicera progress via channels

### Filsystemoperationer
- Använd alltid `std::fs` för filoperationer
- Hantera fel gracefully (visa användarvänliga meddelanden)
- Validera sökvägar innan operationer

---

## Anteckningar för varje session

### Session 2026-01-29 (Initial)
- Skapade migreringsplan
- Identifierade 114 funktioner att migrera
- Prioriterade faser baserat på användarbehov

### Session 2026-01-29 (Fas 1 - Progress)
**Status:** Pågående

**Skapade filer:**
- `src/utils/file_ops.rs` - Filoperationer (kopiera, flytta, ta bort, skanna katalog)
- `src/ui/modals/document_upload.rs` - Modal för dokumentuppladdning
- `src/ui/views/document_viewer.rs` - Vy för att visa/redigera dokument
- `src/services/mod.rs` - Services-modul
- `src/services/document_sync.rs` - Dokumentsynkronisering från filsystem

**Uppdaterade filer:**
- `src/utils/mod.rs` - Lade till file_ops
- `src/ui/modals/mod.rs` - Lade till DocumentUploadModal
- `src/ui/views/mod.rs` - Lade till DocumentViewerView
- `src/ui/state.rs` - Lade till View::DocumentViewer, document states
- `src/ui/views/person_detail.rs` - Integrerade dokument-uppladdning och synk
- `src/app.rs` - Integrerade nya vyer och modaler
- `src/main.rs` - Lade till services-modul
- `src/lib.rs` - Lade till services-modul

**Implementerade funktioner:**
- [x] 1.1 Dokumentuppladdning (modal med filväljare, dokumenttyp, kopiera till rätt katalog)
- [x] 1.2 Dokumentvisning (text, bilder, PDF-placeholder)
- [x] 1.3 Dokumentredigering (textfiler inline, metadata)
- [x] 1.4 Dokumentsynkronisering (skanna filsystem, lägg till/ta bort/uppdatera)

**Kvarstående för Fas 1:**
- [ ] Testa uppladdning i praktiken
- [ ] Testa synkronisering i praktiken
- [ ] Eventuella buggfixar

**Kompileringsstatus:** ✅ Kompilerar med endast varningar (oanvända funktioner)

**Teststatus:** ✅ 26 av 26 tester passerar

**Buggfix:**
- Fixade relationslogiken i `find_by_person_with_names()` - returnerade fel perspektiv

### Session 2026-01-30 (Fas 2 - Klar)
**Status:** ✅ Komplett

**Ändringar:**
- Integrerade `RelationshipFormModal` i `app.rs`
- Lade till delete-knappar för relationer i `person_detail.rs`
- Uppdaterade `app.rs` för att refresha person_detail efter relation-ändringar

**Uppdaterade filer:**
- `src/app.rs` - Importerade och instansierade `RelationshipFormModal`, lade till modal-visning
- `src/ui/views/person_detail.rs` - Lade till delete-knappar för varje relation

**Implementerade funktioner:**
- [x] 2.1 Skapa relation (modal med personval och relationstyp)
- [x] 2.2 Ta bort relation (delete-knapp med bekräftelsedialog)

**Kompileringsstatus:** ✅ Kompilerar med endast varningar (oanvända funktioner)

**Teststatus:** ✅ 26 av 26 tester passerar

### Session 2026-01-30 (Fas 5 + Fas 6 - Sprint 2 Klar)
**Status:** ✅ Komplett

**Fas 5: GEDCOM-import**

Skapade filer:
- `src/gedcom/mod.rs` - Modulstruktur
- `src/gedcom/models.rs` - GedcomIndividual, GedcomFamily, GedcomDate, GedcomData
- `src/gedcom/parser.rs` - GedcomParser med stöd för GEDCOM 5.5
- `src/gedcom/importer.rs` - GedcomImporter med preview och import-logik
- `src/ui/modals/gedcom_import.rs` - Import-modal med steg (välj fil → förhandsgranska → importera → klar)

Uppdaterade filer:
- `src/lib.rs` - Lade till gedcom-modul
- `src/main.rs` - Lade till gedcom-modul
- `src/db/person_repo.rs` - Lade till `find_by_directory()` metod
- `src/ui/modals/mod.rs` - Lade till GedcomImportModal
- `src/ui/state.rs` - Lade till `show_gedcom_import` flagga
- `src/ui/views/dashboard.rs` - Lade till "Importera GEDCOM" knapp
- `src/app.rs` - Integrerade GedcomImportModal

Implementerade funktioner:
- [x] 5.1 GEDCOM-parser (INDI, FAM, datumformat)
- [x] 5.2 Import-logik (personer, relationer, unika katalognamn)
- [x] 5.3 Import-UI (filval, förhandsgranskning, progress, resultat)

**Fas 6: Backup & Restore**

Skapade filer:
- `src/services/backup.rs` - BackupService med skapa/lista/radera backuper
- `src/services/restore.rs` - RestoreService med preview och restore
- `src/ui/views/backup_view.rs` - BackupView med lista, skapa, återställ

Uppdaterade filer:
- `src/services/mod.rs` - Lade till backup och restore
- `src/ui/views/mod.rs` - Lade till BackupView
- `src/ui/state.rs` - Lade till View::Backup
- `src/ui/views/dashboard.rs` - Lade till "Backup" knapp
- `src/app.rs` - Integrerade BackupView

Implementerade funktioner:
- [x] 6.1 Skapa backup (ZIP med databas + media)
- [x] 6.2 Återställ backup (validering, förhandsgranskning, alternativ)
- [x] 6.3 Backup-hantering UI (lista, skapa, radera, återställ)

**Kompileringsstatus:** ✅ Kompilerar med endast varningar

**Teststatus:** ✅ 33 av 33 tester passerar (7 nya tester)

### Session 2026-01-30 (Fas 3 + Fas 4 - Sprint 3 Pågående)
**Status:** ✅ Grundläggande funktionalitet klar

**Fas 3: Checklistor**

Skapade filer:
- `src/db/checklist_repo.rs` - ChecklistRepository med CRUD, progress, toggle
- `src/ui/widgets/checklist_panel.rs` - ChecklistPanel widget för persondetaljvy

Uppdaterade filer:
- `src/db/mod.rs` - Lade till checklist_repo modul och checklists() metod
- `src/models/checklist.rs` - Lade till Hash, PartialOrd, Ord traits för ChecklistCategory/Priority
- `src/ui/widgets/mod.rs` - Lade till checklist_panel modul
- `src/ui/views/person_detail.rs` - Integrerade ChecklistPanel i persondetaljvy

Implementerade funktioner:
- [x] 3.1 Visa checklistor (grupperade per kategori, progress bar)
- [x] 3.2 Hantera checklistobjekt (skapa, redigera, toggle, radera)
- [ ] 3.3 Checklistmallar (ej implementerat)

**Fas 4: Bildhantering**

Skapade filer:
- `src/ui/widgets/image_gallery.rs` - ImageGallery widget med thumbnail-grid och lightbox

Uppdaterade filer:
- `src/ui/widgets/mod.rs` - Lade till image_gallery modul
- `src/ui/views/person_detail.rs` - Integrerade ImageGallery i persondetaljvy

Implementerade funktioner:
- [x] 4.1 Bildgalleri (thumbnail-grid, lightbox, navigation)
- [ ] 4.2 Profilbild (ej implementerat)
- [ ] 4.3 Bilduppladdning multi (ej implementerat)
- [ ] 4.4 EXIF-hantering (ej implementerat)

**PersonDetailView layout:**
- Rad 1: Relationer | Dokument
- Rad 2: Checklista | Bildgalleri

**Tekniska utmaningar lösta:**
- Borrow conflicts i ImageGallery: Löst genom att pre-collecta data och ladda texturer före rendering
- HashMap<ChecklistCategory, ...>: Krävde Hash, PartialOrd, Ord traits

**Kompileringsstatus:** ✅ Kompilerar med endast varningar

**Teststatus:** ✅ 36 av 36 tester passerar (3 nya checklist-tester)

### Session 2026-01-30 (Fas 7 - Familjeträd Klar)
**Status:** ✅ Komplett

**Fas 7: Familjeträd**

Skapade filer:
- `src/services/family_tree.rs` - FamilyTreeService för att bygga trädstruktur
  - FamilyTree, FamilyTreeNode, FamilyTreeLink, LinkType
  - Traverserar relationer och bygger noder per generation
  - Automatisk layout-beräkning
- `src/ui/views/family_tree.rs` - FamilyTreeView med interaktiv canvas
  - egui::Painter för custom rendering
  - Pan (dra för att panorera)
  - Zoom (scrolla eller knappar)
  - Klickbara noder (klicka = fokusera, dubbelklicka = navigera till person)
  - Generationsväljare (1-5 generationer)
  - Länkar mellan föräldrar-barn och partners

Uppdaterade filer:
- `src/services/mod.rs` - Lade till family_tree modul och exports
- `src/ui/views/mod.rs` - Lade till family_tree modul
- `src/app.rs` - Integrerade FamilyTreeView, lade till i navigation

Implementerade funktioner:
- [x] 7.1 Träddatastruktur (bygger från relationer)
- [x] 7.2 Trädvisualisering (interaktiv canvas med pan/zoom)

**Kompileringsstatus:** ✅ Kompilerar med endast varningar

**Teststatus:** ✅ 38 av 38 tester passerar (2 nya family_tree-tester)

<!-- Lägg till nya sessionsanteckningar här -->
### Session 2026-01-30 (Koordinering)
**Status:** ℹ️ Ingen kod ändrad

**Anteckning:**
- Codex läste `COORDINATION.md`, `MIGRATION_PLAN.md`, `CLAUDE.md` för att följa arbetsflöde och konventioner.

### Session 2026-01-30 (Fas 3.3 - Checklistmallar Klar)
**Status:** ✅ Komplett

**Skapade filer:**
- `src/services/checklist_sync.rs` - ChecklistSyncService för att applicera mallar till personer
- `src/ui/views/checklist_templates.rs` - ChecklistTemplatesView (CRUD + applicera mall)

**Uppdaterade filer:**
- `src/db/checklist_repo.rs` - CRUD för checklistmallar och mall-objekt
- `src/services/mod.rs` - Export av checklist_sync
- `src/ui/views/mod.rs` - Export av checklist_templates
- `src/ui/views/settings.rs` - Länk till checklistmallar
- `src/ui/state.rs` - Ny View::ChecklistTemplates
- `src/app.rs` - Integrerad vy och navigation

**Implementerade funktioner:**
- [x] 3.3 Checklistmallar (CRUD för mallar och mall-objekt)
- [x] Applicera mall till alla personer eller vald person (skapar PersonChecklistItem)

### Session 2026-01-30 (Fas 8 - Setup Wizard Klar)
**Status:** ✅ Komplett

**Skapade filer:**
- `src/ui/views/setup_wizard.rs` - SetupWizardView med steg och valfria åtgärder

**Uppdaterade filer:**
- `src/app.rs` - Visar wizard om setup ej klar, och integrerar vy
- `src/ui/views/mod.rs` - Export av setup_wizard

**Implementerade funktioner:**
- [x] 8.1 Första start (steg: media-katalog, backup-katalog, klar)
- [x] Valfria åtgärder: importera GEDCOM, återställ backup

### Session 2026-01-30 (Städa varningar)
**Status:** ✅ Komplett

**Ändringar:**
- Kör `cargo fix` för lib och bin
- Tog bort oanvänd assignment i `src/ui/views/backup_view.rs`
- Tog bort oanvända fält i `src/ui/views/family_tree.rs`
- La till `#![allow(dead_code)]` i `src/lib.rs` och `src/main.rs` för att tysta kvarstående dead_code-varningar
- Uppgraderade `rfd` till 0.17.2 för att undvika future-incompat från `ashpd` 0.8.1
