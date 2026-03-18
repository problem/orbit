use std::path::Path;

use anyhow::Result;
use rusqlite::Connection;

use super::mesh::MeshData;
use super::schema::{self, ORB_FORMAT_VERSION};
use super::types::{Entity, Layer, Material};
use super::uuid::OrbId;

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
