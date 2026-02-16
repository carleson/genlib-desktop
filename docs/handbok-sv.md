# Genlib Desktop – Fullständig handbok

En komplett manual för släktforskare som vill få maximal nytta av Genlib Desktop.

![PLACEHOLDER: Översikt över applikationen](images/placeholder-oversikt.png)

## 1. Översikt
Genlib Desktop är en lokal desktop‑applikation för släktforskning. Fokus ligger på struktur, kvalitet och långsiktig hållbarhet i din forskning.

### Nytta och värde
- **Samlat arkiv:** Personer, relationer, dokument och bilder på ett ställe
- **Spårbarhet:** Dokument kopplas direkt till personer och händelser
- **Överblick:** Familjeträd och statistik ger helhetsbild
- **Trygghet:** Enkel backup och återställning
- **Kontroll:** All data ligger lokalt – du äger allt
 - **Öppen struktur:** Filerna ligger lättåtkomliga i filsystemet och är enkla att arkivera och dela

## 1.1 Skillnaden mot traditionella släktforskningsprogram
Många traditionella program låser in data i egna format eller databaser som kräver samma program för att kunna öppnas fullt ut. Genlib Desktop fungerar istället som ett **skal ovanpå dina egna filer** – du får en enhetlig lagringsstrategi där bilder, PDF:er och anteckningar ligger i en tydlig katalogstruktur i filsystemet.

Det innebär att du kan:
- arkivera och distribuera material till personer utan släktforskningsprogram
- dela forskningsunderlag som vanliga filer, utan export‑krångel
- byta verktyg i framtiden utan att tappa tillgången till källmaterialet

## 2. Installation och första start

### 2.1 Om du har en installerare
Följ installationsstegen som vanligt. Starta Genlib Desktop när installationen är klar.

### 2.2 Om du bygger från källkod (avancerat)
Kräver Rust 1.75+.

```bash
cargo build --release
cargo run --release
```

### 2.3 Förstagångsguiden (Setup Wizard)
Guiden visas automatiskt vid första start.

![PLACEHOLDER: Setup wizard – helvy](images/placeholder-setup-helvy.png)

Steg i guiden:
1. **Media‑katalog** – där dokument och bilder sparas
2. **Backup‑katalog** – var säkerhetskopior hamnar
3. **(Valfritt) Import** – GEDCOM eller återställning från backup

## 3. Grundläggande begrepp

### Person
En personpost innehåller namn, datum, platser och egna anteckningar.

### Relation
Relationer beskriver familjeband (förälder/barn, make/maka, syskon).

### Dokument
Alla typer av källor: bilder, PDF, textanteckningar och övriga filer.

### Checklistor
Forskningssteg att bocka av – hjälper dig se vad som saknas.

### Rapporter
Exporterar sammanställningar till JSON, CSV eller PDF.

## 4. Navigering

![PLACEHOLDER: Navigering och huvudmeny](images/placeholder-nav.png)

- **Dashboard** – snabb överblick
- **Personer** – lista och sök
- **Familjeträd** – visualisering
- **Inställningar** – konfiguration, backup och rapporter

## 5. Personer

### 5.1 Skapa och redigera
1. Klicka **Ny person**
2. Fyll i fält
3. Spara

![PLACEHOLDER: Personformulär](images/placeholder-person-form.png)

### 5.2 Avancerad sökning
Filter för datumintervall, relationer, dokument, profilbild och bokmärken.

![PLACEHOLDER: Avancerade filter](images/placeholder-avancerad-sokning.png)

### 5.3 Profilbild
Sätt en profilbild via bildgalleriet för snabb igenkänning.

![PLACEHOLDER: Profilbild i personlistan](images/placeholder-profilbild.png)

## 6. Relationer
1. Öppna en person
2. Klicka **Lägg till relation**
3. Välj typ och annan person

![PLACEHOLDER: Relationer i personvyn](images/placeholder-relationer.png)

## 7. Dokument och bilder

### 7.1 Lägg till dokument
- **Ladda upp fil** eller **skapa textdokument**
- Dokument kan grupperas per typ

![PLACEHOLDER: Dokumentmodal](images/placeholder-dokument-modal.png)

### 7.2 Flera filer samtidigt
Välj flera bilder eller PDF:er i ett steg.

![PLACEHOLDER: Multi‑upload](images/placeholder-multiupload.png)

### 7.3 Bildgalleri och EXIF
Se metadata (kamera, datum, GPS) direkt i bildvyn.

![PLACEHOLDER: Bildgalleri med EXIF](images/placeholder-exif.png)

## 8. Checklistor
Skapa egna checklistor och använd mallar för återkommande uppgifter.

![PLACEHOLDER: Checklistor](images/placeholder-checklistor.png)

## 9. GEDCOM‑import
Importera data från andra släktprogram via GEDCOM‑fil.

![PLACEHOLDER: GEDCOM‑import](images/placeholder-gedcom-import.png)

**Obs:** Export till GEDCOM är planerat men inte tillgängligt än.

## 10. Familjeträd
Visualisera relationer och navigera mellan generationer.

![PLACEHOLDER: Familjeträd](images/placeholder-familjetrad.png)

## 11. Rapporter och export
Skapa rapporter och exportera till JSON, CSV eller PDF.

![PLACEHOLDER: Rapportvy](images/placeholder-rapporter.png)

## 12. Backup och återställning
Skapa ZIP‑backup och återställ med ett klick.

![PLACEHOLDER: Backup/Restore](images/placeholder-backup.png)

## 13. Datalagring
Standardplatser för databasen:
- **Linux:** `~/.local/share/genlib/genlib.db`
- **macOS:** `~/Library/Application Support/se.genlib.Genlib/`
- **Windows:** `%APPDATA%\genlib\Genlib\`

## 14. Felsökning och FAQ

### Programmet startar inte
- Kontrollera att du har skrivbehörighet i dina valda mappar
- Kör om installationen eller bygg om projektet

### Jag hittar inte mina filer
- Kontrollera vilken media‑katalog du valt i Inställningar

### Backup saknas
- Kontrollera backup‑katalog och skapa en ny backup manuellt

## 15. Tips för effektiv släktforskning
- Namnge dokument konsekvent (år‑typ‑plats)
- Lägg in källhänvisningar tidigt
- Använd checklistor för att driva forskningen framåt

## 16. Vidare steg
- Skapa regelbundna rutiner för backup
- Exportera rapporter för delning med andra släktforskare
- Håll dig uppdaterad om nya versioner
