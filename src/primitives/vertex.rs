use bevy::prelude::*;
use nalgebra::Vector3;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct Vertex {
    /// Position in 3D space (in meters)
    pub position: Vector3<f32>,
}

// Custom serialization for Vector3<f32>
impl Serialize for Vertex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Vertex", 1)?;
        let pos = [self.position.x, self.position.y, self.position.z];
        state.serialize_field("position", &pos)?;
        state.end()
    }
}

// Custom deserialization for Vector3<f32>
impl<'de> Deserialize<'de> for Vertex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            position: [f32; 3],
        }

        let helper = Helper::deserialize(deserializer)?;
        Ok(Vertex {
            position: Vector3::new(helper.position[0], helper.position[1], helper.position[2]),
        })
    }
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: Vector3::new(x, y, z),
        }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn distance_to(&self, other: &Vertex) -> f32 {
        (self.position - other.position).norm()
    }
}

/// Bevy-specific component to store vertex rendering data
#[derive(Component)]
pub struct VertexMarker;
