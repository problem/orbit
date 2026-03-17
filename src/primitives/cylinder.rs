use nalgebra::Vector3;
use crate::primitives::{vertex::Vertex, edge::Edge, face::Face};
use crate::units::{Measurement, Unit};
use std::f32::consts::PI;

#[derive(Debug, Clone, PartialEq)]
pub enum CylinderError {
    NegativeRadius,
    NegativeHeight,
    TooFewSegments,
}

impl std::fmt::Display for CylinderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CylinderError::NegativeRadius => write!(f, "Cylinder radius must be positive"),
            CylinderError::NegativeHeight => write!(f, "Cylinder height must be positive"),
            CylinderError::TooFewSegments => write!(f, "Cylinder must have at least 3 segments"),
        }
    }
}

impl std::error::Error for CylinderError {}

/// Represents a cylinder primitive
pub struct Cylinder {
    /// Radius of the cylinder
    pub radius: Measurement,
    /// Height of the cylinder
    pub height: Measurement,
    /// Position of the center of the cylinder
    pub position: Vector3<f32>,
    /// Number of segments to use for the circular cross-section
    pub segments: usize,
}

impl Cylinder {
    pub fn new(radius: Measurement, height: Measurement, position: Vector3<f32>, segments: usize) -> Result<Self, CylinderError> {
        if radius.value_in_meters() <= 0.0 {
            return Err(CylinderError::NegativeRadius);
        }
        if height.value_in_meters() <= 0.0 {
            return Err(CylinderError::NegativeHeight);
        }
        if segments < 3 {
            return Err(CylinderError::TooFewSegments);
        }

        Ok(Self {
            radius,
            height,
            position,
            segments,
        })
    }

    /// Create a cylinder with equal radius and height
    pub fn cylinder(size: f32, unit: Unit, position: Vector3<f32>, segments: usize) -> Result<Self, CylinderError> {
        let measurement = Measurement::new(size, unit).map_err(|_| CylinderError::NegativeRadius)?;
        Self::new(measurement, measurement, position, segments)
    }

