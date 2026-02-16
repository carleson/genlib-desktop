# Genlib Desktop – Full Handbook

A complete manual for genealogists who want to get the most out of Genlib Desktop.

![PLACEHOLDER: Application overview](images/placeholder-oversikt.png)

## 1. Overview
Genlib Desktop is a local desktop application for genealogy. The focus is on structure, quality, and long-term sustainability of your research.

### Value and benefits
- **Unified archive:** People, relationships, documents, and photos in one place
- **Traceability:** Documents are linked directly to people and events
- **Overview:** Family trees and statistics provide context
- **Safety:** Simple backup and restore
- **Control:** All data stays local—you own it
- **Open structure:** Files are easy to access in the file system and simple to archive and share

## 1.1 Difference from traditional genealogy programs
Many traditional programs lock data into proprietary formats or databases that require the same program to open fully. Genlib Desktop is a **complement** that works as a **shell over your own files**—you get a consistent storage strategy where images, PDFs, and notes live in a clear folder structure in the file system.

This means you can:
- archive and distribute material to people without genealogy software
- share research sources as ordinary files, without export friction
- switch tools in the future without losing access to your source material

## 2. Installation and first start

### 2.1 If you have an installer
Follow the installation steps as usual. Start Genlib Desktop when installation is complete.

### 2.2 If you build from source (advanced)
Requires Rust 1.75+.

```bash
cargo build --release
cargo run --release
```

### 2.3 First‑time setup (Setup Wizard)
The wizard appears automatically on first launch.

![PLACEHOLDER: Setup wizard – full view](images/placeholder-setup-helvy.png)

Steps in the wizard:
1. **Media folder** – where documents and photos are stored
2. **Backup folder** – where backups are saved
3. **(Optional) Import** – GEDCOM or restore from backup

## 3. Core concepts

### Person
A person record contains names, dates, places, and personal notes.

### Relationship
Relationships describe family ties (parent/child, spouse, sibling).

### Document
Any source type: images, PDFs, text notes, and other files.

### Checklists
Research steps to check off—helps you see what is missing.

### Reports
Exports summaries to JSON, CSV, or PDF.

## 4. Navigation

![PLACEHOLDER: Navigation and main menu](images/placeholder-nav.png)

- **Dashboard** – quick overview
- **People** – list and search
- **Family Tree** – visualization
- **Settings** – configuration, backup, and reports

## 5. People

### 5.1 Create and edit
1. Click **New person**
2. Fill in fields
3. Save

![PLACEHOLDER: Person form](images/placeholder-person-form.png)

### 5.2 Advanced search
Filters for date ranges, relationships, documents, profile image, and bookmarks.

![PLACEHOLDER: Advanced filters](images/placeholder-avancerad-sokning.png)

### 5.3 Profile image
Set a profile image via the image gallery for quick recognition.

![PLACEHOLDER: Profile image in the people list](images/placeholder-profilbild.png)

## 6. Relationships
1. Open a person
2. Click **Add relationship**
3. Choose type and the other person

![PLACEHOLDER: Relationships in the person view](images/placeholder-relationer.png)

## 7. Documents and photos

### 7.1 Add documents
- **Upload file** or **create text document**
- Documents can be grouped by type

![PLACEHOLDER: Document modal](images/placeholder-dokument-modal.png)

### 7.2 Multiple files at once
Select multiple images or PDFs in one step.

![PLACEHOLDER: Multi‑upload](images/placeholder-multiupload.png)

### 7.3 Image gallery and EXIF
See metadata (camera, date, GPS) directly in the image view.

![PLACEHOLDER: Image gallery with EXIF](images/placeholder-exif.png)

## 8. Checklists
Create your own checklists and use templates for recurring tasks.

![PLACEHOLDER: Checklists](images/placeholder-checklistor.png)

## 9. GEDCOM import
Import data from other genealogy programs via a GEDCOM file.

![PLACEHOLDER: GEDCOM import](images/placeholder-gedcom-import.png)

**Note:** GEDCOM export is planned but not available yet.

## 10. Family tree
Visualize relationships and navigate between generations.

![PLACEHOLDER: Family tree](images/placeholder-familjetrad.png)

## 11. Reports and export
Create reports and export to JSON, CSV, or PDF.

![PLACEHOLDER: Reports view](images/placeholder-rapporter.png)

## 12. Backup and restore
Create ZIP backups and restore with one click.

![PLACEHOLDER: Backup/Restore](images/placeholder-backup.png)

## 13. Data storage
Default database locations:
- **Linux:** `~/.local/share/genlib/genlib.db`
- **macOS:** `~/Library/Application Support/se.genlib.Genlib/`
- **Windows:** `%APPDATA%\genlib\Genlib\`

## 14. Troubleshooting and FAQ

### The program won’t start
- Check that you have write permissions for your selected folders
- Reinstall or rebuild the project

### I can’t find my files
- Check which media folder you selected in Settings

### Backups are missing
- Check the backup folder and create a new backup manually

## 15. Tips for efficient genealogy
- Name documents consistently (year‑type‑place)
- Add source citations early
- Use checklists to drive research forward

## 16. Next steps
- Create regular backup routines
- Export reports to share with other genealogists
- Stay updated on new releases
