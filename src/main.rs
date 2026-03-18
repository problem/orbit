use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use orbit::renderer::scene::RenderScene;
use orbit::renderer::state::RenderState;

fn main() {
    env_logger::init();

    // Validate the Orb file I/O pipeline
    validate_orb_pipeline();

    // Validate the OIL parser
    validate_oil_parser();

    // Launch the renderer
    log::info!("Starting Orbit CAD");
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = App::default();
    event_loop.run_app(&mut app).expect("Event loop failed");
}

/// Create a test .orb file, write a cube, read it back.
fn validate_orb_pipeline() {
    use orbit::orb::{mesh::MeshData, read::OrbReader, types::*, write::OrbWriter};

    let path = std::env::temp_dir().join("orbit_test.orb");
    log::info!("Creating test .orb file at {:?}", path);

    // Write
    let writer = OrbWriter::create(&path).expect("Failed to create .orb");
    let mut entity = Entity::new(EntityType::Body);
    entity.name = Some("Test Cube".to_string());
    let entity_id = entity.id;
    writer.insert_entity(&entity).expect("Failed to insert entity");

    let cube = MeshData::cube(1.0);
    writer
        .insert_mesh(&entity_id, &cube)
        .expect("Failed to insert mesh");

    let mat = Material::new("Default", "6B8EAD");
    writer.insert_material(&mat).expect("Failed to insert material");
    writer.finalize().expect("Failed to finalize .orb");

    // Read
    let reader = OrbReader::open(&path).expect("Failed to open .orb");
    let meta = reader.read_meta().expect("Failed to read meta");
    log::info!("  format_version: {}", meta.get("format_version").unwrap());
    log::info!("  created_by: {}", meta.get("created_by").unwrap());

    let entities = reader.read_entities().expect("Failed to read entities");
    log::info!("  entities: {}", entities.len());

    let mesh = reader
        .read_mesh(&entity_id)
        .expect("Failed to read mesh")
        .expect("Mesh not found");
    log::info!(
        "  mesh: {} vertices, {} triangles",
        mesh.vertex_count(),
        mesh.triangle_count()
    );

    let _ = std::fs::remove_file(&path);
    log::info!("Orb file I/O pipeline validated successfully");
}

/// Parse a canonical OIL example.
fn validate_oil_parser() {
    let source = r#"
house "Meadowbrook Tudor" {
    site {
        footprint: 12m x 9m
        orientation: north
        setback: front 6m, sides 3m
    }

    style tudor {
        roof_pitch: 12:12
        facade_material: stucco("cream")
        accent_material: timber("dark oak")
        window_style: casement(mullioned, divided_lite: 6)
    }

    floor ground {
        room entry      { area: ~6sqm, connects: [living, dining], has: front_door }
        room living     { area: ~25sqm, aspect: 1.5, windows: south 2, has: fireplace }
        room kitchen    { area: ~15sqm, adjacent_to: living, windows: east 1, has: island }
        room dining     { area: ~12sqm, adjacent_to: [kitchen, living], windows: south 1 }
        room half_bath  { area: ~4sqm, adjacent_to: kitchen }
        room garage     { area: ~35sqm, side: west, has: garage_double }
    }

    floor upper {
        room master_bed  { area: ~18sqm, windows: south 2, has: walk_in_closet }
        room master_bath { area: ~8sqm, adjacent_to: master_bed, has: [shower, tub, double_vanity] }
        room bedroom_2   { area: ~13sqm, windows: north 1, has: closet }
        room bedroom_3   { area: ~12sqm, windows: east 1, has: closet }
        room full_bath   { area: ~6sqm, adjacent_to: [bedroom_2, bedroom_3], has: [shower, tub] }
        room hallway     { connects: [master_bed, bedroom_2, bedroom_3, full_bath] }
    }

    roof {
        primary: gable(ridge: east-west)
        cross_gable: over entry, pitch: 10:12
        dormers: 2, over [bedroom_2, bedroom_3]
    }
}
"#;

    match orbit::oil::parser::parse_oil(source) {
        Ok(program) => {
            log::info!("OIL parser validated successfully");
            match &program {
                orbit::oil::ast::Program::House(h) => {
                    log::info!("  house: {:?}", h.name);
                    if let Some(ref style) = h.style {
                        log::info!("  style: {}", style.name);
                        for prop in &style.overrides {
                            log::info!("    {}: {:?}", prop.key, prop.value);
                        }
                    }
                    log::info!("  floors: {}", h.floors.len());
                    for floor in &h.floors {
                        log::info!("    floor '{}': {} rooms", floor.name, floor.rooms.len());
                        for room in &floor.rooms {
                            log::info!("      room '{}': area={:?}", room.name, room.area);
                        }
                    }
                    if let Some(ref roof) = h.roof {
                        log::info!("  roof.primary: {:?}", roof.primary);
                        log::info!("  roof.cross_gable: {:?}", roof.cross_gable);
                        log::info!("  roof.dormers: {:?}", roof.dormers);
                    }
                }
                orbit::oil::ast::Program::Furniture(f) => {
                    log::info!("  furniture: {}", f.name);
                }
            }
        }
        Err(e) => {
            log::error!("OIL parse error: {}", e);
        }
    }
}

// --- winit Application ---

#[derive(Default)]
struct App {
    state: Option<AppState>,
}

struct AppState {
    window: Arc<Window>,
    render_state: RenderState,
    scene: RenderScene,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let window_attrs = Window::default_attributes()
            .with_title("Orbit CAD")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));
        let window = Arc::new(event_loop.create_window(window_attrs).unwrap());

        let render_state = pollster::block_on(RenderState::new(window.clone()));
        let scene = RenderScene::test_scene(&render_state.device, &render_state.bind_group_layout);

        self.state = Some(AppState {
            window,
            render_state,
            scene,
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = self.state.as_mut() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                state.render_state.resize(physical_size);
                state.window.request_redraw();
            }
            WindowEvent::MouseInput {
                state: button_state,
                button: MouseButton::Left,
                ..
            } => {
                match button_state {
                    ElementState::Pressed => state.render_state.camera_controller.on_mouse_press(),
                    ElementState::Released => {
                        state.render_state.camera_controller.on_mouse_release()
                    }
                }
                state.window.request_redraw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                state
                    .render_state
                    .camera_controller
                    .on_mouse_move(position.x, position.y);
                if state.render_state.camera_controller.is_dragging {
                    state.window.request_redraw();
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 50.0,
                };
                state.render_state.camera_controller.on_scroll(scroll);
                state.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                match state.render_state.render(&state.scene) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = state.render_state.size;
                        state.render_state.resize(size);
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        log::error!("Out of GPU memory");
                        event_loop.exit();
                    }
                    Err(e) => {
                        log::warn!("Surface error: {:?}", e);
                    }
                }
            }
            _ => {}
        }
    }
}
