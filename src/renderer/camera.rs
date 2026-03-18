use nalgebra::{Matrix4, Point3, Vector3};

pub struct Camera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            eye: Point3::new(3.0, 3.0, 3.0),
            target: Point3::new(0.0, 0.0, 0.0),
            up: Vector3::new(0.0, 0.0, 1.0), // Z-up per Orb spec default
            fov: 45.0_f32.to_radians(),
            aspect,
            near: 0.1,
            far: 1000.0,
        }
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(&self.eye, &self.target, &self.up)
    }

    pub fn projection_matrix(&self) -> Matrix4<f32> {
        // wgpu uses 0..1 depth range (like Vulkan/Metal), nalgebra's
        // perspective uses -1..1 (OpenGL). We need to remap.
        let proj = Matrix4::new_perspective(self.aspect, self.fov, self.near, self.far);
        // Remap from OpenGL [-1,1] Z to wgpu [0,1] Z
        #[rustfmt::skip]
        let correction = Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.5,
            0.0, 0.0, 0.0, 1.0,
        );
        correction * proj
    }

    pub fn view_projection_matrix(&self) -> Matrix4<f32> {
        self.projection_matrix() * self.view_matrix()
    }
}

/// Arcball orbit camera controller.
pub struct CameraController {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub is_dragging: bool,
    pub last_mouse: Option<(f64, f64)>,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            yaw: std::f32::consts::FRAC_PI_4,    // 45°
            pitch: std::f32::consts::FRAC_PI_6,   // 30°
            distance: 5.0,
            is_dragging: false,
            last_mouse: None,
        }
    }

    /// Create a controller sized for a building with the given diagonal.
    pub fn for_building(diagonal: f32) -> Self {
        Self {
            yaw: std::f32::consts::FRAC_PI_4,
            pitch: std::f32::consts::FRAC_PI_6,
            distance: diagonal * 1.5,
            is_dragging: false,
            last_mouse: None,
        }
    }

    pub fn on_mouse_press(&mut self) {
        self.is_dragging = true;
        self.last_mouse = None;
    }

    pub fn on_mouse_release(&mut self) {
        self.is_dragging = false;
        self.last_mouse = None;
    }

    pub fn on_mouse_move(&mut self, x: f64, y: f64) {
        if !self.is_dragging {
            return;
        }
        if let Some((lx, ly)) = self.last_mouse {
            let dx = (x - lx) as f32;
            let dy = (y - ly) as f32;
            self.yaw += dx * 0.005;
            self.pitch = (self.pitch - dy * 0.005).clamp(-1.5, 1.5);
        }
        self.last_mouse = Some((x, y));
    }

    pub fn on_scroll(&mut self, delta: f32) {
        self.distance = (self.distance - delta * self.distance * 0.08).clamp(0.5, 200.0);
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        let x = self.distance * self.pitch.cos() * self.yaw.cos();
        let y = self.distance * self.pitch.cos() * self.yaw.sin();
        let z = self.distance * self.pitch.sin();
        camera.eye = Point3::new(x, y, z);
    }
}

impl Default for CameraController {
    fn default() -> Self {
        Self::new()
    }
}
