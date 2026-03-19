use std::sync::Arc;

use winit::window::Window;

use super::camera::{Camera, CameraController};
use super::pipeline::{self, Uniforms, SHADOW_MAP_SIZE};
use super::scene::RenderScene;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Solid,
    SolidWireframe,
    WireframeOnly,
}

impl ViewMode {
    pub fn next(self) -> Self {
        match self {
            Self::Solid => Self::SolidWireframe,
            Self::SolidWireframe => Self::WireframeOnly,
            Self::WireframeOnly => Self::Solid,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Solid => "Solid",
            Self::SolidWireframe => "Solid + Wireframe",
            Self::WireframeOnly => "Wireframe Only",
        }
    }
}

pub struct RenderState {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub render_pipeline: wgpu::RenderPipeline,
    pub wireframe_pipeline: wgpu::RenderPipeline,
    pub shadow_pipeline: wgpu::RenderPipeline,
    pub view_mode: ViewMode,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub shadow_bind_group: wgpu::BindGroup,
    pub shadow_depth_view: wgpu::TextureView,
    pub depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,
    pub camera: Camera,
    pub camera_controller: CameraController,
    _window: Arc<Window>,
}

/// Compute a light view-projection matrix for directional shadow mapping.
pub fn compute_light_vp(building_center: nalgebra::Point3<f32>, building_radius: f32) -> nalgebra::Matrix4<f32> {
    let sun_dir = nalgebra::Vector3::new(0.5f32, 0.8, 1.0).normalize();
    let light_pos = building_center + sun_dir * building_radius * 2.0;
    let view = nalgebra::Matrix4::look_at_rh(
        &light_pos,
        &building_center,
        &nalgebra::Vector3::new(0.0, 0.0, 1.0),
    );
    let r = building_radius * 1.5;
    // Orthographic projection for directional light
    let proj = nalgebra::Matrix4::new_orthographic(-r, r, -r, r, 0.1, building_radius * 5.0);
    // Remap Z from [-1,1] to [0,1] for wgpu
    #[rustfmt::skip]
    let correction = nalgebra::Matrix4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.5,
        0.0, 0.0, 0.0, 1.0,
    );
    correction * proj * view
}

impl RenderState {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find a suitable GPU adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Orbit Device"),
                    required_features: wgpu::Features::POLYGON_MODE_LINE,
                    required_limits: wgpu::Limits::default(),
                    ..Default::default()
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let (depth_texture, depth_view) = create_depth_texture(&device, &config);

        let bind_group_layout = pipeline::create_bind_group_layout(&device);
        let shadow_bind_group_layout = pipeline::create_shadow_bind_group_layout(&device);

        let render_pipeline = pipeline::create_render_pipeline_with_shadow(
            &device, surface_format, &bind_group_layout, &shadow_bind_group_layout,
        );
        // Wireframe pipeline also needs shadow layout since it uses the same basic.wgsl shader
        let wireframe_pipeline = pipeline::create_pipeline_impl_pub(
            &device, surface_format, &bind_group_layout, Some(&shadow_bind_group_layout), wgpu::PolygonMode::Line,
        );
        let shadow_pipeline =
            pipeline::create_shadow_pipeline(&device, &bind_group_layout);

