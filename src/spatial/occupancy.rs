use anyhow::{bail, Result};

use crate::orb::types::{BuildingSystem, OccupancyType};
use crate::orb::uuid::OrbId;

/// Clearance envelope primitive types per spec §4.13.2.
#[derive(Debug, Clone, PartialEq)]
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

    /// Byte size of this envelope's parameters (excluding the type byte).
    fn param_size(&self) -> usize {
        match self {
            Self::AaBox { .. } => 6 * 8,           // 6 f64
            Self::OrientedBox { .. } => 10 * 8,     // 3+3+4 f64
            Self::Cylinder { .. } => 8 * 8,          // 3+3+1+1 f64
            Self::HalfCylinder { .. } => 11 * 8,     // 3+3+3+1+1 f64
        }
    }
}

// --- BLOB Codec per spec §4.13.2 ---
// Format: [envelope_count: u16] [envelope_0: type(u8) + params] [envelope_1: ...] ...
// All f64 values are little-endian.

fn write_f64(buf: &mut Vec<u8>, v: f64) {
    buf.extend_from_slice(&v.to_le_bytes());
}

fn write_f64_3(buf: &mut Vec<u8>, v: &[f64; 3]) {
    for &val in v {
        write_f64(buf, val);
    }
}

fn read_f64(data: &[u8], offset: &mut usize) -> Result<f64> {
    if *offset + 8 > data.len() {
        bail!("clearance BLOB truncated at offset {}", *offset);
    }
    let bytes: [u8; 8] = data[*offset..*offset + 8].try_into().unwrap();
    *offset += 8;
    Ok(f64::from_le_bytes(bytes))
}

fn read_f64_3(data: &[u8], offset: &mut usize) -> Result<[f64; 3]> {
    Ok([
        read_f64(data, offset)?,
        read_f64(data, offset)?,
        read_f64(data, offset)?,
    ])
}

fn read_f64_4(data: &[u8], offset: &mut usize) -> Result<[f64; 4]> {
    Ok([
        read_f64(data, offset)?,
        read_f64(data, offset)?,
        read_f64(data, offset)?,
        read_f64(data, offset)?,
    ])
}

/// Serialize clearance envelopes to the packed binary format from spec §4.13.2.
pub fn clearance_to_blob(envelopes: &[ClearanceEnvelope]) -> Vec<u8> {
    let total_size: usize = 2 + envelopes.iter().map(|e| 1 + e.param_size()).sum::<usize>();
    let mut buf = Vec::with_capacity(total_size);

    buf.extend_from_slice(&(envelopes.len() as u16).to_le_bytes());

    for env in envelopes {
        buf.push(env.type_id());
        match env {
            ClearanceEnvelope::AaBox { min, max } => {
                write_f64_3(&mut buf, min);
                write_f64_3(&mut buf, max);
            }
            ClearanceEnvelope::OrientedBox {
                center,
                half_extents,
                rotation,
            } => {
                write_f64_3(&mut buf, center);
                write_f64_3(&mut buf, half_extents);
                for &v in rotation {
                    write_f64(&mut buf, v);
                }
            }
            ClearanceEnvelope::Cylinder {
                base_center,
                axis,
                radius,
                height,
            } => {
                write_f64_3(&mut buf, base_center);
                write_f64_3(&mut buf, axis);
                write_f64(&mut buf, *radius);
                write_f64(&mut buf, *height);
            }
            ClearanceEnvelope::HalfCylinder {
                base_center,
                axis,
                normal,
                radius,
                height,
            } => {
                write_f64_3(&mut buf, base_center);
                write_f64_3(&mut buf, axis);
                write_f64_3(&mut buf, normal);
                write_f64(&mut buf, *radius);
                write_f64(&mut buf, *height);
            }
        }
    }

    buf
}

