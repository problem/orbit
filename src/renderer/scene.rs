use nalgebra::Matrix4;

use super::vertex::GpuMesh;
use crate::orb::mesh::MeshData;

/// A drawable object in the render scene.
pub struct DrawableMesh {
    pub gpu_mesh: GpuMesh,
    pub model_matrix: Matrix4<f32>,
    pub base_color: [f32; 3],
}

/// The set of objects to render.
pub struct RenderScene {
    pub drawables: Vec<DrawableMesh>,
}

impl RenderScene {
    pub fn new() -> Self {
        Self {
            drawables: Vec::new(),
        }
    }

    /// Create a test scene with a few colored cubes.
    pub fn test_scene(device: &wgpu::Device) -> Self {
        let mut scene = Self::new();

        // Main cube at origin
        let cube = MeshData::cube(1.0);
        scene.drawables.push(DrawableMesh {
            gpu_mesh: GpuMesh::from_mesh_data(device, &cube),
            model_matrix: Matrix4::identity(),
            base_color: [0.6, 0.7, 0.85], // steel blue
        });

        // Smaller cube offset to the right
        let small_cube = MeshData::cube(0.5);
        scene.drawables.push(DrawableMesh {
            gpu_mesh: GpuMesh::from_mesh_data(device, &small_cube),
            model_matrix: Matrix4::new_translation(&nalgebra::Vector3::new(2.0, 0.0, 0.0)),
            base_color: [0.85, 0.55, 0.35], // warm orange
        });

        // Ground plane (flat cube)
        let ground = MeshData::cube(1.0);
        scene.drawables.push(DrawableMesh {
            gpu_mesh: GpuMesh::from_mesh_data(device, &ground),
            model_matrix: Matrix4::new_nonuniform_scaling(&nalgebra::Vector3::new(5.0, 5.0, 0.05))
                * Matrix4::new_translation(&nalgebra::Vector3::new(0.0, 0.0, -0.55)),
            base_color: [0.4, 0.5, 0.4], // muted green
        });

        scene
    }
}

impl Default for RenderScene {
    fn default() -> Self {
        Self::new()
    }
}
