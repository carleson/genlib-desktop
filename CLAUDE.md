# CLAUDE.md - Genlib Desktop

> **Multi-agent samarbete:** Se **[COORDINATION.md](COORDINATION.md)** för uppgiftsfördelning mellan Claude och Codex. Läs den filen först vid varje session!

## Projektöversikt

**Genlib Desktop** är en native desktop-applikation för släktforskning, skriven i Rust med egui som GUI-ramverk. Projektet är en konvertering av Django-webapplikationen [genlib](../genlib) till en standalone desktop-app.

## Tekniska Val och Resonemang

### GUI: egui + eframe

**Valt ramverk:** egui v0.29 med eframe

**Fördelar:**
- Pure Rust - inga externa beroenden på webview eller GTK
- Immediate mode GUI - enkel mental modell
- Cross-platform (Linux, macOS, Windows)
- Snabb iteration under utveckling
- Bra dokumentation och aktiv community

**Nackdelar:**
- Immediate mode kan vara ineffektivt för komplexa UI:er
- Inte "native" utseende (men acceptabelt för detta projekt)
- Begränsade widgets jämfört med Qt eller GTK

**Alternativ som övervägdes:**
- Tauri - kräver webkunskaper, mer komplext setup
- iced - Elm-arkitektur, steep learning curve
- Slint - licensfrågor för kommersiell användning

### Databas: SQLite + rusqlite

**Valt:** rusqlite v0.32 med bundled SQLite

**Resonemang:**
- Samma databasmotor som Django-versionen
- Enkel distribution (ingen extern databas krävs)
- `bundled` feature - SQLite kompileras in, inga systemkrav
- `chrono` feature för datumhantering

**Schema-kompatibilitet:**
- Schemat är designat för att vara kompatibelt med Django-versionen
- Migreringssystem för framtida schemaändringar

### Projektstruktur

```
genlib-desktop/
├── src/
│   ├── main.rs           # Entry point, fönsterinställningar
│   ├── app.rs            # GenlibApp - huvudapplikation (eframe::App)
│   ├── lib.rs            # Library exports
│   ├── models/           # Datamodeller
│   │   ├── person.rs     # Person med validering
│   │   ├── document.rs   # Document och DocumentType
│   │   ├── relationship.rs # PersonRelationship med kanonisk ordning
│   │   ├── checklist.rs  # Checklist-modeller
│   │   └── config.rs     # SystemConfig, Template, AppSettings
│   ├── db/               # Databaslager
│   │   ├── mod.rs        # Database wrapper med Arc<Mutex<Connection>>
│   │   ├── schema.rs     # SQL CREATE-statements
│   │   ├── migrations.rs # Migreringssystem
│   │   ├── person_repo.rs
│   │   ├── document_repo.rs
│   │   ├── relationship_repo.rs
│   │   └── config_repo.rs
│   ├── ui/               # Användargränssnitt
│   │   ├── mod.rs
│   │   ├── state.rs      # AppState, View enum, FormData
│   │   ├── theme.rs      # Färger, ikoner, stil
│   │   ├── views/        # Huvudvyer
│   │   │   ├── dashboard.rs
│   │   │   ├── person_list.rs
│   │   │   ├── person_detail.rs
│   │   │   └── settings.rs
│   │   ├── modals/       # Popup-dialoger
│   │   │   ├── person_form.rs
│   │   │   └── confirm_dialog.rs
│   │   └── widgets/      # Återanvändbara widgets
│   └── utils/            # Hjälpfunktioner
│       ├── path.rs       # Sökvägshantering
│       ├── date.rs       # Datumformatering
│       └── error.rs      # AppError typ
├── resources/            # Ikoner, fonts (framtida)
├── tests/                # Integrationstester
├── Cargo.toml
└── README.md
```

### Arkitekturmönster

**Repository Pattern:**
- `Database` är en wrapper med `Arc<Mutex<Connection>>`
- Varje entitet har ett repository (PersonRepository, DocumentRepository, etc.)
- Thread-safe genom Mutex

**State Management:**
- `AppState` håller UI-tillstånd (current_view, selected_person_id, etc.)
- Vyer cachar data lokalt (`needs_refresh` pattern)
- Modals hanterar sin egen form data

**View Pattern:**
- Varje vy är en struct med `show(&mut self, ui, state, db)` metod
- Vyer tar `&mut AppState` för navigation och `&Database` för data

## Implementeringsstatus

### Klart

- [x] **Projektstruktur** - Komplett modulstruktur
- [x] **Databas**
  - [x] Schema med alla tabeller
  - [x] Migreringssystem
  - [x] PersonRepository (CRUD, sök, bokmärken)
  - [x] DocumentRepository (CRUD, gruppering per typ)
  - [x] RelationshipRepository (med gruppering)
  - [x] ConfigRepository