    /// Generate mesh data for this cylinder
    pub fn generate_mesh_data(&self) -> (Vec<Vertex>, Vec<Edge>, Vec<Face>) {
        let radius_m = self.radius.value_in_meters();
        let height_m = self.height.value_in_meters();
        let half_height = height_m / 2.0;

        // Generate vertices
        let mut vertices = Vec::new();

        // Center points for the top and bottom caps
        vertices.push(Vertex::new(self.position.x, self.position.y - half_height, self.position.z)); // Bottom center (0)
        vertices.push(Vertex::new(self.position.x, self.position.y + half_height, self.position.z)); // Top center (1)

        // Create vertices for the bottom cap
        for i in 0..self.segments {
            let angle = (i as f32 / self.segments as f32) * 2.0 * PI;
            let x = self.position.x + radius_m * angle.cos();
            let z = self.position.z + radius_m * angle.sin();
            vertices.push(Vertex::new(x, self.position.y - half_height, z));
        }

        // Create vertices for the top cap
        for i in 0..self.segments {
            let angle = (i as f32 / self.segments as f32) * 2.0 * PI;
            let x = self.position.x + radius_m * angle.cos();
            let z = self.position.z + radius_m * angle.sin();
            vertices.push(Vertex::new(x, self.position.y + half_height, z));
        }

        // Generate edges
        let mut edges = Vec::new();

        // Bottom cap edges
        for i in 0..self.segments {
            let j = (i + 1) % self.segments;
            edges.push(Edge::new(i + 2, j + 2)); // Outer circle
            edges.push(Edge::new(0, i + 2)); // Spoke to center
        }

        // Top cap edges
        for i in 0..self.segments {
            let j = (i + 1) % self.segments;
            edges.push(Edge::new(i + 2 + self.segments, j + 2 + self.segments)); // Outer circle
            edges.push(Edge::new(1, i + 2 + self.segments)); // Spoke to center
        }

        // Vertical edges connecting top and bottom
        for i in 0..self.segments {
            edges.push(Edge::new(i + 2, i + 2 + self.segments));
        }

        // Generate faces
        let mut faces = Vec::new();

        // Bottom cap triangles
        for i in 0..self.segments {
            let j = (i + 1) % self.segments;
            faces.push(Face::triangle(0, i + 2, j + 2));
        }

        // Top cap triangles
        for i in 0..self.segments {
            let j = (i + 1) % self.segments;
            faces.push(Face::triangle(1, j + 2 + self.segments, i + 2 + self.segments));
        }

        // Side quads
        for i in 0..self.segments {
            let j = (i + 1) % self.segments;
            faces.push(Face::quad(
                i + 2,
                j + 2,
                j + 2 + self.segments,
                i + 2 + self.segments
            ));
        }

        (vertices, edges, faces)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_cylinder_creation() {
        let radius = Measurement::new(1.0, Unit::Meters);
        let height = Measurement::new(2.0, Unit::Meters);
        let position = Vector3::new(0.0, 0.0, 0.0);
        let segments = 8;

        let cylinder = Cylinder::new(radius, height, position, segments).unwrap();

        assert_relative_eq!(cylinder.radius.value_in_meters(), 1.0);
        assert_relative_eq!(cylinder.height.value_in_meters(), 2.0);
        assert_eq!(cylinder.segments, 8);
    }

    #[test]
    fn test_invalid_parameters() {
        let position = Vector3::new(0.0, 0.0, 0.0);

        // Test negative radius
        let result = Cylinder::new(
            Measurement::new(-1.0, Unit::Meters),
            Measurement::new(2.0, Unit::Meters),
            position,
            8,
        );
        assert!(matches!(result, Err(CylinderError::NegativeRadius)));

        // Test negative height
        let result = Cylinder::new(
            Measurement::new(1.0, Unit::Meters),
            Measurement::new(-2.0, Unit::Meters),
            position,
            8,
        );
        assert!(matches!(result, Err(CylinderError::NegativeHeight)));

        // Test too few segments
        let result = Cylinder::new(
            Measurement::new(1.0, Unit::Meters),
            Measurement::new(2.0, Unit::Meters),
            position,
            2,
        );
        assert!(matches!(result, Err(CylinderError::TooFewSegments)));
    }

    #[test]
    fn test_mesh_data_generation() {
        let radius = Measurement::new(1.0, Unit::Meters);
        let height = Measurement::new(2.0, Unit::Meters);
        let position = Vector3::new(0.0, 0.0, 0.0);
        let segments = 4;

        let cylinder = Cylinder::new(radius, height, position, segments).unwrap();
        let (vertices, edges, faces) = cylinder.generate_mesh_data();

        // For a cylinder with 4 segments, we expect:
        // - 2 center vertices (top and bottom)
        // - 4 vertices for bottom circle
        // - 4 vertices for top circle
        assert_eq!(vertices.len(), 10);

        // Edges:
        // - 4 edges for bottom circle
        // - 4 spokes to bottom center
        // - 4 edges for top circle
        // - 4 spokes to top center
        // - 4 vertical edges
        assert_eq!(edges.len(), 20);

        // Faces:
        // - 4 triangles for bottom cap
        // - 4 triangles for top cap
        // - 4 quads for sides
        assert_eq!(faces.len(), 12);

        // Test vertex positions
        let half_height = height.value_in_meters() / 2.0;
        
        // Check center vertices
        assert_relative_eq!(vertices[0].y, -half_height); // Bottom center
        assert_relative_eq!(vertices[1].y, half_height);  // Top center

        // Check radius of a vertex on the bottom circle
        let bottom_vertex = &vertices[2];
        let radius_check = (bottom_vertex.x.powi(2) + bottom_vertex.z.powi(2)).sqrt();
        assert_relative_eq!(radius_check, radius.value_in_meters(), epsilon = 1e-6);
    }

    #[test]
    fn test_different_units() {
        let radius = Measurement::new(100.0, Unit::Centimeters);
        let height = Measurement::new(2.0, Unit::Meters);
        let position = Vector3::new(0.0, 0.0, 0.0);
        let segments = 4;

        let cylinder = Cylinder::new(radius, height, position, segments).unwrap();
        let (vertices, _, _) = cylinder.generate_mesh_data();

        // Check that the radius was correctly converted to meters
        let bottom_vertex = &vertices[2];
        let radius_check = (bottom_vertex.x.powi(2) + bottom_vertex.z.powi(2)).sqrt();
        assert_relative_eq!(radius_check, 1.0, epsilon = 1e-6); // 100cm = 1m
    }

    #[test]
    fn test_error_messages() {
        let position = Vector3::new(0.0, 0.0, 0.0);

        // Test negative radius error message
        let err = Cylinder::new(
            Measurement::new(-1.0, Unit::Meters).unwrap(),
            Measurement::new(2.0, Unit::Meters).unwrap(),
            position,
            8,
        ).unwrap_err();
        assert_eq!(err.to_string(), "Cylinder radius must be positive");

        // Test negative height error message
        let err = Cylinder::new(
            Measurement::new(1.0, Unit::Meters).unwrap(),
            Measurement::new(-2.0, Unit::Meters).unwrap(),
            position,
            8,
        ).unwrap_err();
        assert_eq!(err.to_string(), "Cylinder height must be positive");

        // Test too few segments error message
        let err = Cylinder::new(
            Measurement::new(1.0, Unit::Meters).unwrap(),
            Measurement::new(2.0, Unit::Meters).unwrap(),
            position,
            2,
        ).unwrap_err();
        assert_eq!(err.to_string(), "Cylinder must have at least 3 segments");
    }

    #[test]
    fn test_cylinder_constructor() {
        let position = Vector3::new(0.0, 0.0, 0.0);
        let segments = 8;

        // Test successful creation
        let cylinder = Cylinder::cylinder(2.0, Unit::Meters, position, segments).unwrap();
        assert_relative_eq!(cylinder.radius.value_in_meters(), 2.0);
        assert_relative_eq!(cylinder.height.value_in_meters(), 2.0);
        assert_eq!(cylinder.segments, segments);

        // Test with negative size
        let result = Cylinder::cylinder(-1.0, Unit::Meters, position, segments);
        assert!(matches!(result, Err(CylinderError::NegativeRadius)));

        // Test with too few segments
        let result = Cylinder::cylinder(2.0, Unit::Meters, position, 2);
        assert!(matches!(result, Err(CylinderError::TooFewSegments)));
    }
}
