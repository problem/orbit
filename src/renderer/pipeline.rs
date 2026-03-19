use super::vertex::Vertex;

/// Uniform buffer layout matching the shader.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    pub view_proj: [[f32; 4]; 4],
    pub model: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 4]; 4],
    pub base_color: [f32; 4],
    pub light_view_proj: [[f32; 4]; 4],
}

impl Uniforms {
    pub fn new() -> Self {
        let id = nalgebra::Matrix4::<f32>::identity().into();
        Self {
            view_proj: id,
            model: id,
            normal_matrix: id,
            base_color: [0.8, 0.8, 0.8, 1.0],
            light_view_proj: id,
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

pub fn create_shadow_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Shadow Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                count: None,
            },
        ],
    })
}

pub const SHADOW_MAP_SIZE: u32 = 2048;

pub fn create_shadow_pipeline(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shadow Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/shadow.wgsl").into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Shadow Pipeline Layout"),
        bind_group_layouts: &[bind_group_layout],
        push_constant_ranges: &[],
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Shadow Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[Vertex::desc()],
            compilation_options: Default::default(),
        },
        fragment: None, // depth-only pass
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState {
                constant: 2,
                slope_scale: 2.0,
                clamp: 0.0,
            },
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}

pub fn create_render_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    create_pipeline_impl(device, format, bind_group_layout, None, wgpu::PolygonMode::Fill)
}

pub fn create_wireframe_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    create_pipeline_impl(device, format, bind_group_layout, None, wgpu::PolygonMode::Line)
}

/// Uniform buffer with black color for wireframe overlay pass.
pub fn black_uniforms(view_proj: [[f32; 4]; 4], model: [[f32; 4]; 4], normal_matrix: [[f32; 4]; 4]) -> Uniforms {
    Uniforms {
        view_proj,
        model,
        normal_matrix,
        base_color: [0.0, 0.0, 0.0, 1.0],
        light_view_proj: nalgebra::Matrix4::<f32>::identity().into(),
    }
}

pub fn create_render_pipeline_with_shadow(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    bind_group_layout: &wgpu::BindGroupLayout,
    shadow_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    create_pipeline_impl(device, format, bind_group_layout, Some(shadow_bind_group_layout), wgpu::PolygonMode::Fill)
}

fn create_pipeline_with_mode(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    bind_group_layout: &wgpu::BindGroupLayout,
    polygon_mode: wgpu::PolygonMode,
) -> wgpu::RenderPipeline {
    create_pipeline_impl(device, format, bind_group_layout, None, polygon_mode)
}

pub fn create_pipeline_impl_pub(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    bind_group_layout: &wgpu::BindGroupLayout,
    shadow_bind_group_layout: Option<&wgpu::BindGroupLayout>,
    polygon_mode: wgpu::PolygonMode,
) -> wgpu::RenderPipeline {
    create_pipeline_impl(device, format, bind_group_layout, shadow_bind_group_layout, polygon_mode)
}

fn create_pipeline_impl(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    bind_group_layout: &wgpu::BindGroupLayout,
    shadow_bind_group_layout: Option<&wgpu::BindGroupLayout>,
    polygon_mode: wgpu::PolygonMode,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Basic Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/basic.wgsl").into()),
    });

    let layouts: Vec<&wgpu::BindGroupLayout> = if let Some(sbl) = shadow_bind_group_layout {
        vec![bind_group_layout, sbl]
    } else {
        vec![bind_group_layout]
    };
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &layouts,
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
