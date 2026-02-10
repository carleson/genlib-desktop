use anyhow::Result;
use rusqlite::Connection;
use tracing::info;

use super::schema::{CREATE_TABLES, DEFAULT_DOCUMENT_TYPES, SCHEMA_VERSION};

/// Kör alla nödvändiga migrationer
pub fn run_migrations(conn: &Connection) -> Result<()> {
    let current_version = get_current_version(conn)?;

    if current_version == 0 {
        // Ny databas - skapa allt
        info!("Skapar ny databas med schema version {}", SCHEMA_VERSION);
        initial_setup(conn)?;
    } else if current_version < SCHEMA_VERSION {
        // Uppdatera befintlig databas
        info!(
            "Migrerar databas från version {} till {}",
            current_version, SCHEMA_VERSION
        );
        migrate_from(conn, current_version)?;
    } else {
        info!("Databas är uppdaterad (version {})", current_version);
    }

    Ok(())
}

fn get_current_version(conn: &Connection) -> Result<i32> {
    // Kontrollera om schema_migrations-tabellen finns
    let table_exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='schema_migrations')",
        [],
        |row| row.get(0),
    )?;

    if !table_exists {
        return Ok(0);
    }

    // Hämta senaste version
    let version: Option<i32> = conn
        .query_row(
            "SELECT MAX(version) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .ok();

    Ok(version.unwrap_or(0))
}

fn initial_setup(conn: &Connection) -> Result<()> {
    // Skapa alla tabeller
    conn.execute_batch(CREATE_TABLES)?;

    // Sätt in standarddokumenttyper
    insert_default_document_types(conn)?;

    // Sätt in standardmallar
    insert_default_templates(conn)?;

    // Markera migration som klar
    conn.execute(
        "INSERT INTO schema_migrations (version) VALUES (?)",
        [SCHEMA_VERSION],
    )?;

    info!("Initial setup klar");
    Ok(())
}

fn insert_default_document_types(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare(
        "INSERT OR IGNORE INTO document_types (name, target_directory, default_filename) VALUES (?, ?, ?)"
    )?;

    for (name, target_dir, filename) in DEFAULT_DOCUMENT_TYPES {
        stmt.execute([*name, *target_dir, *filename])?;
    }

    info!("Lade till {} standarddokumenttyper", DEFAULT_DOCUMENT_TYPES.len());
    Ok(())
}

fn insert_default_templates(conn: &Connection) -> Result<()> {
    use crate::models::Template;

    for template in Template::default_templates() {
        conn.execute(
            "INSERT OR IGNORE INTO templates (name, description, directories) VALUES (?, ?, ?)",
            [&template.name, &template.description.unwrap_or_default(), &template.directories],
        )?;
    }

    info!("Lade till standardmallar för kataloger");
    Ok(())
}

fn migrate_from(conn: &Connection, from_version: i32) -> Result<()> {
    // Kör migrationer stegvis
    for version in (from_version + 1)..=SCHEMA_VERSION {
        match version {
            2 => migrate_v1_to_v2(conn)?,
            3 => migrate_v2_to_v3(conn)?,
            _ => {}
        }

        // Markera version som migrerad
        conn.execute(
            "INSERT INTO schema_migrations (version) VALUES (?)",
            [version],
        )?;

        info!("Migrerade till version {}", version);
    }

    Ok(())
}

/// Migration v1 -> v2: Byt relationship_a_to_b och relationship_b_to_a
///
/// Buggfix: Relationsformuläret skapade relationer med omvänd riktning.
/// "Förälder" tolkades som "current person ÄR förälder" istället för
/// "den andra personen ÄR förälder". Denna migrering byter kolumnerna
/// så att befintliga relationer får rätt riktning.
fn migrate_v1_to_v2(conn: &Connection) -> Result<()> {
    info!("Migration v2: Korrigerar omvända relationsriktningar");

    let affected = conn.execute(
        "UPDATE person_relationships
         SET relationship_a_to_b = relationship_b_to_a,
             relationship_b_to_a = relationship_a_to_b",
        [],
    )?;

    info!("Korrigerade {} relationer", affected);
    Ok(())
}

/// Migration v2 -> v3: Lägg till dir_name_format i system_config
fn migrate_v2_to_v3(conn: &Connection) -> Result<()> {
    info!("Migration v3: Lägger till dir_name_format i system_config");

    conn.execute_batch(
        "ALTER TABLE system_config ADD COLUMN dir_name_format TEXT NOT NULL DEFAULT 'firstname_first';"
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_initial_migration() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // Verifiera att tabeller skapades
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"persons".to_string()));
        assert!(tables.contains(&"documents".to_string()));
        assert!(tables.contains(&"person_relationships".to_string()));
    }

    #[test]
    fn test_idempotent_migration() {
        let conn = Connection::open_in_memory().unwrap();

        // Kör migrationer två gånger
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();

        // Ska inte krascha
        let version = get_current_version(&conn).unwrap();
        assert_eq!(version, SCHEMA_VERSION);
    }
}