- [x] **Modeller**
  - [x] Person med validering och åldersberäkning
  - [x] Document med filtypsdetektering
  - [x] PersonRelationship med kanonisk ordning
  - [x] Checklist-modeller
  - [x] SystemConfig och Template
- [x] **UI - Vyer**
  - [x] Dashboard med statistik
  - [x] PersonList med sökning, sortering, filter
  - [x] PersonDetail med info, relationer, dokument
  - [x] Settings med konfiguration
- [x] **UI - Modals**
  - [x] PersonFormModal (skapa/redigera)
  - [x] ConfirmDialog
- [x] **Tema**
  - [x] Dark/light mode
  - [x] Färgpalett
  - [x] Unicode-ikoner

### Klart (Faser 1-9)

- [x] **Dokumenthantering** (Fas 1)
  - [x] Ladda upp dokument
  - [x] Visa dokument (bilder, PDF, text)
  - [x] Synkronisera från filsystem
- [x] **Relationshantering** (Fas 2)
  - [x] Skapa relationer (modal)
  - [x] Ta bort relationer
- [x] **Checklistor** (Fas 3)
  - [x] Visa checklistor
  - [x] Bocka av items
  - [x] Checklistmallar
- [x] **Bildhantering** (Fas 4)
  - [x] Bildgalleri med lightbox
  - [x] Profilbild-funktion
  - [x] Thumbnails i personlistan
- [x] **GEDCOM** (Fas 5)
  - [x] Import
- [x] **Backup/Restore** (Fas 6)
  - [x] Skapa ZIP-backup
  - [x] Återställ från backup
- [x] **Familjeträd** (Fas 7)
  - [x] Visualisering av släktträd
  - [x] Pan/zoom
  - [x] Klickbara noder
- [x] **Setup Wizard** (Fas 8)
  - [x] Första-start guide
- [x] **Rapporter** (Fas 9)
  - [x] Export till JSON/CSV
  - [x] Statistiksammanfattning

### Pågående / TODO

- [ ] **GEDCOM Export** - Exportera data till GEDCOM-format

### Nyligen tillagt

- [x] **Avancerad sökning** - Filterkombinatorer (datumintervall, relationer, dokument, profilbild, bokmärken)
- [x] **Multi-upload bilder** - Ladda upp flera bilder/dokument samtidigt
- [x] **EXIF-hantering** - Visa EXIF-metadata i bildgalleriets lightbox
- [x] **PDF-generering** - Exportera rapporter till PDF-format

## Kommandon

```bash
# Utveckling
cargo build          # Debug build
cargo run            # Kör applikationen
cargo check          # Snabb kompileringskontroll

# Test
cargo test           # Kör alla tester

# Release
cargo build --release   # Optimerad build
```

## Databassökvägar

Applikationen använder platform-specifika sökvägar via `directories` crate:

| Platform | Databas | Config |
|----------|---------|--------|
| Linux | `~/.local/share/genlib/genlib.db` | `~/.config/genlib/` |
| macOS | `~/Library/Application Support/se.genlib.Genlib/` | samma |
| Windows | `%APPDATA%\genlib\Genlib\` | samma |

## Databasschema

Se `src/db/schema.rs` för fullständigt schema. Huvudtabeller:

- `persons` - Personer med namn, datum, katalognamn
- `person_relationships` - Relationer med kanonisk ordning (person_a_id < person_b_id)
- `documents` - Dokument med metadata
- `document_types` - Dokumenttyper
- `person_checklist_items` - Checklistobjekt per person
- `system_config` - Systemkonfiguration (singleton)
- `templates` - Katalogmallar

## Viktiga Koncept

### Kanonisk Relationsordning
Relationer lagras alltid med `person_a_id < person_b_id` för att undvika duplicerade relationer. `PersonRelationship::new()` hanterar detta automatiskt.

### Dynamisk Media Root
`SystemConfig` lagrar media-katalog som kan konfigureras av användaren. Använd `config.persons_directory()` för att få sökvägen till persons-katalogen.

### Needs Refresh Pattern
Vyer cachar data lokalt och använder `needs_refresh: bool` för att veta när de ska hämta ny data. Anropa `mark_needs_refresh()` efter ändringar.

## Nästa Steg

Se **[MIGRATION_PLAN.md](MIGRATION_PLAN.md)** för detaljerad migreringsplan med alla faser.

### Kortfattat:
1. **Fas 1: Dokumenthantering** - Ladda upp, visa och synkronisera dokument
2. **Fas 2: Relationshantering** - Skapa och ta bort relationer
3. **Fas 5: GEDCOM-import** - Importera befintlig släktdata
4. **Fas 6: Backup/Restore** - Säkerhetskopiera data

## Anteckningar

- Projektet kompilerar med endast varningar (inga fel)
- Vissa utility-funktioner i `error.rs` är oanvända men behålls för framtida användning
- Unicode-emojis används som ikoner (fungerar på alla plattformar)
