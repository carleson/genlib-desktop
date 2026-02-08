# Genlib Desktop

Ett dokumenthanteringssystem för släktforskning - native desktop-applikation byggd med Rust och egui.

## Funktioner

- **Personhantering** - Skapa, redigera och organisera personer med födelse/dödsdata
- **Relationer** - Hantera familjerelationer (förälder/barn, make/maka, syskon)
- **Dokumenthantering** - Organisera dokument, bilder och anteckningar per person
- **Checklistor** - Spåra forskningsframsteg med anpassningsbara checklistor
- **GEDCOM-import** - Importera befintlig släktdata (kommande)
- **Familjeträd** - Visualisera släktträd (kommande)
- **Backup/Restore** - Säkerhetskopiera och återställ all data

## Byggkrav

- Rust 1.75+
- Linux: `libgtk-3-dev` `libxdo-dev`
- macOS: Xcode Command Line Tools
- Windows: Visual Studio Build Tools

## Bygga

```bash
# Debug-build
cargo build

# Release-build (optimerad)
cargo build --release
```

## Köra

```bash
# Utvecklingsläge
cargo run

# Release
cargo run --release
```

## Köra tester

```bash
cargo test
```

## Projektstruktur

```
genlib-desktop/
├── src/
│   ├── main.rs           # Entry point
│   ├── app.rs            # Huvudapplikation (eframe::App)
│   ├── lib.rs            # Library exports
│   ├── models/           # Datamodeller
│   ├── db/               # Databas (SQLite)
│   ├── ui/               # egui-vyer och widgets
│   └── utils/            # Hjälpfunktioner
├── resources/            # Ikoner, fonts
└── tests/                # Integrationstester
```

## Data

Applikationen sparar data i:
- **Linux**: `~/.local/share/genlib/`
- **macOS**: `~/Library/Application Support/se.genlib.Genlib/`
- **Windows**: `%APPDATA%\genlib\Genlib\`

## Licens

MIT
