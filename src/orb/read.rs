use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use rusqlite::Connection;

use super::mesh::MeshData;
use super::schema;
use super::transform::Transform;
use super::types::{Entity, EntityType, Material};
use super::uuid::OrbId;

/// Reader for .orb files.
pub struct OrbReader {
    conn: Connection,
}

impl OrbReader {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        schema::verify_database(&conn)?;
        Ok(Self { conn })
    }

    pub fn read_meta(&self) -> Result<HashMap<String, String>> {
        let mut stmt = self.conn.prepare("SELECT key, value FROM orb_meta")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut map = HashMap::new();
        for row in rows {
            let (k, v) = row?;
            map.insert(k, v);
        }
        Ok(map)
    }

    pub fn read_entities(&self) -> Result<Vec<Entity>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, parent_id, name, entity_type, transform, visible, locked, layer_id, source_unit, created_at, modified_at
             FROM orb_entities"
        )?;
        let rows = stmt.query_map([], |row| {
            let entity_type_str: String = row.get(3)?;
            Ok(Entity {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                name: row.get(2)?,
                entity_type: entity_type_str
                    .parse::<EntityType>()
                    .unwrap_or(EntityType::Body),
                transform: row.get(4).unwrap_or_else(|_| Transform::identity()),
                visible: row.get::<_, i32>(5).unwrap_or(1) != 0,
                locked: row.get::<_, i32>(6).unwrap_or(0) != 0,
                layer_id: row.get(7)?,
                source_unit: row.get(8)?,
                created_at: row.get(9)?,
                modified_at: row.get(10)?,
            })
        })?;
        let mut entities = Vec::new();
        for row in rows {
            entities.push(row?);
        }
        Ok(entities)
    }

    pub fn read_mesh(&self, entity_id: &OrbId) -> Result<Option<MeshData>> {
        let mut stmt = self.conn.prepare(
            "SELECT positions, normals, indices, edges FROM orb_geometry_mesh WHERE entity_id = ?1",
        )?;
        let result = stmt.query_row(rusqlite::params![entity_id], |row| {
            let pos_blob: Vec<u8> = row.get(0)?;
            let norm_blob: Vec<u8> = row.get(1)?;
            let idx_blob: Vec<u8> = row.get(2)?;
            let edge_blob: Option<Vec<u8>> = row.get(3)?;
            Ok((pos_blob, norm_blob, idx_blob, edge_blob))
        });

        match result {
            Ok((pos_blob, norm_blob, idx_blob, _edge_blob)) => {
                let positions = MeshData::positions_from_blob(&pos_blob)?;
                let normals = MeshData::normals_from_blob(&norm_blob)?;
                let indices = MeshData::indices_from_blob(&idx_blob)?;
                Ok(Some(MeshData {
                    positions,
                    normals,
                    indices,
                    edges: None,
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn read_materials(&self) -> Result<Vec<Material>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, base_color, metallic, roughness, opacity, double_sided FROM orb_materials",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Material {
                id: row.get(0)?,
                name: row.get(1)?,
                base_color: row.get(2)?,
                metallic: row.get(3)?,
                roughness: row.get(4)?,
                opacity: row.get(5)?,
                double_sided: row.get::<_, i32>(6)? != 0,
            })
        })?;
        let mut materials = Vec::new();
        for row in rows {
            materials.push(row?);
        }
        Ok(materials)
    }
}
