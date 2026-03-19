use crate::oil::ast::WindowSpec;
use crate::oil::types::{Cardinal, Feature, RoofForm, RoomType};

/// Style defaults resolved from the style block.
#[derive(Debug, Clone)]
pub struct ResolvedStyle {
    pub name: String,
    pub ceiling_height: f64,
    pub wall_thickness: f64,
    pub exterior_wall_thickness: f64,
    pub floor_thickness: f64,
    pub exterior_color: [f32; 3],
    pub interior_wall_color: [f32; 3],
    pub floor_color: [f32; 3],
    pub roof_color: [f32; 3],
    pub roof_overhang: f64,
}

/// A resolved room with concrete numeric targets (all in meters).
#[derive(Debug, Clone)]
pub struct ResolvedRoom {
    pub name: String,
    pub room_type: RoomType,
    pub target_area: f64,
    pub area_tolerance: f64,
    pub aspect_target: f64,
    pub side_pin: Option<Cardinal>,
    pub windows: Vec<WindowSpec>,
    pub features: Vec<Feature>,
    pub adjacent_to: Vec<String>,
    pub connects: Vec<String>,
}

/// A solved room with final 2D placement (meters, origin at footprint SW corner).
#[derive(Debug, Clone)]
pub struct SolvedRoom {
    pub name: String,
    pub room_type: RoomType,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub depth: f64,
    pub windows: Vec<WindowSpec>,
    pub features: Vec<Feature>,
}

/// A complete solved floor.
#[derive(Debug, Clone)]
pub struct SolvedFloor {
    pub name: String,
    pub floor_index: usize,
    pub elevation: f64,
    pub ceiling_height: f64,
    pub rooms: Vec<SolvedRoom>,
}

/// Resolved roof specification.
#[derive(Debug, Clone)]
pub struct SolvedRoof {
    pub form: RoofForm,
    /// Ridge runs along this axis: "east-west" means ridge along X, "north-south" along Y.
    pub ridge_along_x: bool,
    /// Pitch as rise/run ratio (e.g. 12:12 = 1.0, 10:12 = 0.833).
    pub pitch_ratio: f64,
}

/// The solver's final output.
pub struct SolvedBuilding {
    pub floors: Vec<SolvedFloor>,
    pub roof: Option<SolvedRoof>,
    pub style: ResolvedStyle,
    pub footprint_width: f64,
    pub footprint_depth: f64,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
}

#[derive(Debug)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
}

/// Room type default area (sqm) and aspect ratio.
pub fn room_defaults(room_type: RoomType) -> (f64, f64) {
    match room_type {
        RoomType::Living => (22.0, 1.4),
        RoomType::Kitchen => (14.0, 1.2),
        RoomType::Dining => (12.0, 1.2),
        RoomType::MasterBed => (18.0, 1.3),
        RoomType::Bedroom => (12.0, 1.2),
        RoomType::Bathroom => (6.0, 1.0),
        RoomType::MasterBath => (8.0, 1.0),
        RoomType::HalfBath => (3.5, 0.8),
        RoomType::Entry => (5.0, 1.0),
        RoomType::Hallway => (5.0, 3.0),
        RoomType::Garage => (35.0, 1.5),
        RoomType::Office => (10.0, 1.2),
        RoomType::Laundry => (6.0, 1.0),
        RoomType::Mudroom => (4.0, 1.0),
        RoomType::Closet => (3.0, 0.8),
        RoomType::Pantry => (3.0, 0.8),
        RoomType::Family => (20.0, 1.3),
        RoomType::Porch => (8.0, 2.0),
        _ => (10.0, 1.0),
    }
}

/// Room type color for visualization.
pub fn room_color(room_type: RoomType) -> [f32; 3] {
    match room_type {
        RoomType::Living | RoomType::Family => [0.92, 0.87, 0.78],   // warm beige
        RoomType::Kitchen => [0.85, 0.88, 0.82],                      // sage
        RoomType::Dining => [0.90, 0.85, 0.75],                       // light tan
        RoomType::MasterBed | RoomType::Bedroom => [0.82, 0.86, 0.92], // light blue
        RoomType::Bathroom | RoomType::MasterBath => [0.80, 0.90, 0.88], // light teal
        RoomType::HalfBath => [0.83, 0.88, 0.87],                     // lighter teal
        RoomType::Entry | RoomType::Hallway => [0.88, 0.88, 0.85],    // light gray-beige
        RoomType::Garage => [0.78, 0.78, 0.78],                       // concrete gray
        RoomType::Office => [0.85, 0.82, 0.78],                       // warm gray
        RoomType::Laundry => [0.84, 0.84, 0.88],                      // cool gray
        RoomType::Closet | RoomType::Pantry => [0.86, 0.84, 0.80],    // tan
        _ => [0.85, 0.85, 0.85],                                       // neutral
    }
}
