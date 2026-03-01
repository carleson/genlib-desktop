use anyhow::{anyhow, Result};
use rusqlite::{params, Connection, Row};
use std::sync::{Arc, Mutex};

use crate::models::{Resource, ResourceAddress, ResourceDocument, ResourceType};

pub struct ResourceRepository {
    conn: Arc<Mutex<Connection>>,
}

impl ResourceRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    // ── Resurstyper ──────────────────────────────────────────────────────────

    pub fn get_all_types(&self) -> Result<Vec<ResourceType>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, directory_name, created_at, updated_at
             FROM resource_types ORDER BY name",
        )?;
        let types = stmt
            .query_map([], Self::row_to_type)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(types)
    }

    pub fn get_type_by_id(&self, id: i64) -> Result<Option<ResourceType>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, directory_name, created_at, updated_at
             FROM resource_types WHERE id = ?",
        )?;
        let result = stmt
            .query_map([id], Self::row_to_type)?
            .filter_map(|r| r.ok())
            .next();
        Ok(result)
    }

    pub fn create_type(&self, t: &ResourceType) -> Result<ResourceType> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO resource_types (name, directory_name) VALUES (?, ?)",
            params![t.name, t.directory_name],
        )?;
        let id = conn.last_insert_rowid();
        drop(conn);
        self.get_type_by_id(id)?.ok_or_else(|| anyhow!("Kunde inte hämta skapad resurstyp"))
    }

    pub fn update_type(&self, t: &ResourceType) -> Result<()> {
        let id = t.id.ok_or_else(|| anyhow!("ResourceType saknar id"))?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE resource_types SET name = ?, directory_name = ?, updated_at = datetime('now') WHERE id = ?",
            params![t.name, t.directory_name, id],
        )?;
        Ok(())
    }

    pub fn delete_type(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM resource_types WHERE id = ?", [id])?;
        Ok(())
    }

    pub fn type_has_resources(&self, type_id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM resources WHERE resource_type_id = ?",
            [type_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    // ── Resurser ─────────────────────────────────────────────────────────────

    pub fn find_all(&self) -> Result<Vec<(Resource, ResourceType)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT r.id, r.resource_type_id, r.name, r.directory_name,
                    r.information, r.comment, r.lat, r.lon, r.profile_image_path,
                    r.created_at, r.updated_at,
                    rt.id, rt.name, rt.directory_name, rt.created_at, rt.updated_at
             FROM resources r
             JOIN resource_types rt ON r.resource_type_id = rt.id
             ORDER BY rt.name, r.name",
        )?;
        let results = stmt
            .query_map([], Self::row_to_resource_with_type)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(results)
    }

    pub fn find_by_id(&self, id: i64) -> Result<Option<Resource>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, resource_type_id, name, directory_name,
                    information, comment, lat, lon, profile_image_path,
                    created_at, updated_at
             FROM resources WHERE id = ?",
        )?;
        let result = stmt
            .query_map([id], Self::row_to_resource)?
            .filter_map(|r| r.ok())
            .next();
        Ok(result)
    }

    pub fn find_with_type(&self, id: i64) -> Result<Option<(Resource, ResourceType)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT r.id, r.resource_type_id, r.name, r.directory_name,
                    r.information, r.comment, r.lat, r.lon, r.profile_image_path,
                    r.created_at, r.updated_at,
                    rt.id, rt.name, rt.directory_name, rt.created_at, rt.updated_at
             FROM resources r
             JOIN resource_types rt ON r.resource_type_id = rt.id
             WHERE r.id = ?",
        )?;
        let result = stmt
            .query_map([id], Self::row_to_resource_with_type)?
            .filter_map(|r| r.ok())
            .next();
        Ok(result)
    }

    pub fn search(&self, query: &str, type_filter: Option<i64>) -> Result<Vec<(Resource, ResourceType)>> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("%{}%", query.to_lowercase());

        let sql = if type_filter.is_some() {
            "SELECT r.id, r.resource_type_id, r.name, r.directory_name,
                    r.information, r.comment, r.lat, r.lon, r.profile_image_path,
                    r.created_at, r.updated_at,
                    rt.id, rt.name, rt.directory_name, rt.created_at, rt.updated_at
             FROM resources r
             JOIN resource_types rt ON r.resource_type_id = rt.id
             WHERE (LOWER(r.name) LIKE ?1 OR LOWER(r.information) LIKE ?1)
               AND r.resource_type_id = ?2
             ORDER BY rt.name, r.name"
        } else {
            "SELECT r.id, r.resource_type_id, r.name, r.directory_name,
                    r.information, r.comment, r.lat, r.lon, r.profile_image_path,
                    r.created_at, r.updated_at,
                    rt.id, rt.name, rt.directory_name, rt.created_at, rt.updated_at
             FROM resources r
             JOIN resource_types rt ON r.resource_type_id = rt.id
             WHERE LOWER(r.name) LIKE ?1 OR LOWER(r.information) LIKE ?1
             ORDER BY rt.name, r.name"
        };

        let mut stmt = conn.prepare(sql)?;
        let results = if let Some(type_id) = type_filter {
            stmt.query_map(params![pattern, type_id], Self::row_to_resource_with_type)?
                .filter_map(|r| r.ok())
                .collect()
        } else {
            stmt.query_map(params![pattern], Self::row_to_resource_with_type)?
                .filter_map(|r| r.ok())
                .collect()
        };
        Ok(results)
    }

    pub fn create(&self, r: &Resource) -> Result<Resource> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO resources (resource_type_id, name, directory_name, information, comment, lat, lon, profile_image_path)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                r.resource_type_id,
                r.name,
                r.directory_name,
                r.information,
                r.comment,
                r.lat,
                r.lon,
                r.profile_image_path,
            ],
        )?;
        let id = conn.last_insert_rowid();
        drop(conn);
        self.find_by_id(id)?.ok_or_else(|| anyhow!("Kunde inte hämta skapad resurs"))
    }

    pub fn update(&self, r: &Resource) -> Result<()> {
        let id = r.id.ok_or_else(|| anyhow!("Resource saknar id"))?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE resources SET resource_type_id = ?, name = ?, directory_name = ?,
             information = ?, comment = ?, lat = ?, lon = ?, profile_image_path = ?,
             updated_at = datetime('now')
             WHERE id = ?",
            params![
                r.resource_type_id,
                r.name,
                r.directory_name,
                r.information,
                r.comment,
                r.lat,
                r.lon,
                r.profile_image_path,
                id,
            ],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM resources WHERE id = ?", [id])?;
        Ok(())
    }

    // ── Adresser ─────────────────────────────────────────────────────────────

    pub fn get_addresses(&self, resource_id: i64) -> Result<Vec<ResourceAddress>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, resource_id, street, postal_code, city, country, created_at
             FROM resource_addresses WHERE resource_id = ? ORDER BY id",
        )?;
        let results = stmt
            .query_map([resource_id], Self::row_to_address)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(results)
    }

    pub fn create_address(&self, a: &ResourceAddress) -> Result<ResourceAddress> {
        let id = {
            let conn = self.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO resource_addresses (resource_id, street, postal_code, city, country)
                 VALUES (?, ?, ?, ?, ?)",
                params![a.resource_id, a.street, a.postal_code, a.city, a.country],
            )?;
            conn.last_insert_rowid()
        };
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, resource_id, street, postal_code, city, country, created_at
             FROM resource_addresses WHERE id = ?",
        )?;
        let result: Option<ResourceAddress> = stmt
            .query_map([id], Self::row_to_address)?
            .filter_map(|r| r.ok())
            .next();
        result.ok_or_else(|| anyhow!("Kunde inte hämta skapad adress"))
    }

    pub fn delete_address(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM resource_addresses WHERE id = ?", [id])?;
        Ok(())
    }

    // ── Dokument ─────────────────────────────────────────────────────────────

    pub fn get_documents(&self, resource_id: i64) -> Result<Vec<ResourceDocument>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, resource_id, document_type_id, filename, relative_path,
                    file_size, file_type, file_modified_at, created_at, updated_at
             FROM resource_documents WHERE resource_id = ? ORDER BY created_at DESC",
        )?;
        let results = stmt
            .query_map([resource_id], Self::row_to_document)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(results)
    }

    pub fn create_document(&self, d: &ResourceDocument) -> Result<ResourceDocument> {
        let id = {
            let conn = self.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO resource_documents (resource_id, document_type_id, filename, relative_path,
                 file_size, file_type, file_modified_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
                params![
                    d.resource_id,
                    d.document_type_id,
                    d.filename,
                    d.relative_path,
                    d.file_size,
                    d.file_type,
                    d.file_modified_at,
                ],
            )?;
            conn.last_insert_rowid()
        };
        self.find_document_by_id(id)?.ok_or_else(|| anyhow!("Kunde inte hämta skapat dokument"))
    }

    pub fn find_document_by_id(&self, id: i64) -> Result<Option<ResourceDocument>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, resource_id, document_type_id, filename, relative_path,
                    file_size, file_type, file_modified_at, created_at, updated_at
             FROM resource_documents WHERE id = ?",
        )?;
        let result = stmt
            .query_map([id], Self::row_to_document)?
            .filter_map(|r| r.ok())
            .next();
        Ok(result)
    }

    pub fn delete_document(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM resource_documents WHERE id = ?", [id])?;
        Ok(())
    }

    pub fn count(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count = conn.query_row("SELECT COUNT(*) FROM resources", [], |row| row.get(0))?;
        Ok(count)
    }

    // ── Hjälpmetoder ─────────────────────────────────────────────────────────

    fn row_to_type(row: &Row<'_>) -> rusqlite::Result<ResourceType> {
        Ok(ResourceType {
            id: row.get(0)?,
            name: row.get(1)?,
            directory_name: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    }

    fn row_to_resource(row: &Row<'_>) -> rusqlite::Result<Resource> {
        Ok(Resource {
            id: row.get(0)?,
            resource_type_id: row.get(1)?,
            name: row.get(2)?,
            directory_name: row.get(3)?,
            information: row.get(4)?,
            comment: row.get(5)?,
            lat: row.get(6)?,
            lon: row.get(7)?,
            profile_image_path: row.get(8)?,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
        })
    }

    fn row_to_resource_with_type(row: &Row<'_>) -> rusqlite::Result<(Resource, ResourceType)> {
        let resource = Resource {
            id: row.get(0)?,
            resource_type_id: row.get(1)?,
            name: row.get(2)?,
            directory_name: row.get(3)?,
            information: row.get(4)?,
            comment: row.get(5)?,
            lat: row.get(6)?,
            lon: row.get(7)?,
            profile_image_path: row.get(8)?,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
        };
        let resource_type = ResourceType {
            id: row.get(11)?,
            name: row.get(12)?,
            directory_name: row.get(13)?,
            created_at: row.get(14)?,
            updated_at: row.get(15)?,
        };
        Ok((resource, resource_type))
    }

    fn row_to_address(row: &Row<'_>) -> rusqlite::Result<ResourceAddress> {
        Ok(ResourceAddress {
            id: row.get(0)?,
            resource_id: row.get(1)?,
            street: row.get(2)?,
            postal_code: row.get(3)?,
            city: row.get(4)?,
            country: row.get(5)?,
            created_at: row.get(6)?,
        })
    }

    fn row_to_document(row: &Row<'_>) -> rusqlite::Result<ResourceDocument> {
        Ok(ResourceDocument {
            id: row.get(0)?,
            resource_id: row.get(1)?,
            document_type_id: row.get(2)?,
            filename: row.get(3)?,
            relative_path: row.get(4)?,
            file_size: row.get(5)?,
            file_type: row.get(6)?,
            file_modified_at: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::models::resource::sanitize_directory_name;

    fn test_db() -> Database {
        Database::open_in_memory().unwrap()
    }

    #[test]
    fn test_create_and_find_type() {
        let db = test_db();
        let repo = db.resources();
        let types = repo.get_all_types().unwrap();
        // Standardtyper skapas vid migration
        assert!(types.len() >= 3);
        assert!(types.iter().any(|t| t.name == "Fastigheter"));
    }

    #[test]
    fn test_create_resource() {
        let db = test_db();
        let repo = db.resources();
        let types = repo.get_all_types().unwrap();
        let type_id = types[0].id.unwrap();

        let r = Resource::new("Teststuga".to_string(), type_id);
        let created = repo.create(&r).unwrap();
        assert!(created.id.is_some());
        assert_eq!(created.name, "Teststuga");
        assert_eq!(created.directory_name, sanitize_directory_name("Teststuga"));
    }

    #[test]
    fn test_create_address() {
        let db = test_db();
        let repo = db.resources();
        let types = repo.get_all_types().unwrap();
        let type_id = types[0].id.unwrap();

        let r = Resource::new("Teststuga".to_string(), type_id);
        let resource = repo.create(&r).unwrap();

        let mut addr = ResourceAddress::new(resource.id.unwrap());
        addr.street = Some("Storgatan 1".to_string());
        addr.city = Some("Stockholm".to_string());
        let created = repo.create_address(&addr).unwrap();
        assert!(created.id.is_some());
        assert_eq!(created.city, Some("Stockholm".to_string()));
    }

    #[test]
    fn test_search_resources() {
        let db = test_db();
        let repo = db.resources();
        let types = repo.get_all_types().unwrap();
        let type_id = types[0].id.unwrap();

        repo.create(&Resource::new("Gamla Stallet".to_string(), type_id)).unwrap();
        repo.create(&Resource::new("Nya Hotellet".to_string(), type_id)).unwrap();

        let results = repo.search("gamla", None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.name, "Gamla Stallet");

        let all = repo.search("", None).unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_delete_resource() {
        let db = test_db();
        let repo = db.resources();
        let types = repo.get_all_types().unwrap();
        let type_id = types[0].id.unwrap();

        let r = repo.create(&Resource::new("Att radera".to_string(), type_id)).unwrap();
        let id = r.id.unwrap();
        repo.delete(id).unwrap();
        assert!(repo.find_by_id(id).unwrap().is_none());
    }
}
