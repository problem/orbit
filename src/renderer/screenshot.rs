use std::path::Path;

use anyhow::Result;
use wgpu::util::DeviceExt;

use super::camera::Camera;
use super::pipeline::{self, Uniforms};
use super::vertex::{GpuMesh, Vertex};
use crate::solver::types::SolvedBuilding;

/// Render a solved building to a PNG file using headless offscreen rendering.
/// Creates its own wgpu device — no window required.
pub fn render_building_to_png(
    building: &SolvedBuilding,
    camera: &Camera,
    width: u32,
    height: u32,
    output_path: &Path,
) -> Result<()> {
    render_building_to_png_opts(building, camera, width, height, output_path, false)
}

pub fn render_building_to_png_wireframe(
    building: &SolvedBuilding,
    camera: &Camera,
    width: u32,
    height: u32,
    output_path: &Path,
) -> Result<()> {
    render_building_to_png_opts(building, camera, width, height, output_path, true)
}

fn render_building_to_png_opts(
    building: &SolvedBuilding,
    camera: &Camera,
    width: u32,
    height: u32,
    output_path: &Path,
    wireframe: bool,
) -> Result<()> {
    use crate::solver::structure::generate_building_meshes;

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok_or_else(|| anyhow::anyhow!("no suitable GPU adapter for headless rendering"))?;

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("Screenshot Device"),
            required_features: wgpu::Features::POLYGON_MODE_LINE,
            required_limits: wgpu::Limits::default(),
            ..Default::default()
        },
        None,
    ))?;

    let texture_format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let bind_group_layout = pipeline::create_bind_group_layout(&device);
    let shadow_bind_group_layout = pipeline::create_shadow_bind_group_layout(&device);

    let render_pipeline = if wireframe {
        pipeline::create_pipeline_impl_pub(&device, texture_format, &bind_group_layout, Some(&shadow_bind_group_layout), wgpu::PolygonMode::Line)
    } else {
        pipeline::create_render_pipeline_with_shadow(&device, texture_format, &bind_group_layout, &shadow_bind_group_layout)
    };

    // Dummy shadow map for screenshot (no shadows in screenshots for now)
    let shadow_tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Dummy Shadow"),
        size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
        mip_level_count: 1, sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let shadow_view = shadow_tex.create_view(&wgpu::TextureViewDescriptor::default());
    let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        compare: Some(wgpu::CompareFunction::LessEqual),
        ..Default::default()
    });
    let shadow_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Screenshot Shadow BG"),
        layout: &shadow_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&shadow_view) },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&shadow_sampler) },
        ],
    });

    // Create offscreen render target + depth
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Screenshot Texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: texture_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Screenshot Depth"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

    // Generate building meshes and upload everything to THIS device
    let building_meshes = generate_building_meshes(building);
    let view_proj = camera.view_projection_matrix();

    struct GpuDrawable {
        gpu_mesh: GpuMesh,
        bind_group: wgpu::BindGroup,
        _uniform_buffer: wgpu::Buffer,
    }

    let mut drawables = Vec::new();
    for bm in &building_meshes {
        // Upload mesh to screenshot device
        let vertices: Vec<Vertex> = bm
            .mesh
            .positions
            .iter()
            .zip(bm.mesh.normals.iter())
            .map(|(p, n)| Vertex {
                position: *p,
                normal: *n,
            })
            .collect();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&bm.mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let gpu_mesh = GpuMesh {
            vertex_buffer,
            index_buffer,
            num_indices: bm.mesh.indices.len() as u32,
        };

        // Compute normal matrix
        let normal_mat = bm
            .model_matrix
            .try_inverse()
            .unwrap_or_else(nalgebra::Matrix4::identity)
            .transpose();

        let uniforms = Uniforms {
            view_proj: view_proj.into(),
            model: bm.model_matrix.into(),
            normal_matrix: normal_mat.into(),
            base_color: [bm.color[0], bm.color[1], bm.color[2], 1.0],
            light_view_proj: nalgebra::Matrix4::<f32>::identity().into(),
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        drawables.push(GpuDrawable {
            gpu_mesh,
            bind_group,
            _uniform_buffer: uniform_buffer,
        });
    }

    // Render
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenshot Encoder"),
    });

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Screenshot Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.15,
                        g: 0.15,
                        b: 0.18,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });

        render_pass.set_pipeline(&render_pipeline);
        render_pass.set_bind_group(1, &shadow_bind_group, &[]);
        for d in &drawables {
            render_pass.set_bind_group(0, &d.bind_group, &[]);
            render_pass.set_vertex_buffer(0, d.gpu_mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                d.gpu_mesh.index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            render_pass.draw_indexed(0..d.gpu_mesh.num_indices, 0, 0..1);
        }
    }

    // Copy texture to buffer for readback
    let bytes_per_row = align_to(width * 4, 256);
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Screenshot Readback"),
        size: (bytes_per_row * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &output_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    queue.submit(std::iter::once(encoder.finish()));

    // Map buffer and read pixels
    let buffer_slice = output_buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        tx.send(result).unwrap();
    });
    device.poll(wgpu::Maintain::Wait);
    rx.recv()??;

    let data = buffer_slice.get_mapped_range();
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for row in 0..height {
        let start = (row * bytes_per_row) as usize;
        let end = start + (width * 4) as usize;
        pixels.extend_from_slice(&data[start..end]);
    }
    drop(data);
    output_buffer.unmap();

    // Save PNG
    image::save_buffer(
        output_path,
        &pixels,
        width,
        height,
        image::ColorType::Rgba8,
    )?;

    log::info!(
        "Screenshot saved to {:?} ({}x{})",
        output_path,
        width,
        height
    );
    Ok(())
}

fn align_to(value: u32, alignment: u32) -> u32 {
    (value + alignment - 1) / alignment * alignment
}
