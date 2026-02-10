/// SQL-schema för Genlib Desktop
/// Kompatibelt med Django-export för migration

pub const SCHEMA_VERSION: i32 = 3;

pub const CREATE_TABLES: &str = r#"
-- Systeminställningar (singleton, id=1)
CREATE TABLE IF NOT EXISTS system_config (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    media_directory_path TEXT NOT NULL,
    backup_directory_path TEXT NOT NULL,
    dir_name_format TEXT NOT NULL DEFAULT 'firstname_first',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Personer
CREATE TABLE IF NOT EXISTS persons (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    firstname TEXT,
    surname TEXT,
    birth_date TEXT,
    death_date TEXT,
    age INTEGER,
    notes TEXT,
    directory_name TEXT NOT NULL UNIQUE,
    profile_image_path TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    CHECK (firstname IS NOT NULL OR surname IS NOT NULL)
);

CREATE INDEX IF NOT EXISTS idx_persons_directory ON persons(directory_name);
CREATE INDEX IF NOT EXISTS idx_persons_surname ON persons(surname);
CREATE INDEX IF NOT EXISTS idx_persons_firstname ON persons(firstname);

-- Personrelationer
CREATE TABLE IF NOT EXISTS person_relationships (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    person_a_id INTEGER NOT NULL,
    person_b_id INTEGER NOT NULL,
    relationship_a_to_b INTEGER NOT NULL,
    relationship_b_to_a INTEGER NOT NULL,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (person_a_id) REFERENCES persons(id) ON DELETE CASCADE,
    FOREIGN KEY (person_b_id) REFERENCES persons(id) ON DELETE CASCADE,
    CHECK (person_a_id < person_b_id),
    UNIQUE (person_a_id, person_b_id)
);

CREATE INDEX IF NOT EXISTS idx_relationships_a ON person_relationships(person_a_id);
CREATE INDEX IF NOT EXISTS idx_relationships_b ON person_relationships(person_b_id);

-- Dokumenttyper
CREATE TABLE IF NOT EXISTS document_types (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    target_directory TEXT NOT NULL,
    default_filename TEXT,
    description TEXT
);

-- Dokument
CREATE TABLE IF NOT EXISTS documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    person_id INTEGER NOT NULL,
    document_type_id INTEGER,
    filename TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    file_size INTEGER NOT NULL DEFAULT 0,
    file_type TEXT,
    tags TEXT,
    file_modified_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (person_id) REFERENCES persons(id) ON DELETE CASCADE,
    FOREIGN KEY (document_type_id) REFERENCES document_types(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_documents_person ON documents(person_id);
CREATE INDEX IF NOT EXISTS idx_documents_type ON documents(document_type_id);

-- Checklistmallar
CREATE TABLE IF NOT EXISTS checklist_templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    is_active INTEGER NOT NULL DEFAULT 1
);

-- Checklistmall-objekt
CREATE TABLE IF NOT EXISTS checklist_template_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    template_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    category INTEGER NOT NULL DEFAULT 0,
    priority INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (template_id) REFERENCES checklist_templates(id) ON DELETE CASCADE
);

-- Person-specifika checklistobjekt
CREATE TABLE IF NOT EXISTS person_checklist_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    person_id INTEGER NOT NULL,
    template_item_id INTEGER,
    title TEXT NOT NULL,
    description TEXT,
    category INTEGER NOT NULL DEFAULT 0,
    priority INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_completed INTEGER NOT NULL DEFAULT 0,
    completed_at TEXT,
    notes TEXT,
    FOREIGN KEY (person_id) REFERENCES persons(id) ON DELETE CASCADE,
    FOREIGN KEY (template_item_id) REFERENCES checklist_template_items(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_checklist_person ON person_checklist_items(person_id);

-- Bokmärken
CREATE TABLE IF NOT EXISTS bookmarked_persons (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    person_id INTEGER NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (person_id) REFERENCES persons(id) ON DELETE CASCADE
);

-- Katalogmallar
CREATE TABLE IF NOT EXISTS templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    directories TEXT NOT NULL
);

-- Migrationshistorik
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);
"#;

/// Standard dokumenttyper att skapa vid första start
pub const DEFAULT_DOCUMENT_TYPES: &[(&str, &str, &str)] = &[
    ("Personbevis", "dokument/personbevis", "personbevis.pdf"),
    ("Födelseattest", "dokument/födelseattest", "födelseattest.pdf"),
    ("Dopbevis", "dokument/dopbevis", "dopbevis.pdf"),
    ("Vigselbevis", "dokument/vigselbevis", "vigselbevis.pdf"),
    ("Dödsattest", "dokument/dödsattest", "dödsattest.pdf"),
    ("Folkräkning", "dokument/folkräkning", "folkräkning.pdf"),
    ("Husförhörslängd", "dokument/husförhör", "husförhörslängd.pdf"),
    ("Porträtt", "bilder/porträtt", "porträtt.jpg"),
    ("Dokument-scan", "bilder/dokument", "scan.jpg"),
    ("Anteckningar", "anteckningar", "anteckningar.txt"),
];
