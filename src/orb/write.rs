use std::path::Path;

use anyhow::Result;
use rusqlite::Connection;

use super::mesh::MeshData;
use super::schema::{self, ORB_FORMAT_VERSION};
use super::types::{Entity, Layer, Material};
use super::uuid::OrbId;
use crate::spatial::aabb::Aabb;
use crate::spatial::clash::ClashResult;
use crate::spatial::occupancy::OccupancyRecord;

/// Writer for creating and populating .orb files.
pub struct OrbWriter {
    conn: Connection,
}

impl OrbWriter {
    /// Create a new .orb file with the full schema and required metadata.
    pub fn create(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        schema::init_database(&conn)?;

        let now = chrono::Utc::now().to_rfc3339();
        let writer = Self { conn };
        writer.set_meta("format_version", ORB_FORMAT_VERSION)?;
        writer.set_meta("created_by", &format!("Orbit {}", env!("CARGO_PKG_VERSION")))?;
        writer.set_meta("created_at", &now)?;
        writer.set_meta("modified_at", &now)?;
        writer.set_meta("display_unit", "mm")?;
        writer.set_meta("up_axis", "z")?;
        Ok(writer)
    }

    /// Begin an explicit transaction. All writes until `commit()` are atomic.
    pub fn begin_transaction(&self) -> Result<()> {
        self.conn.execute_batch("BEGIN TRANSACTION")?;
        Ok(())
    }

    /// Commit the current transaction.
    pub fn commit(&self) -> Result<()> {
        self.conn.execute_batch("COMMIT")?;
        Ok(())
    }

    /// Rollback the current transaction.
    pub fn rollback(&self) -> Result<()> {
        self.conn.execute_batch("ROLLBACK")?;
        Ok(())
    }

    pub fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO orb_meta (key, value) VALUES (?1, ?2)",
            rusqlite::params![key, value],
        )?;
        Ok(())
    }

    pub fn insert_entity(&self, entity: &Entity) -> Result<()> {
        self.conn.execute(
            "INSERT INTO orb_entities (id, parent_id, name, entity_type, transform, visible, locked, layer_id, source_unit, created_at, modified_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                entity.id,
                entity.parent_id,
                entity.name,
                entity.entity_type.to_string(),
                entity.transform,
                entity.visible as i32,
                entity.locked as i32,
                entity.layer_id,
                entity.source_unit,
                entity.created_at,
                entity.modified_at,
            ],
        )?;
        Ok(())
    }

    pub fn insert_mesh(&self, entity_id: &OrbId, mesh: &MeshData) -> Result<()> {
        self.conn.execute(
            "INSERT INTO orb_geometry_mesh (entity_id, positions, normals, indices)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                entity_id,
                mesh.positions_to_blob(),
                mesh.normals_to_blob(),
                mesh.indices_to_blob(),
            ],
        )?;
        Ok(())
    }

    pub fn insert_material(&self, mat: &Material) -> Result<()> {
        self.conn.execute(
            "INSERT INTO orb_materials (id, name, base_color, metallic, roughness, opacity, double_sided)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                mat.id,
                mat.name,
                mat.base_color,
                mat.metallic,
                mat.roughness,
                mat.opacity,
                mat.double_sided as i32,
            ],
        )?;
        Ok(())
    }

    pub fn insert_layer(&self, layer: &Layer) -> Result<()> {
        self.conn.execute(
            "INSERT INTO orb_layers (id, name, color, visible, locked, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                layer.id,
                layer.name,
                layer.color,
                layer.visible as i32,
                layer.locked as i32,
                layer.sort_order,
            ],
        )?;
        Ok(())
    }

    // --- Spatial Index ---

    /// Insert or update the spatial index entry for an entity.
    /// Uses the orb_entity_rowids bridge table to map UUIDv7 → integer rowid.
    pub fn upsert_spatial_entry(&self, entity_id: &OrbId, aabb: &Aabb) -> Result<()> {
        // Get or create the rowid mapping
        self.conn.execute(
            "INSERT OR IGNORE INTO orb_entity_rowids (entity_id) VALUES (?1)",
            rusqlite::params![entity_id],
        )?;
        let rowid: i64 = self.conn.query_row(
            "SELECT rowid FROM orb_entity_rowids WHERE entity_id = ?1",
            rusqlite::params![entity_id],
            |row| row.get(0),
        )?;
        // Upsert into the R-tree
        self.conn.execute(
            "INSERT OR REPLACE INTO orb_spatial_index (id, min_x, max_x, min_y, max_y, min_z, max_z)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                rowid,
                aabb.min.x,
                aabb.max.x,
                aabb.min.y,
                aabb.max.y,
                aabb.min.z,
                aabb.max.z,
            ],
        )?;
        Ok(())
    }

    /// Remove the spatial index entry for an entity.
    pub fn delete_spatial_entry(&self, entity_id: &OrbId) -> Result<()> {
        let result: rusqlite::Result<i64> = self.conn.query_row(
            "SELECT rowid FROM orb_entity_rowids WHERE entity_id = ?1",
            rusqlite::params![entity_id],
            |row| row.get(0),
        );
        if let Ok(rowid) = result {
            self.conn.execute(
                "DELETE FROM orb_spatial_index WHERE id = ?1",
                rusqlite::params![rowid],
            )?;
            self.conn.execute(
                "DELETE FROM orb_entity_rowids WHERE entity_id = ?1",
                rusqlite::params![entity_id],
            )?;
        }
        Ok(())
    }

    // --- Occupancy ---

    pub fn insert_occupancy(&self, record: &OccupancyRecord) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO orb_occupancy (entity_id, occupancy_type, clearance_data, priority, system)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                record.entity_id,
                record.occupancy_type.to_string(),
                record.clearance_blob(),
                record.priority,
                record.system.map(|s| s.to_string()),
            ],
        )?;
        Ok(())
    }

    // --- Clash Results ---

    pub fn insert_clash_result(&self, clash: &ClashResult) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO orb_clash_results
             (id, entity_a, entity_b, clash_type, severity, system_a, system_b,
              intersection_point_x, intersection_point_y, intersection_point_z,
              distance, status, resolved_by, detected_at, resolved_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            rusqlite::params![
                clash.id,
                clash.entity_a,
                clash.entity_b,
                clash.clash_type.to_string(),
                clash.severity.to_string(),
                clash.system_a,
                clash.system_b,
                clash.intersection_point.map(|p| p[0]),
                clash.intersection_point.map(|p| p[1]),
                clash.intersection_point.map(|p| p[2]),
                clash.distance,
                clash.status.to_string(),
                clash.resolved_by,
                clash.detected_at,
                clash.resolved_at,
            ],
        )?;
        Ok(())
    }

    pub fn update_clash_status(
        &self,
        clash_id: &OrbId,
        status: &str,
        resolved_by: Option<&str>,
    ) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE orb_clash_results SET status = ?1, resolved_by = ?2, resolved_at = ?3 WHERE id = ?4",
            rusqlite::params![status, resolved_by, now, clash_id],
        )?;
        Ok(())
    }

    /// Finalize for distribution: checkpoint WAL, vacuum, analyze.
    pub fn finalize(self) -> Result<()> {
        self.conn
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        self.conn.execute_batch("VACUUM;")?;
        self.conn.execute_batch("ANALYZE;")?;
        self.conn.execute_batch("PRAGMA journal_mode = DELETE;")?;
        Ok(())
    }
}
