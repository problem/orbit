use nalgebra::Vector3;
use crate::primitives::{vertex::Vertex, edge::Edge, face::Face};
use crate::units::measurement::{Unit, Dimensions3D};

/// Represents a cube/box primitive
pub struct Cube {
    /// Dimensions of the cube
    pub dimensions: Dimensions3D,
    /// Position of the center of the cube
    pub position: Vector3<f32>,
}

impl Cube {
    pub fn new(dimensions: Dimensions3D, position: Vector3<f32>) -> Self {
        Self {
            dimensions,
            position,
        }
    }

    /// Create a cube with equal dimensions in all axes
    pub fn cube(size: f32, unit: Unit, position: Vector3<f32>) -> Result<Self, crate::units::MeasurementError> {
        let dimensions = Dimensions3D::cuboid(size, unit)?;
        Ok(Self::new(dimensions, position))
    }

    /// Generate mesh data for this cube
    pub fn generate_mesh_data(&self) -> (Vec<Vertex>, Vec<Edge>, Vec<Face>) {
        let width_m = self.dimensions.width.value_in_meters();
        let height_m = self.dimensions.height.value_in_meters();
        let depth_m = self.dimensions.depth.value_in_meters();

        let half_width = width_m / 2.0;
        let half_height = height_m / 2.0;
        let half_depth = depth_m / 2.0;

        // Generate 8 vertices for the cube
        let vertices = vec![
            // Bottom face (y = -half_height)
            Vertex::new(self.position.x - half_width, self.position.y - half_height, self.position.z - half_depth), // 0: left, bottom, back
            Vertex::new(self.position.x + half_width, self.position.y - half_height, self.position.z - half_depth), // 1: right, bottom, back
            Vertex::new(self.position.x + half_width, self.position.y - half_height, self.position.z + half_depth), // 2: right, bottom, front
            Vertex::new(self.position.x - half_width, self.position.y - half_height, self.position.z + half_depth), // 3: left, bottom, front

            // Top face (y = +half_height)
            Vertex::new(self.position.x - half_width, self.position.y + half_height, self.position.z - half_depth), // 4: left, top, back
            Vertex::new(self.position.x + half_width, self.position.y + half_height, self.position.z - half_depth), // 5: right, top, back
            Vertex::new(self.position.x + half_width, self.position.y + half_height, self.position.z + half_depth), // 6: right, top, front
            Vertex::new(self.position.x - half_width, self.position.y + half_height, self.position.z + half_depth), // 7: left, top, front
        ];

        // Generate 12 edges for the cube
        let edges = vec![
            // Bottom face
            Edge::new(0, 1), // bottom back
            Edge::new(1, 2), // bottom right
            Edge::new(2, 3), // bottom front
            Edge::new(3, 0), // bottom left

            // Top face
            Edge::new(4, 5), // top back
            Edge::new(5, 6), // top right
            Edge::new(6, 7), // top front
            Edge::new(7, 4), // top left

            // Vertical edges
            Edge::new(0, 4), // back left
            Edge::new(1, 5), // back right
            Edge::new(2, 6), // front right
            Edge::new(3, 7), // front left
        ];

        // Generate 6 faces for the cube with consistent winding order (counter-clockwise when viewed from outside)
        let faces = vec![
            Face::quad(3, 2, 1, 0), // bottom (-Y)
            Face::quad(4, 5, 6, 7), // top (+Y)
            Face::quad(4, 0, 1, 5), // back (-Z)
            Face::quad(7, 6, 2, 3), // front (+Z)
            Face::quad(4, 7, 3, 0), // left (-X)
            Face::quad(5, 1, 2, 6), // right (+X)
        ];

        (vertices, edges, faces)
    }
}
