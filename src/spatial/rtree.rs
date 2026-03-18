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
pub struct SpatialIndex {
    tree: RTree<SpatialEntry>,
}

impl SpatialIndex {
    pub fn new() -> Self {
        Self {
            tree: RTree::new(),
        }
    }

    pub fn insert(&mut self, entry: SpatialEntry) {
        self.tree.insert(entry);
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

    pub fn len(&self) -> usize {
        self.tree.size()
    }

    pub fn is_empty(&self) -> bool {
        self.tree.size() == 0
    }
}

impl Default for SpatialIndex {
    fn default() -> Self {
        Self::new()
    }
}
