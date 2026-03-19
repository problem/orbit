use nalgebra::Matrix4;
use wgpu::util::DeviceExt;

use super::pipeline::Uniforms;
use super::vertex::GpuMesh;
use crate::orb::mesh::MeshData;
use crate::solver::types::SolvedBuilding;

/// A drawable object in the render scene with its own GPU uniform buffer.
pub struct DrawableMesh {
    pub gpu_mesh: GpuMesh,
    pub model_matrix: Matrix4<f32>,
    pub base_color: [f32; 3],
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl DrawableMesh {
    pub fn new(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        mesh: &MeshData,
        model_matrix: Matrix4<f32>,
        base_color: [f32; 3],
    ) -> Self {
        let gpu_mesh = GpuMesh::from_mesh_data(device, mesh);
        let uniforms = Uniforms::new();
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Drawable Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Drawable Bind Group"),
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });
        Self {
            gpu_mesh,
            model_matrix,
            base_color,
            uniform_buffer,
            bind_group,
        }
    }

    /// Compute the normal matrix (inverse-transpose of model matrix).
    pub fn normal_matrix(&self) -> Matrix4<f32> {
        self.model_matrix
            .try_inverse()
            .unwrap_or_else(Matrix4::identity)
            .transpose()
    }
}

/// The set of objects to render.
pub struct RenderScene {
    pub drawables: Vec<DrawableMesh>,
    /// Black edge outlines for wireframe overlay mode.
    pub edge_drawables: Vec<DrawableMesh>,
}

impl RenderScene {
    pub fn new() -> Self {
        Self {
            drawables: Vec::new(),
            edge_drawables: Vec::new(),
        }
    }

    /// Create a test scene with a few colored cubes.
    pub fn test_scene(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let mut scene = Self::new();

        // Main cube at origin
        let cube = MeshData::cube(1.0);
        scene.drawables.push(DrawableMesh::new(
            device,
            bind_group_layout,
            &cube,
            Matrix4::identity(),
            [0.6, 0.7, 0.85],
        ));

        // Smaller cube offset to the right
        let small_cube = MeshData::cube(0.5);
        scene.drawables.push(DrawableMesh::new(
            device,
            bind_group_layout,
            &small_cube,
            Matrix4::new_translation(&nalgebra::Vector3::new(2.0, 0.0, 0.0)),
            [0.85, 0.55, 0.35],
        ));

        // Ground plane (flat cube) — tests non-uniform scaling normal fix
        let ground = MeshData::cube(1.0);
        scene.drawables.push(DrawableMesh::new(
            device,
            bind_group_layout,
            &ground,
            Matrix4::new_nonuniform_scaling(&nalgebra::Vector3::new(5.0, 5.0, 0.05))
                * Matrix4::new_translation(&nalgebra::Vector3::new(0.0, 0.0, -0.55)),
            [0.4, 0.5, 0.4],
        ));

        scene
    }

    /// Create a scene from solver output.
    pub fn from_solved_building(
        building: &SolvedBuilding,
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        edge_thickness: f32,
    ) -> Self {
        use crate::solver::structure::{generate_building_meshes, generate_edge_meshes};

        let mut scene = Self::new();
        let building_meshes = generate_building_meshes(building);

        for bm in &building_meshes {
            scene.drawables.push(DrawableMesh::new(
                device,
                bind_group_layout,
                &bm.mesh,
                bm.model_matrix,
                bm.color,
            ));
        }

        // Generate black edge outlines for wireframe overlay (auto from all meshes)
        let edge_meshes = generate_edge_meshes(building, edge_thickness);
        for em in &edge_meshes {
            scene.edge_drawables.push(DrawableMesh::new(
                device,
                bind_group_layout,
                &em.mesh,
                em.model_matrix,
                em.color,
            ));
        }

        scene
    }
}

impl Default for RenderScene {
    fn default() -> Self {
        Self::new()
    }
}
