use std::sync::Arc;

use winit::window::Window;

use super::camera::{Camera, CameraController};
use super::pipeline::{self, Uniforms};
use super::scene::RenderScene;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    /// Normal solid rendering
    Solid,
    /// Solid fill + dark wireframe edges overlaid
    SolidWireframe,
    /// Wireframe only (see-through, inspect interior geometry)
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
    pub view_mode: ViewMode,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,
    pub camera: Camera,
    pub camera_controller: CameraController,
    _window: Arc<Window>,
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
        let render_pipeline =
            pipeline::create_render_pipeline(&device, surface_format, &bind_group_layout);
        let wireframe_pipeline =
            pipeline::create_wireframe_pipeline(&device, surface_format, &bind_group_layout);

        let camera = Camera::new(size.width as f32 / size.height.max(1) as f32);
        let camera_controller = CameraController::new();

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            wireframe_pipeline,
            view_mode: ViewMode::Solid,
            bind_group_layout,
            depth_texture,
            depth_view,
            camera,
            camera_controller,
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

    pub fn render(&mut self, scene: &RenderScene) -> Result<(), wgpu::SurfaceError> {
        self.camera_controller.update_camera(&mut self.camera);
        let view_proj = self.camera.view_projection_matrix();

        // Upload uniforms with normal colors
        for drawable in &scene.drawables {
            let normal_mat = drawable.normal_matrix();
            let uniforms = Uniforms {
                view_proj: view_proj.into(),
                model: drawable.model_matrix.into(),
                normal_matrix: normal_mat.into(),
                base_color: [
                    drawable.base_color[0],
                    drawable.base_color[1],
                    drawable.base_color[2],
                    1.0,
                ],
            };
            self.queue.write_buffer(
                &drawable.uniform_buffer,
                0,
                bytemuck::cast_slice(&[uniforms]),
            );
        }

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            match self.view_mode {
                ViewMode::Solid => {
                    render_pass.set_pipeline(&self.render_pipeline);
                    for drawable in &scene.drawables {
                        render_pass.set_bind_group(0, &drawable.bind_group, &[]);
                        render_pass.set_vertex_buffer(0, drawable.gpu_mesh.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(drawable.gpu_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..drawable.gpu_mesh.num_indices, 0, 0..1);
                    }
                }
                ViewMode::SolidWireframe => {
                    // Solid fill
                    render_pass.set_pipeline(&self.render_pipeline);
                    for drawable in &scene.drawables {
                        render_pass.set_bind_group(0, &drawable.bind_group, &[]);
                        render_pass.set_vertex_buffer(0, drawable.gpu_mesh.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(drawable.gpu_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..drawable.gpu_mesh.num_indices, 0, 0..1);
                    }
                    // Wireframe overlay — reuse same uniforms (no buffer overwrite).
                    // Depth bias makes lines draw on top. Edges visible from lighting
                    // difference between wireframe (re-interpolated) and solid fill.
                    let px = 2.0 / self.config.width as f32;
                    let py = 2.0 / self.config.height as f32;
                    let offsets: [[f32; 2]; 13] = [
                        [0.0, 0.0], [px, 0.0], [-px, 0.0], [0.0, py], [0.0, -py],
                        [px, py], [-px, py], [px, -py], [-px, -py],
                        [2.0*px, 0.0], [-2.0*px, 0.0], [0.0, 2.0*py], [0.0, -2.0*py],
                    ];
                    render_pass.set_pipeline(&self.wireframe_pipeline);
                    for [ox, oy] in offsets {
                        let mut shifted_vp = view_proj;
                        shifted_vp[(0, 3)] += ox;
                        shifted_vp[(1, 3)] += oy;
                        for drawable in &scene.drawables {
                            let normal_mat = drawable.normal_matrix();
                            let u = Uniforms {
                                view_proj: shifted_vp.into(),
                                model: drawable.model_matrix.into(),
                                normal_matrix: normal_mat.into(),
                                base_color: [0.0, 0.0, 0.0, 0.5],
                            };
                            self.queue.write_buffer(&drawable.uniform_buffer, 0, bytemuck::cast_slice(&[u]));
                            render_pass.set_bind_group(0, &drawable.bind_group, &[]);
                            render_pass.set_vertex_buffer(0, drawable.gpu_mesh.vertex_buffer.slice(..));
                            render_pass.set_index_buffer(drawable.gpu_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                            render_pass.draw_indexed(0..drawable.gpu_mesh.num_indices, 0, 0..1);
                        }
                    }
                    // Restore original uniforms for next frame
                    for drawable in &scene.drawables {
                        let normal_mat = drawable.normal_matrix();
                        let orig = Uniforms {
                            view_proj: view_proj.into(),
                            model: drawable.model_matrix.into(),
                            normal_matrix: normal_mat.into(),
                            base_color: [drawable.base_color[0], drawable.base_color[1], drawable.base_color[2], 1.0],
                        };
                        self.queue.write_buffer(&drawable.uniform_buffer, 0, bytemuck::cast_slice(&[orig]));
                    }
                }
                ViewMode::WireframeOnly => {
                    // Same multi-pass offset for thick lines
                    let px = 2.0 / self.config.width as f32;
                    let py = 2.0 / self.config.height as f32;
                    let offsets = [
                        [0.0, 0.0], [px, 0.0], [-px, 0.0], [0.0, py], [0.0, -py],
                        [px, py], [-px, py], [px, -py], [-px, -py],
                        [2.0*px, 0.0], [-2.0*px, 0.0], [0.0, 2.0*py], [0.0, -2.0*py],
                    ];

                    render_pass.set_pipeline(&self.wireframe_pipeline);
                    for [ox, oy] in offsets {
                        let mut shifted_vp = view_proj;
                        shifted_vp[(0, 3)] += ox;
                        shifted_vp[(1, 3)] += oy;

                        for drawable in &scene.drawables {
                            let normal_mat = drawable.normal_matrix();
                            let uniforms = Uniforms {
                                view_proj: shifted_vp.into(),
                                model: drawable.model_matrix.into(),
                                normal_matrix: normal_mat.into(),
                                base_color: [
                                    drawable.base_color[0],
                                    drawable.base_color[1],
                                    drawable.base_color[2],
                                    1.0,
                                ],
                            };
                            self.queue.write_buffer(&drawable.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
                            render_pass.set_bind_group(0, &drawable.bind_group, &[]);
                            render_pass.set_vertex_buffer(0, drawable.gpu_mesh.vertex_buffer.slice(..));
                            render_pass.set_index_buffer(drawable.gpu_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                            render_pass.draw_indexed(0..drawable.gpu_mesh.num_indices, 0, 0..1);
                        }
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
        size: wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        },
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