/// Deserialize clearance envelopes from the packed binary format.
pub fn clearance_from_blob(data: &[u8]) -> Result<Vec<ClearanceEnvelope>> {
    if data.len() < 2 {
        bail!("clearance BLOB too short: {} bytes", data.len());
    }

    let count = u16::from_le_bytes([data[0], data[1]]) as usize;
    let mut offset = 2;
    let mut envelopes = Vec::with_capacity(count);

    for i in 0..count {
        if offset >= data.len() {
            bail!("clearance BLOB truncated at envelope {i}");
        }
        let type_id = data[offset];
        offset += 1;

        let env = match type_id {
            0x01 => {
                let min = read_f64_3(data, &mut offset)?;
                let max = read_f64_3(data, &mut offset)?;
                ClearanceEnvelope::AaBox { min, max }
            }
            0x02 => {
                let center = read_f64_3(data, &mut offset)?;
                let half_extents = read_f64_3(data, &mut offset)?;
                let rotation = read_f64_4(data, &mut offset)?;
                ClearanceEnvelope::OrientedBox {
                    center,
                    half_extents,
                    rotation,
                }
            }
            0x03 => {
                let base_center = read_f64_3(data, &mut offset)?;
                let axis = read_f64_3(data, &mut offset)?;
                let radius = read_f64(data, &mut offset)?;
                let height = read_f64(data, &mut offset)?;
                ClearanceEnvelope::Cylinder {
                    base_center,
                    axis,
                    radius,
                    height,
                }
            }
            0x04 => {
                let base_center = read_f64_3(data, &mut offset)?;
                let axis = read_f64_3(data, &mut offset)?;
                let normal = read_f64_3(data, &mut offset)?;
                let radius = read_f64(data, &mut offset)?;
                let height = read_f64(data, &mut offset)?;
                ClearanceEnvelope::HalfCylinder {
                    base_center,
                    axis,
                    normal,
                    radius,
                    height,
                }
            }
            _ => bail!("unknown clearance envelope type: 0x{:02X}", type_id),
        };
        envelopes.push(env);
    }

    Ok(envelopes)
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

    pub fn penetrable(entity_id: OrbId, system: BuildingSystem) -> Self {
        let priority = default_priority(&system);
        Self {
            entity_id,
            occupancy_type: OccupancyType::Penetrable,
            clearance_envelopes: Vec::new(),
            priority,
            system: Some(system),
        }
    }

    pub fn reservation(entity_id: OrbId, system: BuildingSystem) -> Self {
        let priority = default_priority(&system);
        Self {
            entity_id,
            occupancy_type: OccupancyType::Reservation,
            clearance_envelopes: Vec::new(),
            priority,
            system: Some(system),
        }
    }

    pub fn with_clearance(mut self, envelope: ClearanceEnvelope) -> Self {
        self.clearance_envelopes.push(envelope);
        self
    }

    /// Serialize clearance envelopes to BLOB, or None if empty.
    pub fn clearance_blob(&self) -> Option<Vec<u8>> {
        if self.clearance_envelopes.is_empty() {
            None
        } else {
            Some(clearance_to_blob(&self.clearance_envelopes))
        }
    }
}

/// Default priority for a building system per spec §4.13.4.
pub fn default_priority(system: &BuildingSystem) -> i32 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clearance_blob_roundtrip_empty() {
        let blob = clearance_to_blob(&[]);
        let result = clearance_from_blob(&blob).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_clearance_blob_roundtrip_aa_box() {
        let envelopes = vec![ClearanceEnvelope::AaBox {
            min: [-500.0, -500.0, 0.0],
            max: [500.0, 500.0, 2400.0],
        }];
        let blob = clearance_to_blob(&envelopes);
        let result = clearance_from_blob(&blob).unwrap();
        assert_eq!(result, envelopes);
    }

    #[test]
    fn test_clearance_blob_roundtrip_all_types() {
        let envelopes = vec![
            ClearanceEnvelope::AaBox {
                min: [0.0, 0.0, 0.0],
                max: [1.0, 1.0, 1.0],
            },
            ClearanceEnvelope::OrientedBox {
                center: [5.0, 5.0, 5.0],
                half_extents: [1.0, 2.0, 3.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
            },
            ClearanceEnvelope::Cylinder {
                base_center: [0.0, 0.0, 0.0],
                axis: [0.0, 0.0, 1.0],
                radius: 900.0,
                height: 2100.0,
            },
            ClearanceEnvelope::HalfCylinder {
                base_center: [0.0, 0.0, 0.0],
                axis: [0.0, 0.0, 1.0],
                normal: [1.0, 0.0, 0.0],
                radius: 900.0,
                height: 2100.0,
            },
        ];
        let blob = clearance_to_blob(&envelopes);
        let result = clearance_from_blob(&blob).unwrap();
        assert_eq!(result, envelopes);
    }
}
