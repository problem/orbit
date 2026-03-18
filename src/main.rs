use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use orbit::renderer::camera::CameraController;
use orbit::renderer::scene::RenderScene;
use orbit::renderer::state::RenderState;

const TUDOR_OIL: &str = r#"
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

fn main() {
    env_logger::init();

    // Kill any previously running orbit instance so we don't accumulate processes
    kill_previous_orbit();

    log::info!("Starting Orbit CAD");
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = App::default();
    event_loop.run_app(&mut app).expect("Event loop failed");
}

/// Kill any other running orbit process (by name, excluding ourselves).
fn kill_previous_orbit() {
    let my_pid = std::process::id();
    let output = std::process::Command::new("pgrep")
        .args(["-x", "orbit"])
        .output();
    if let Ok(output) = output {
        let pids = String::from_utf8_lossy(&output.stdout);
        for line in pids.lines() {
            if let Ok(pid) = line.trim().parse::<u32>() {
                if pid != my_pid {
                    log::info!("Killing previous orbit process (PID {})", pid);
                    let _ = std::process::Command::new("kill")
                        .arg(pid.to_string())
                        .status();
                }
            }
        }
    }
}

/// Find the next sequential screenshot path: screenshots/NNN_YYYY-MM-DD.png
fn next_screenshot_path() -> std::path::PathBuf {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("screenshots");
    let _ = std::fs::create_dir_all(&dir);

    // Find highest existing number
    let mut max_num = 0u32;
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if let Some(num_str) = name.split('_').next() {
                if let Ok(n) = num_str.parse::<u32>() {
                    max_num = max_num.max(n);
                }
            }
        }
    }

    let next = max_num + 1;
    let date = chrono::Local::now().format("%Y-%m-%d");
    dir.join(format!("{:03}_{}.png", next, date))
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

        let mut render_state = pollster::block_on(RenderState::new(window.clone()));

        // Parse OIL
        let program = orbit::oil::parser::parse_oil(TUDOR_OIL).expect("OIL parse failed");

        // Solve
        let building = orbit::solver::solve(&program).expect("Solver failed");
        for d in &building.diagnostics {
            log::warn!("solver: [{:?}] {}", d.level, d.message);
        }
        log::info!(
            "Solved: {:.1}m x {:.1}m footprint, {} floors, {} total rooms",
            building.footprint_width,
            building.footprint_depth,
            building.floors.len(),
            building.floors.iter().map(|f| f.rooms.len()).sum::<usize>(),
        );

        // Build scene from solver output
        let scene = RenderScene::from_solved_building(
            &building,
            &render_state.device,
            &render_state.bind_group_layout,
        );
        log::info!("Scene: {} drawables", scene.drawables.len());

        // Adjust camera for building scale
        let total_height: f64 = building.floors.iter().map(|f| f.ceiling_height + building.style.floor_thickness).sum();
        let diag = ((building.footprint_width.powi(2)
            + building.footprint_depth.powi(2)
            + total_height.powi(2)) as f32)
            .sqrt();
        render_state.camera_controller = CameraController::for_building(diag);
        render_state.camera.target = nalgebra::Point3::new(0.0, 0.0, total_height as f32 / 2.0);
        render_state.camera_controller.update_camera(&mut render_state.camera);

        // Export screenshot for QA — sequentially numbered in repo
        let screenshot_path = next_screenshot_path();
        if let Err(e) = orbit::renderer::screenshot::render_building_to_png(
            &building,
            &render_state.camera,
            1920,
            1080,
            &screenshot_path,
        ) {
            log::warn!("Screenshot export failed: {}", e);
        }

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
