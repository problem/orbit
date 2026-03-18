use rstar::{RTree, RTreeObject, AABB as RstarAABB};

use super::aabb::Aabb;
use crate::orb::uuid::OrbId;

/// Entry in the spatial index: an entity ID with its bounding box.
#[derive(Debug, Clone)]
pub struct SpatialEntry {
    pub entity_id: OrbId,
    pub aabb: Aabb,
}

impl RTreeObject for SpatialEntry {
    type Envelope = RstarAABB<[f64; 3]>;

    fn envelope(&self) -> Self::Envelope {
        RstarAABB::from_corners(
            [self.aabb.min.x, self.aabb.min.y, self.aabb.min.z],
            [self.aabb.max.x, self.aabb.max.y, self.aabb.max.z],
        )
    }
}

impl rstar::PointDistance for SpatialEntry {
    fn distance_2(&self, point: &[f64; 3]) -> f64 {
        self.envelope().distance_2(point)
    }
}

/// In-memory R-tree spatial index for interactive queries.
/// Synchronizes with the SQLite orb_spatial_index on load/save.
pub struct SpatialIndex {
    tree: RTree<SpatialEntry>,
}

impl SpatialIndex {
    pub fn new() -> Self {
        Self {
            tree: RTree::new(),
        }
    }

    /// Bulk-load from a pre-built list of entries.
    pub fn from_entries(entries: Vec<SpatialEntry>) -> Self {
        Self {
            tree: RTree::bulk_load(entries),
        }
    }

    pub fn insert(&mut self, entry: SpatialEntry) {
        self.tree.insert(entry);
    }

    /// Remove an entity from the index. Returns true if found and removed.
    pub fn remove(&mut self, entity_id: &OrbId) -> bool {
        // rstar requires the exact entry for removal, so we need to find it first.
        let entry = self
            .tree
            .iter()
            .find(|e| &e.entity_id == entity_id)
            .cloned();
        if let Some(entry) = entry {
            self.tree.remove(&entry);
            true
        } else {
            false
        }
    }

    /// Update an entity's bounding box. Removes old entry and inserts new one.
    pub fn update(&mut self, entity_id: OrbId, new_aabb: Aabb) {
        self.remove(&entity_id);
        self.insert(SpatialEntry {
            entity_id,
            aabb: new_aabb,
        });
    }

    /// Query all entries whose AABB intersects the given box.
    pub fn query_aabb(&self, aabb: &Aabb) -> Vec<&SpatialEntry> {
        let envelope = RstarAABB::from_corners(
            [aabb.min.x, aabb.min.y, aabb.min.z],
            [aabb.max.x, aabb.max.y, aabb.max.z],
        );
        self.tree
            .locate_in_envelope_intersecting(&envelope)
            .collect()
    }

    /// Query the nearest entry to a point.
    pub fn query_nearest(&self, point: [f64; 3]) -> Option<&SpatialEntry> {
        self.tree.nearest_neighbor(&point)
    }

    /// Get all entries as a slice (for saving to SQLite).
    pub fn entries(&self) -> Vec<&SpatialEntry> {
        self.tree.iter().collect()
    }

    pub fn len(&self) -> usize {
        self.tree.size()
    }

    pub fn is_empty(&self) -> bool {
        self.tree.size() == 0
    }

    pub fn clear(&mut self) {
        self.tree = RTree::new();
    }

    /// Load the spatial index from an OrbReader (SQLite → in-memory R-tree).
    pub fn load_from_db(reader: &crate::orb::read::OrbReader) -> anyhow::Result<Self> {
        let entities = reader.read_entities()?;
        let mut entries = Vec::new();
        for entity in &entities {
            if let Some(aabb) = reader.read_entity_aabb(&entity.id)? {
                entries.push(SpatialEntry {
                    entity_id: entity.id,
                    aabb,
                });
            }
        }
        Ok(Self::from_entries(entries))
    }

    /// Save the spatial index to an OrbWriter (in-memory R-tree → SQLite).
    pub fn save_to_db(&self, writer: &crate::orb::write::OrbWriter) -> anyhow::Result<()> {
        for entry in self.tree.iter() {
            writer.upsert_spatial_entry(&entry.entity_id, &entry.aabb)?;
        }
        Ok(())
    }
}

impl Default for SpatialIndex {
    fn default() -> Self {
        Self::new()
    }
}

// rstar::Remove requires PartialEq on the entry type.
impl PartialEq for SpatialEntry {
    fn eq(&self, other: &Self) -> bool {
        self.entity_id == other.entity_id
    }
}
