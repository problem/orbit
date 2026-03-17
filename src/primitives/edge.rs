use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use crate::primitives::vertex::Vertex;
use crate::units::Measurement;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Component)]
pub struct Edge {
    /// Index of start vertex
    pub start_vertex_id: usize,
    /// Index of end vertex
    pub end_vertex_id: usize,
}

impl Edge {
    pub fn new(start_vertex_id: usize, end_vertex_id: usize) -> Self {
        Self {
            start_vertex_id,
            end_vertex_id,
        }
    }

    pub fn length(&self, vertices: &[Vertex]) -> Option<f32> {
        if let (Some(start), Some(end)) = (vertices.get(self.start_vertex_id), vertices.get(self.end_vertex_id)) {
            Some(start.distance_to(end))
        } else {
            None
        }
    }

    pub fn midpoint(&self, vertices: &[Vertex]) -> Option<Vertex> {
        if let (Some(start), Some(end)) = (vertices.get(self.start_vertex_id), vertices.get(self.end_vertex_id)) {
            let midpoint = (start.position + end.position) * 0.5;
            Some(Vertex::new(midpoint.x, midpoint.y, midpoint.z))
        } else {
            None
        }
    }
}

/// Bevy-specific component to store edge rendering data
#[derive(Component)]
pub struct EdgeMarker;
