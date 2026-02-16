# Genlib Desktop – Handbuch

Ein vollständiges Handbuch für Genealoginnen und Genealogen, die den maximalen Nutzen aus Genlib Desktop ziehen möchten.

![PLACEHOLDER: Anwendungsübersicht](images/placeholder-oversikt.png)

## 1. Überblick
Genlib Desktop ist eine lokale Desktop‑Anwendung für Genealogie. Der Fokus liegt auf Struktur, Qualität und langfristiger Nachhaltigkeit Ihrer Forschung.

### Nutzen und Wert
- **Zentrales Archiv:** Personen, Beziehungen, Dokumente und Fotos an einem Ort
- **Nachvollziehbarkeit:** Dokumente sind direkt mit Personen und Ereignissen verknüpft
- **Überblick:** Stammbäume und Statistiken liefern Kontext
- **Sicherheit:** Einfaches Backup und Wiederherstellung
- **Kontrolle:** Alle Daten bleiben lokal – Sie besitzen sie
- **Offene Struktur:** Dateien sind im Dateisystem leicht zugänglich und einfach zu archivieren und zu teilen

## 1.1 Unterschied zu traditionellen Genealogieprogrammen
Viele traditionelle Programme sperren Daten in proprietäre Formate oder Datenbanken, die das gleiche Programm zum vollständigen Öffnen erfordern. Genlib Desktop ist eine **Ergänzung**, die als **Hülle über Ihren eigenen Dateien** fungiert – Sie erhalten eine konsistente Ablagestrategie, bei der Bilder, PDFs und Notizen in einer klaren Ordnerstruktur im Dateisystem liegen.

Das bedeutet, dass Sie:
- Material an Personen ohne Genealogie‑Software archivieren und weitergeben können
- Quellen als normale Dateien teilen können, ohne Export‑Hürden
- später das Werkzeug wechseln können, ohne den Zugriff auf Ihr Quellenmaterial zu verlieren

## 2. Installation und erster Start

### 2.1 Wenn Sie einen Installer haben
Folgen Sie den Installationsschritten wie üblich. Starten Sie Genlib Desktop nach Abschluss der Installation.

### 2.2 Wenn Sie aus dem Quellcode bauen (fortgeschritten)
Erfordert Rust 1.75+.

```bash
cargo build --release
cargo run --release
```

### 2.3 Ersteinrichtung (Setup Wizard)
Der Assistent erscheint automatisch beim ersten Start.

![PLACEHOLDER: Setup‑Wizard – Gesamtansicht](images/placeholder-setup-helvy.png)

Schritte im Assistenten:
1. **Medienordner** – dort werden Dokumente und Fotos gespeichert
2. **Backup‑Ordner** – wo Backups gespeichert werden
3. **(Optional) Import** – GEDCOM oder Wiederherstellung aus Backup

## 3. Grundbegriffe

### Person
Ein Personendatensatz enthält Namen, Daten, Orte und persönliche Notizen.

### Beziehung
Beziehungen beschreiben Familienbande (Elternteil/Kind, Ehepartner, Geschwister).

### Dokument
Alle Quellenarten: Bilder, PDFs, Textnotizen und andere Dateien.

### Checklisten
Forschungsschritte zum Abhaken – hilft, offene Punkte zu sehen.

### Berichte
Exportiert Zusammenfassungen nach JSON, CSV oder PDF.

## 4. Navigation

![PLACEHOLDER: Navigation und Hauptmenü](images/placeholder-nav.png)

- **Dashboard** – schneller Überblick
- **Personen** – Liste und Suche
- **Stammbaum** – Visualisierung
- **Einstellungen** – Konfiguration, Backup und Berichte

## 5. Personen

### 5.1 Erstellen und bearbeiten
1. **Neue Person** anklicken
2. Felder ausfüllen
3. Speichern

![PLACEHOLDER: Personenformular](images/placeholder-person-form.png)

### 5.2 Erweiterte Suche
Filter für Datumsbereiche, Beziehungen, Dokumente, Profilbild und Lesezeichen.

![PLACEHOLDER: Erweiterte Filter](images/placeholder-avancerad-sokning.png)

### 5.3 Profilbild
Profilbild über die Bildergalerie setzen für schnelle Wiedererkennung.

![PLACEHOLDER: Profilbild in der Personenliste](images/placeholder-profilbild.png)

## 6. Beziehungen
1. Person öffnen
2. **Beziehung hinzufügen** anklicken
3. Typ und andere Person wählen

![PLACEHOLDER: Beziehungen in der Personenansicht](images/placeholder-relationer.png)

## 7. Dokumente und Fotos

### 7.1 Dokumente hinzufügen
- **Datei hochladen** oder **Textdokument erstellen**
- Dokumente können nach Typ gruppiert werden

![PLACEHOLDER: Dokument‑Modal](images/placeholder-dokument-modal.png)

### 7.2 Mehrere Dateien gleichzeitig
Mehrere Bilder oder PDFs in einem Schritt auswählen.

![PLACEHOLDER: Multi‑Upload](images/placeholder-multiupload.png)

### 7.3 Bildergalerie und EXIF
Metadaten (Kamera, Datum, GPS) direkt in der Bildansicht sehen.

![PLACEHOLDER: Bildergalerie mit EXIF](images/placeholder-exif.png)

## 8. Checklisten
Eigene Checklisten erstellen und Vorlagen für wiederkehrende Aufgaben nutzen.

![PLACEHOLDER: Checklisten](images/placeholder-checklistor.png)

## 9. GEDCOM‑Import
Daten aus anderen Genealogieprogrammen per GEDCOM‑Datei importieren.

![PLACEHOLDER: GEDCOM‑Import](images/placeholder-gedcom-import.png)

**Hinweis:** GEDCOM‑Export ist geplant, aber noch nicht verfügbar.

## 10. Stammbaum
Beziehungen visualisieren und zwischen Generationen navigieren.

![PLACEHOLDER: Stammbaum](images/placeholder-familjetrad.png)

## 11. Berichte und Export
Berichte erstellen und nach JSON, CSV oder PDF exportieren.

![PLACEHOLDER: Berichtsansicht](images/placeholder-rapporter.png)

## 12. Backup und Wiederherstellung
ZIP‑Backups erstellen und mit einem Klick wiederherstellen.

![PLACEHOLDER: Backup/Restore](images/placeholder-backup.png)

## 13. Datenspeicherung
Standard‑Datenbankorte:
- **Linux:** `~/.local/share/genlib/genlib.db`
- **macOS:** `~/Library/Application Support/se.genlib.Genlib/`
- **Windows:** `%APPDATA%\\genlib\\Genlib\\`

## 14. Fehlerbehebung und FAQ

### Das Programm startet nicht
- Prüfen Sie, ob Sie Schreibrechte für die ausgewählten Ordner haben
- Neu installieren oder das Projekt neu bauen

### Ich finde meine Dateien nicht
- Prüfen Sie, welchen Medienordner Sie in den Einstellungen gewählt haben

### Backups fehlen
- Prüfen Sie den Backup‑Ordner und erstellen Sie manuell ein neues Backup

## 15. Tipps für effiziente Genealogie
- Dokumente konsistent benennen (Jahr‑Typ‑Ort)
- Quellenangaben früh hinzufügen
- Checklisten nutzen, um die Forschung voranzutreiben

## 16. Nächste Schritte
- Regelmäßige Backup‑Routinen anlegen
- Berichte exportieren und mit anderen Genealogen teilen
- Über neue Versionen informiert bleiben
