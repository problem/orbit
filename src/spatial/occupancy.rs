use crate::orb::types::{BuildingSystem, OccupancyType};
use crate::orb::uuid::OrbId;

/// Clearance envelope primitive types per spec §4.13.2.
#[derive(Debug, Clone)]
pub enum ClearanceEnvelope {
    /// Axis-aligned box: min and max corners.
    AaBox {
        min: [f64; 3],
        max: [f64; 3],
    },
    /// Oriented box: center, half-extents, and rotation quaternion.
    OrientedBox {
        center: [f64; 3],
        half_extents: [f64; 3],
        rotation: [f64; 4],
    },
    /// Cylinder: base center, axis direction, radius, height.
    Cylinder {
        base_center: [f64; 3],
        axis: [f64; 3],
        radius: f64,
        height: f64,
    },
    /// Half-cylinder: base center, axis, normal (defines which half), radius, height.
    HalfCylinder {
        base_center: [f64; 3],
        axis: [f64; 3],
        normal: [f64; 3],
        radius: f64,
        height: f64,
    },
}

impl ClearanceEnvelope {
    /// Envelope type ID for BLOB serialization.
    pub fn type_id(&self) -> u8 {
        match self {
            Self::AaBox { .. } => 0x01,
            Self::OrientedBox { .. } => 0x02,
            Self::Cylinder { .. } => 0x03,
            Self::HalfCylinder { .. } => 0x04,
        }
    }
}

/// Spatial occupancy record for an entity.
#[derive(Debug, Clone)]
pub struct OccupancyRecord {
    pub entity_id: OrbId,
    pub occupancy_type: OccupancyType,
    pub clearance_envelopes: Vec<ClearanceEnvelope>,
    pub priority: i32,
    pub system: Option<BuildingSystem>,
}

impl OccupancyRecord {
    pub fn solid(entity_id: OrbId, system: BuildingSystem) -> Self {
        let priority = default_priority(&system);
        Self {
            entity_id,
            occupancy_type: OccupancyType::Solid,
            clearance_envelopes: Vec::new(),
            priority,
            system: Some(system),
        }
    }
}

/// Default priority for a building system per spec §4.13.4.
fn default_priority(system: &BuildingSystem) -> i32 {
    match system {
        BuildingSystem::Structural => 10,
        BuildingSystem::Architectural => 20,
        BuildingSystem::FireProtection => 30,
        BuildingSystem::Plumbing => 40,
        BuildingSystem::Mechanical => 50,
        BuildingSystem::Electrical => 60,
        BuildingSystem::Furniture => 90,
    }
}
