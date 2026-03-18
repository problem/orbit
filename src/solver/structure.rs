use nalgebra::{Matrix4, Vector3};

use crate::orb::mesh::MeshData;
use super::types::*;

/// A mesh with its position and color, ready for the renderer.
pub struct BuildingMesh {
    pub mesh: MeshData,
    pub model_matrix: Matrix4<f32>,
    pub color: [f32; 3],
}

/// Generate all 3D geometry from a solved building.
pub fn generate_building_meshes(building: &SolvedBuilding) -> Vec<BuildingMesh> {
    let mut meshes = Vec::new();
    let half_w = (building.footprint_width / 2.0) as f32;
    let half_d = (building.footprint_depth / 2.0) as f32;

    // Offset to center the building at the origin
    let offset = Vector3::new(-half_w, -half_d, 0.0);

    for floor in &building.floors {
        // Floor slab
        meshes.push(make_floor_slab(building, floor, offset));

        // Room ground planes (colored per room type)
        for room in &floor.rooms {
            meshes.push(make_room_plane(room, floor, offset));
        }

        // Walls
        let walls = collect_wall_segments(floor, building);
        for wall in &walls {
            meshes.push(make_wall(wall, floor, &building.style, offset));
        }
    }

    meshes
}

fn make_floor_slab(building: &SolvedBuilding, floor: &SolvedFloor, offset: Vector3<f32>) -> BuildingMesh {
    let w = building.footprint_width as f32;
    let d = building.footprint_depth as f32;
    let thickness = building.style.floor_thickness as f32;
    let z = floor.elevation as f32;

    let mesh = MeshData::box_mesh(w, d, thickness);
    let model = Matrix4::new_translation(&Vector3::new(
        offset.x + w / 2.0,
        offset.y + d / 2.0,
        z - thickness / 2.0,
    ));

    BuildingMesh {
        mesh,
        model_matrix: model,
        color: building.style.floor_color,
    }
}

fn make_room_plane(room: &SolvedRoom, floor: &SolvedFloor, offset: Vector3<f32>) -> BuildingMesh {
    let w = room.width as f32 - 0.01; // slight inset to see wall boundaries
    let d = room.depth as f32 - 0.01;
    let z = floor.elevation as f32 + 0.005; // just above the floor slab

    let mesh = MeshData::box_mesh(w, d, 0.01);
    let cx = room.x as f32 + room.width as f32 / 2.0;
    let cy = room.y as f32 + room.depth as f32 / 2.0;
    let model = Matrix4::new_translation(&Vector3::new(
        offset.x + cx,
        offset.y + cy,
        z,
    ));

    BuildingMesh {
        mesh,
        model_matrix: model,
        color: room_color(room.room_type),
    }
}

/// A wall segment definition.
#[derive(Debug, Clone)]
struct WallSegment {
    /// Start point (x, y) in meters from footprint origin.
    x1: f64,
    y1: f64,
    /// End point (x, y).
    x2: f64,
    y2: f64,
    is_exterior: bool,
}

