use crate::orb::types::{ClashSeverity, ClashStatus, ClashType};
use crate::orb::uuid::OrbId;

/// A detected spatial conflict between two entities.
#[derive(Debug, Clone)]
pub struct ClashResult {
    pub id: OrbId,
    pub entity_a: OrbId,
    pub entity_b: OrbId,
    pub clash_type: ClashType,
    pub severity: ClashSeverity,
    pub system_a: Option<String>,
    pub system_b: Option<String>,
    pub intersection_point: Option<[f64; 3]>,
    /// Penetration depth (hard clash) or clearance shortfall (clearance violation), in mm.
    pub distance: Option<f64>,
    pub status: ClashStatus,
    pub resolved_by: Option<String>,
    pub detected_at: String,
    pub resolved_at: Option<String>,
}

impl ClashResult {
    pub fn new_hard(entity_a: OrbId, entity_b: OrbId) -> Self {
        Self {
            id: OrbId::new(),
            entity_a,
            entity_b,
            clash_type: ClashType::Hard,
            severity: ClashSeverity::Error,
            system_a: None,
            system_b: None,
            intersection_point: None,
            distance: None,
            status: ClashStatus::Active,
            resolved_by: None,
            detected_at: chrono::Utc::now().to_rfc3339(),
            resolved_at: None,
        }
    }

    pub fn new_clearance(entity_a: OrbId, entity_b: OrbId, shortfall_mm: f64) -> Self {
        Self {
            id: OrbId::new(),
            entity_a,
            entity_b,
            clash_type: ClashType::Clearance,
            severity: ClashSeverity::Warning,
            system_a: None,
            system_b: None,
            intersection_point: None,
            distance: Some(shortfall_mm),
            status: ClashStatus::Active,
            resolved_by: None,
            detected_at: chrono::Utc::now().to_rfc3339(),
            resolved_at: None,
        }
    }
}
