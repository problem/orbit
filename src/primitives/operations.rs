use nalgebra::Vector3;
use crate::primitives::{
    vertex::Vertex,
    edge::Edge,
    face::Face,
};

/// Extrudes a face along its normal by a specified distance
pub fn extrude_face(
    vertices: &mut Vec<Vertex>,
    edges: &mut Vec<Edge>,
    faces: &mut Vec<Face>,
    face_idx: usize,
    distance: f32,
) -> Option<()> {
    let face = faces.get(face_idx)?.clone(); // Clone the face to avoid borrow issues
    let _original_vertex_count = vertices.len();

    // Create new vertices by extruding the face vertices
    let mut new_vertex_indices = Vec::new();
    for &vertex_idx in &face.vertex_indices {
        let vertex = vertices.get(vertex_idx)?;
        let new_vertex = Vertex::new(
            vertex.position.x,
            vertex.position.y + distance,
            vertex.position.z,
        );
        vertices.push(new_vertex);
        new_vertex_indices.push(vertices.len() - 1);
    }

    // Create new edges connecting original vertices to extruded vertices
    for i in 0..face.vertex_indices.len() {
        edges.push(Edge::new(
            face.vertex_indices[i],
            new_vertex_indices[i],
        ));
    }

    // Create new faces for the sides of the extrusion
    for i in 0..face.vertex_indices.len() {
        let j = (i + 1) % face.vertex_indices.len();
        faces.push(Face::quad(
            face.vertex_indices[i],
            face.vertex_indices[j],
            new_vertex_indices[j],
            new_vertex_indices[i],
        ));
    }

    // Create the top face
    faces.push(Face::new(new_vertex_indices));

    Some(())
}

/// Scales an object based on a center point and scale factors
pub fn scale_object(
    vertices: &mut [Vertex],
    center: Vector3<f32>,
    scale_x: f32,
    scale_y: f32,
    scale_z: f32,
) {
    for vertex in vertices.iter_mut() {
        // Calculate the offset from the center
        let offset = vertex.position - center;

        // Apply scaling
        let scaled_offset = Vector3::new(
            offset.x * scale_x,
            offset.y * scale_y,
            offset.z * scale_z,
        );

        // Update the vertex position
        vertex.position = center + scaled_offset;
    }
}

/// Rotates an object around an axis by the specified angle in radians
pub fn rotate_object(
    vertices: &mut [Vertex],
    center: Vector3<f32>,
    axis: Vector3<f32>,
    angle_rad: f32,
) {
    let axis = if axis.norm() != 0.0 {
        axis.normalize()
    } else {
        return; // Invalid rotation axis
    };

    let rotation = nalgebra::Rotation3::from_axis_angle(&nalgebra::Unit::new_normalize(axis), angle_rad);

    for vertex in vertices.iter_mut() {
        // Calculate the offset from the center
        let offset = vertex.position - center;

        // Apply rotation
        let rotated_offset = rotation * offset;

        // Update the vertex position
        vertex.position = center + rotated_offset;
    }
}