/// Collect all wall segments for a floor, deduplicating shared walls.
fn collect_wall_segments(floor: &SolvedFloor, building: &SolvedBuilding) -> Vec<WallSegment> {
    let mut segments: Vec<WallSegment> = Vec::new();
    let _ext = building.style.exterior_wall_thickness;

    // Exterior walls (footprint boundary)
    let fw = building.footprint_width;
    let fd = building.footprint_depth;
    segments.push(WallSegment { x1: 0.0, y1: 0.0, x2: fw, y2: 0.0, is_exterior: true }); // south
    segments.push(WallSegment { x1: fw, y1: 0.0, x2: fw, y2: fd, is_exterior: true }); // east
    segments.push(WallSegment { x1: fw, y1: fd, x2: 0.0, y2: fd, is_exterior: true }); // north
    segments.push(WallSegment { x1: 0.0, y1: fd, x2: 0.0, y2: 0.0, is_exterior: true }); // west

    // Interior walls from room boundaries
    // For each room edge that doesn't coincide with the footprint boundary,
    // add an interior wall segment (deduplicated).
    let tol = 0.01;
    for room in &floor.rooms {
        let edges = [
            // bottom
            (room.x, room.y, room.x + room.width, room.y),
            // top
            (room.x, room.y + room.depth, room.x + room.width, room.y + room.depth),
            // left
            (room.x, room.y, room.x, room.y + room.depth),
            // right
            (room.x + room.width, room.y, room.x + room.width, room.y + room.depth),
        ];

        for (ex1, ey1, ex2, ey2) in edges {
            let on_boundary = (ex1.abs() < tol && ex2.abs() < tol)         // left boundary (before inset)
                || ((ex1 - fw).abs() < tol && (ex2 - fw).abs() < tol)       // right
                || (ey1.abs() < tol && ey2.abs() < tol)                     // bottom
                || ((ey1 - fd).abs() < tol && (ey2 - fd).abs() < tol);      // top

            if on_boundary {
                continue; // exterior walls already added
            }

            // Check if this segment is already in the list (shared wall between two rooms)
            let is_dup = segments.iter().any(|s| {
                !s.is_exterior && segments_overlap(s.x1, s.y1, s.x2, s.y2, ex1, ey1, ex2, ey2, tol)
            });

            if !is_dup {
                segments.push(WallSegment {
                    x1: ex1,
                    y1: ey1,
                    x2: ex2,
                    y2: ey2,
                    is_exterior: false,
                });
            }
        }
    }

    segments
}

/// Check if two line segments overlap (same axis, overlapping range).
fn segments_overlap(
    ax1: f64, ay1: f64, ax2: f64, ay2: f64,
    bx1: f64, by1: f64, bx2: f64, by2: f64,
    tol: f64,
) -> bool {
    // Both horizontal (same y)?
    if (ay1 - ay2).abs() < tol && (by1 - by2).abs() < tol && (ay1 - by1).abs() < tol {
        let a_min = ax1.min(ax2);
        let a_max = ax1.max(ax2);
        let b_min = bx1.min(bx2);
        let b_max = bx1.max(bx2);
        return a_min < b_max - tol && b_min < a_max - tol;
    }
    // Both vertical (same x)?
    if (ax1 - ax2).abs() < tol && (bx1 - bx2).abs() < tol && (ax1 - bx1).abs() < tol {
        let a_min = ay1.min(ay2);
        let a_max = ay1.max(ay2);
        let b_min = by1.min(by2);
        let b_max = by1.max(by2);
        return a_min < b_max - tol && b_min < a_max - tol;
    }
    false
}

fn make_wall(
    seg: &WallSegment,
    floor: &SolvedFloor,
    style: &ResolvedStyle,
    offset: Vector3<f32>,
) -> BuildingMesh {
    let thickness = if seg.is_exterior {
        style.exterior_wall_thickness as f32
    } else {
        style.wall_thickness as f32
    };
    let height = floor.ceiling_height as f32;
    let z = floor.elevation as f32 + height / 2.0;

    let dx = (seg.x2 - seg.x1) as f32;
    let dy = (seg.y2 - seg.y1) as f32;
    let length = (dx * dx + dy * dy).sqrt();

    let cx = (seg.x1 as f32 + seg.x2 as f32) / 2.0;
    let cy = (seg.y1 as f32 + seg.y2 as f32) / 2.0;

    // Walls are axis-aligned, so one dimension is length, the other is thickness
    let (w, d) = if dy.abs() < 0.001 {
        // Horizontal wall (along X)
        (length, thickness)
    } else {
        // Vertical wall (along Y)
        (thickness, length)
    };

    let mesh = MeshData::box_mesh(w, d, height);
    let model = Matrix4::new_translation(&Vector3::new(
        offset.x + cx,
        offset.y + cy,
        z,
    ));

    let color = if seg.is_exterior {
        style.exterior_color
    } else {
        style.interior_wall_color
    };

    BuildingMesh {
        mesh,
        model_matrix: model,
        color,
    }
}
