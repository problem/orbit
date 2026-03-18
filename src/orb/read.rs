use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use rusqlite::Connection;

use super::mesh::MeshData;
use super::schema;
use super::transform::Transform;
use super::types::{Entity, EntityType, Layer, Material};
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
        self.query_entities("SELECT id, parent_id, name, entity_type, transform, visible, locked, layer_id, source_unit, created_at, modified_at FROM orb_entities", [])
    }

    pub fn read_entity(&self, id: &OrbId) -> Result<Option<Entity>> {
        let entities = self.query_entities(
            "SELECT id, parent_id, name, entity_type, transform, visible, locked, layer_id, source_unit, created_at, modified_at FROM orb_entities WHERE id = ?1",
            [id as &dyn rusqlite::types::ToSql],
        )?;
        Ok(entities.into_iter().next())
    }

    pub fn read_entities_by_type(&self, entity_type: EntityType) -> Result<Vec<Entity>> {
        self.query_entities(
            "SELECT id, parent_id, name, entity_type, transform, visible, locked, layer_id, source_unit, created_at, modified_at FROM orb_entities WHERE entity_type = ?1",
            [&entity_type.to_string() as &dyn rusqlite::types::ToSql],
        )
    }

    pub fn read_children(&self, parent_id: &OrbId) -> Result<Vec<Entity>> {
        self.query_entities(
            "SELECT id, parent_id, name, entity_type, transform, visible, locked, layer_id, source_unit, created_at, modified_at FROM orb_entities WHERE parent_id = ?1",
            [parent_id as &dyn rusqlite::types::ToSql],
        )
    }

    pub fn read_root_entities(&self) -> Result<Vec<Entity>> {
        self.query_entities(
            "SELECT id, parent_id, name, entity_type, transform, visible, locked, layer_id, source_unit, created_at, modified_at FROM orb_entities WHERE parent_id IS NULL",
            [],
        )
    }

    fn query_entities<P: rusqlite::Params>(&self, sql: &str, params: P) -> Result<Vec<Entity>> {
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(params, |row| {
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

    pub fn read_layers(&self) -> Result<Vec<Layer>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, color, visible, locked, sort_order FROM orb_layers ORDER BY sort_order",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Layer {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                visible: row.get::<_, i32>(3).unwrap_or(1) != 0,
                locked: row.get::<_, i32>(4).unwrap_or(0) != 0,
                sort_order: row.get(5).unwrap_or(0),
            })
        })?;
        let mut layers = Vec::new();
        for row in rows {
            layers.push(row?);
        }
        Ok(layers)
    }

    pub fn entity_count(&self) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM orb_entities", [], |row| row.get(0))?;
        Ok(count as usize)
    }
}
