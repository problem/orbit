use bevy::prelude::*;
use nalgebra::Vector3;
use serde::{Serialize, Deserialize};
use crate::primitives::vertex::Vertex;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Component)]
pub struct Face {
    /// Indices of vertices that make up this face
    pub vertex_indices: Vec<usize>,
}

impl Face {
    pub fn new(vertex_indices: Vec<usize>) -> Self {
        Self { vertex_indices }
    }

    pub fn triangle(v1: usize, v2: usize, v3: usize) -> Self {
        Self::new(vec![v1, v2, v3])
    }

    pub fn quad(v1: usize, v2: usize, v3: usize, v4: usize) -> Self {
        Self::new(vec![v1, v2, v3, v4])
    }

    pub fn normal(&self, vertices: &[Vertex]) -> Option<Vector3<f32>> {
        if self.vertex_indices.len() < 3 {
            return None;
        }

        let v0 = vertices.get(self.vertex_indices[0])?;
        let v1 = vertices.get(self.vertex_indices[1])?;
        let v2 = vertices.get(self.vertex_indices[2])?;

        let edge1 = v1.position - v0.position;
        let edge2 = v2.position - v0.position;

        let normal = edge1.cross(&edge2);
        let normalized = if normal.norm() != 0.0 {
            normal.normalize()
        } else {
            normal
        };

        Some(normalized)
    }

    pub fn area(&self, vertices: &[Vertex]) -> Option<f32> {
        if self.vertex_indices.len() < 3 {
            return None;
        }

        // For triangulated face, we can compute the area directly
        if self.vertex_indices.len() == 3 {
            let v0 = vertices.get(self.vertex_indices[0])?;
            let v1 = vertices.get(self.vertex_indices[1])?;
            let v2 = vertices.get(self.vertex_indices[2])?;

            let edge1 = v1.position - v0.position;
            let edge2 = v2.position - v0.position;

            let cross = edge1.cross(&edge2);
            return Some(cross.norm() * 0.5);
        }

        // For more complex polygons, triangulate and sum areas
        // Simplified triangulation assuming convex polygon
        let mut total_area = 0.0;
        let v0 = vertices.get(self.vertex_indices[0])?;

        for i in 1..(self.vertex_indices.len() - 1) {
            let v1 = vertices.get(self.vertex_indices[i])?;
            let v2 = vertices.get(self.vertex_indices[i + 1])?;

            let edge1 = v1.position - v0.position;
            let edge2 = v2.position - v0.position;

            let cross = edge1.cross(&edge2);
            total_area += cross.norm() * 0.5;
        }

        Some(total_area)
    }
}

/// Bevy-specific component to store face rendering data
#[derive(Component)]
pub struct FaceMarker;
