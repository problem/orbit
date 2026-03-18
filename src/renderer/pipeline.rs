use super::vertex::Vertex;

/// Uniform buffer layout matching the shader.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    pub view_proj: [[f32; 4]; 4],
    pub model: [[f32; 4]; 4],
    /// Inverse-transpose of model matrix for correct normal transformation
    /// under non-uniform scaling. Stored as mat4x4 for alignment; only the
    /// upper-left 3x3 is used in the shader.
    pub normal_matrix: [[f32; 4]; 4],
    pub base_color: [f32; 4],
}

impl Uniforms {
    pub fn new() -> Self {
        Self {
            view_proj: nalgebra::Matrix4::<f32>::identity().into(),
            model: nalgebra::Matrix4::<f32>::identity().into(),
            normal_matrix: nalgebra::Matrix4::<f32>::identity().into(),
            base_color: [0.8, 0.8, 0.8, 1.0],
        }
    }
}

impl Default for Uniforms {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Uniform Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

pub fn create_render_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    create_pipeline_with_mode(device, format, bind_group_layout, wgpu::PolygonMode::Fill)
}

pub fn create_wireframe_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    create_pipeline_with_mode(device, format, bind_group_layout, wgpu::PolygonMode::Line)
}

/// Uniform buffer with black color for wireframe overlay pass.
pub fn black_uniforms(view_proj: [[f32; 4]; 4], model: [[f32; 4]; 4], normal_matrix: [[f32; 4]; 4]) -> Uniforms {
    Uniforms {
        view_proj,
        model,
        normal_matrix,
        base_color: [0.0, 0.0, 0.0, 1.0],
    }
}

fn create_pipeline_with_mode(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    bind_group_layout: &wgpu::BindGroupLayout,
    polygon_mode: wgpu::PolygonMode,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Basic Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/basic.wgsl").into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[Vertex::desc()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None, // disabled — render all faces from all angles
            polygon_mode,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: polygon_mode == wgpu::PolygonMode::Fill,
            depth_compare: if polygon_mode == wgpu::PolygonMode::Line {
                wgpu::CompareFunction::LessEqual // wireframe draws on top of solid
            } else {
                wgpu::CompareFunction::Less
            },
            stencil: wgpu::StencilState::default(),
            bias: if polygon_mode == wgpu::PolygonMode::Line {
                wgpu::DepthBiasState {
                    constant: -2, // push wireframe slightly toward camera
                    slope_scale: -2.0,
                    clamp: 0.0,
                }
            } else {
                wgpu::DepthBiasState::default()
            },
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    })
}