        // Shadow map
        let shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Map"),
            size: wgpu::Extent3d { width: SHADOW_MAP_SIZE, height: SHADOW_MAP_SIZE, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let shadow_depth_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shadow Sampler"),
            compare: Some(wgpu::CompareFunction::LessEqual),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let shadow_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shadow Bind Group"),
            layout: &shadow_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&shadow_depth_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&shadow_sampler) },
            ],
        });

        let camera = Camera::new(size.width as f32 / size.height.max(1) as f32);
        let camera_controller = CameraController::new();

        Self {
            surface, device, queue, config, size,
            render_pipeline, wireframe_pipeline, shadow_pipeline,
            view_mode: ViewMode::Solid,
            bind_group_layout,
            shadow_bind_group,
            shadow_depth_view,
            depth_texture, depth_view,
            camera, camera_controller,
            _window: window,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            let (dt, dv) = create_depth_texture(&self.device, &self.config);
            self.depth_texture = dt;
            self.depth_view = dv;
            self.camera.aspect = new_size.width as f32 / new_size.height as f32;
        }
    }

    pub fn render(&mut self, scene: &RenderScene, light_vp: &nalgebra::Matrix4<f32>) -> Result<(), wgpu::SurfaceError> {
        self.camera_controller.update_camera(&mut self.camera);
        let view_proj = self.camera.view_projection_matrix();

        // Upload uniforms
        for drawable in &scene.drawables {
            let normal_mat = drawable.normal_matrix();
            let uniforms = Uniforms {
                view_proj: view_proj.into(),
                model: drawable.model_matrix.into(),
                normal_matrix: normal_mat.into(),
                base_color: [drawable.base_color[0], drawable.base_color[1], drawable.base_color[2], 1.0],
                light_view_proj: (*light_vp).into(),
            };
            self.queue.write_buffer(&drawable.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
        }

        // Also upload for edge drawables
        for drawable in &scene.edge_drawables {
            let normal_mat = drawable.normal_matrix();
            let uniforms = Uniforms {
                view_proj: view_proj.into(),
                model: drawable.model_matrix.into(),
                normal_matrix: normal_mat.into(),
                base_color: [drawable.base_color[0], drawable.base_color[1], drawable.base_color[2], 1.0],
                light_view_proj: (*light_vp).into(),
            };
            self.queue.write_buffer(&drawable.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
        }

        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Shadow pass
        {
            let mut shadow_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Shadow Pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow_depth_view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            shadow_pass.set_pipeline(&self.shadow_pipeline);
            for drawable in &scene.drawables {
                shadow_pass.set_bind_group(0, &drawable.bind_group, &[]);
                shadow_pass.set_vertex_buffer(0, drawable.gpu_mesh.vertex_buffer.slice(..));
                shadow_pass.set_index_buffer(drawable.gpu_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                shadow_pass.draw_indexed(0..drawable.gpu_mesh.num_indices, 0, 0..1);
            }
        }

        // Main render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.15, g: 0.15, b: 0.18, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            match self.view_mode {
                ViewMode::Solid => {
                    render_pass.set_pipeline(&self.render_pipeline);
                    render_pass.set_bind_group(1, &self.shadow_bind_group, &[]);
                    for drawable in &scene.drawables {
                        render_pass.set_bind_group(0, &drawable.bind_group, &[]);
                        render_pass.set_vertex_buffer(0, drawable.gpu_mesh.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(drawable.gpu_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..drawable.gpu_mesh.num_indices, 0, 0..1);
                    }
                }
                ViewMode::SolidWireframe => {
                    render_pass.set_pipeline(&self.render_pipeline);
                    render_pass.set_bind_group(1, &self.shadow_bind_group, &[]);
                    for drawable in &scene.drawables {
                        render_pass.set_bind_group(0, &drawable.bind_group, &[]);
                        render_pass.set_vertex_buffer(0, drawable.gpu_mesh.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(drawable.gpu_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..drawable.gpu_mesh.num_indices, 0, 0..1);
                    }
                    // Edge geometry overlay (uses solid pipeline, no shadow bind group needed since edges are black)
                    for drawable in &scene.edge_drawables {
                        render_pass.set_bind_group(0, &drawable.bind_group, &[]);
                        render_pass.set_vertex_buffer(0, drawable.gpu_mesh.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(drawable.gpu_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..drawable.gpu_mesh.num_indices, 0, 0..1);
                    }
                }
                ViewMode::WireframeOnly => {
                    render_pass.set_pipeline(&self.render_pipeline);
                    render_pass.set_bind_group(1, &self.shadow_bind_group, &[]);
                    for drawable in &scene.edge_drawables {
                        render_pass.set_bind_group(0, &drawable.bind_group, &[]);
                        render_pass.set_vertex_buffer(0, drawable.gpu_mesh.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(drawable.gpu_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..drawable.gpu_mesh.num_indices, 0, 0..1);
                    }
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

fn create_depth_texture(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth Texture"),
        size: wgpu::Extent3d { width: config.width.max(1), height: config.height.max(1), depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}
