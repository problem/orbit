use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::window::CursorGrabMode;
use std::f32::consts::PI;

pub struct OrbitCameraPlugin;

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<OrbitCameraState>()
            .add_systems(Startup, setup_camera)
            .add_systems(Update, (orbit_camera_controller, camera_zoom_controller));
    }
}

#[derive(Default, Resource)]
pub struct OrbitCameraState {
    pub active: bool,
    pub sensitivity: f32,
    pub orbit_distance: f32,
    pub orbit_target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Component)]
pub struct OrbitCamera;

fn setup_camera(
    mut commands: Commands,
    mut orbit_camera_state: ResMut<OrbitCameraState>,
) {
    // Initialize camera state
    *orbit_camera_state = OrbitCameraState {
        active: false,
        sensitivity: 0.005,
        orbit_distance: 10.0,
        orbit_target: Vec3::new(0.0, 0.0, 0.0),
        yaw: -PI / 2.0, // Start looking at the negative z axis
        pitch: PI / 4.0, // Look slightly down
    };

    // Setup camera entity
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(
                orbit_camera_state.orbit_distance * orbit_camera_state.yaw.cos() * orbit_camera_state.pitch.cos(),
                orbit_camera_state.orbit_distance * orbit_camera_state.pitch.sin(),
                orbit_camera_state.orbit_distance * orbit_camera_state.yaw.sin() * orbit_camera_state.pitch.cos(),
            )
            .looking_at(orbit_camera_state.orbit_target, Vec3::Y),
            ..default()
        },
        OrbitCamera,
    ));
}

fn orbit_camera_controller(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut windows: Query<&mut Window>,
    keyboard: Res<Input<KeyCode>>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut orbit_camera_state: ResMut<OrbitCameraState>,
    mut query: Query<&mut Transform, With<OrbitCamera>>,
) {
    let mut window = windows.single_mut();

    // Activate/deactivate orbit camera with middle mouse button
    if mouse_buttons.just_pressed(MouseButton::Middle) {
        orbit_camera_state.active = true;
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
    }

    if mouse_buttons.just_released(MouseButton::Middle) {
        orbit_camera_state.active = false;
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
    }

    // Reset camera position with 'R' key
    if keyboard.just_pressed(KeyCode::R) {
        orbit_camera_state.yaw = -PI / 2.0;
        orbit_camera_state.pitch = PI / 4.0;
        orbit_camera_state.orbit_distance = 10.0;
        orbit_camera_state.orbit_target = Vec3::new(0.0, 0.0, 0.0);
    }

    // Pan camera with arrow keys
    let pan_speed = 0.1;
    let forward = Vec3::new(
        orbit_camera_state.yaw.cos(),
        0.0,
        orbit_camera_state.yaw.sin(),
    ).normalize();
    let right = forward.cross(Vec3::Y).normalize();

    if keyboard.pressed(KeyCode::Up) {
        orbit_camera_state.orbit_target += forward * pan_speed;
    }
    if keyboard.pressed(KeyCode::Down) {
        orbit_camera_state.orbit_target -= forward * pan_speed;
    }
    if keyboard.pressed(KeyCode::Left) {
        orbit_camera_state.orbit_target -= right * pan_speed;
    }
    if keyboard.pressed(KeyCode::Right) {
        orbit_camera_state.orbit_target += right * pan_speed;
    }

    // Process orbit camera rotation
    if orbit_camera_state.active {
        for event in mouse_motion_events.read() {
            orbit_camera_state.yaw -= event.delta.x * orbit_camera_state.sensitivity;
            orbit_camera_state.pitch += event.delta.y * orbit_camera_state.sensitivity;
            orbit_camera_state.pitch = orbit_camera_state.pitch.clamp(-PI / 2.0 + 0.1, PI / 2.0 - 0.1);
        }
    }

    // Apply camera transformation
    if let Ok(mut transform) = query.get_single_mut() {
        let new_position = Vec3::new(
            orbit_camera_state.orbit_target.x + orbit_camera_state.orbit_distance * orbit_camera_state.yaw.cos() * orbit_camera_state.pitch.cos(),
            orbit_camera_state.orbit_target.y + orbit_camera_state.orbit_distance * orbit_camera_state.pitch.sin(),
            orbit_camera_state.orbit_target.z + orbit_camera_state.orbit_distance * orbit_camera_state.yaw.sin() * orbit_camera_state.pitch.cos(),
        );

        *transform = Transform::from_translation(new_position)
            .looking_at(orbit_camera_state.orbit_target, Vec3::Y);
    }
}

fn camera_zoom_controller(
    mut scroll_events: EventReader<MouseWheel>,
    mut orbit_camera_state: ResMut<OrbitCameraState>,
) {
    for event in scroll_events.read() {
        orbit_camera_state.orbit_distance -= event.y * 0.5;
        orbit_camera_state.orbit_distance = orbit_camera_state.orbit_distance.clamp(1.0, 30.0);
    }
}
