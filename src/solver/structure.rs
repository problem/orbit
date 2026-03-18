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

    // Roof
    if let Some(ref roof) = building.roof {
        let roof_meshes = make_roof(building, roof, offset);
        meshes.extend(roof_meshes);
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

    // Exterior walls, inset by half their thickness so they sit within the slab
    let fw = building.footprint_width;
    let fd = building.footprint_depth;
    let hw = _ext / 2.0;
    segments.push(WallSegment { x1: 0.0, y1: hw, x2: fw, y2: hw, is_exterior: true });       // south
    segments.push(WallSegment { x1: fw - hw, y1: 0.0, x2: fw - hw, y2: fd, is_exterior: true }); // east
    segments.push(WallSegment { x1: fw, y1: fd - hw, x2: 0.0, y2: fd - hw, is_exterior: true }); // north
    segments.push(WallSegment { x1: hw, y1: fd, x2: hw, y2: 0.0, is_exterior: true });       // west

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
            // Skip edges near the footprint boundary — those are the interior
            // faces of exterior walls and don't need separate interior walls.
            let ext_t = building.style.exterior_wall_thickness + tol;
            let on_boundary = ex1 < ext_t && ex2 < ext_t                          // near left
                || (fw - ex1) < ext_t && (fw - ex2) < ext_t                       // near right
                || ey1 < ext_t && ey2 < ext_t                                     // near bottom
                || (fd - ey1) < ext_t && (fd - ey2) < ext_t;                      // near top

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

/// Generate roof meshes. For a gable roof: two sloped rectangular planes + two triangular gable ends.
fn make_roof(
    building: &SolvedBuilding,
    roof: &SolvedRoof,
    offset: Vector3<f32>,
) -> Vec<BuildingMesh> {
    let mut meshes = Vec::new();

    // Roof sits on top of the highest floor
    let top_floor = match building.floors.last() {
        Some(f) => f,
        None => return meshes,
    };
    let base_z = (top_floor.elevation + top_floor.ceiling_height) as f32;

    let fw = building.footprint_width as f32;
    let fd = building.footprint_depth as f32;
    let overhang = building.style.roof_overhang as f32;
    let pitch = roof.pitch_ratio as f32;

    match roof.form {
        crate::oil::types::RoofForm::Gable => {
            if roof.ridge_along_x {
                // Ridge runs along X (east-west). Slopes face north and south.
                // Ridge height = pitch * (depth/2)
                let half_span = fd / 2.0;
                let ridge_height = pitch * half_span;

                // Two sloped roof planes
                let roof_planes = make_gable_roof_planes(
                    fw, fd, ridge_height, base_z, overhang, true, offset,
                    building.style.roof_color,
                );
                meshes.extend(roof_planes);

                // Two triangular gable walls (east and west ends)
                meshes.push(make_gable_wall(
                    0.0, fd / 2.0, base_z, half_span, ridge_height, true,
                    true, offset, building.style.exterior_color,
                ));
                meshes.push(make_gable_wall(
                    fw, fd / 2.0, base_z, half_span, ridge_height, true,
                    false, offset, building.style.exterior_color,
                ));
            } else {
                // Ridge runs along Y (north-south). Slopes face east and west.
                let half_span = fw / 2.0;
                let ridge_height = pitch * half_span;

                let roof_planes = make_gable_roof_planes(
                    fw, fd, ridge_height, base_z, overhang, false, offset,
                    building.style.roof_color,
                );
                meshes.extend(roof_planes);

                // Gable walls (north and south ends)
                meshes.push(make_gable_wall(
                    fw / 2.0, 0.0, base_z, half_span, ridge_height, false,
                    true, offset, building.style.exterior_color,
                ));
                meshes.push(make_gable_wall(
                    fw / 2.0, fd, base_z, half_span, ridge_height, false,
                    false, offset, building.style.exterior_color,
                ));
            }
        }
        _ => {
            // Other roof forms: generate a flat "cap" as placeholder
            let cap = MeshData::box_mesh(fw + overhang * 2.0, fd + overhang * 2.0, 0.1);
            meshes.push(BuildingMesh {
                mesh: cap,
                model_matrix: Matrix4::new_translation(&Vector3::new(
                    offset.x + fw / 2.0,
                    offset.y + fd / 2.0,
                    base_z + 0.05,
                )),
                color: building.style.roof_color,
            });
        }
    }

    meshes
}

/// Generate two sloped rectangular planes for a gable roof.
fn make_gable_roof_planes(
    fw: f32, fd: f32, ridge_height: f32, base_z: f32,
    overhang: f32, ridge_along_x: bool, offset: Vector3<f32>,
    color: [f32; 3],
) -> Vec<BuildingMesh> {
    let mut meshes = Vec::new();

    if ridge_along_x {
        // Ridge along X. Two planes slope from ridge down to eaves (north and south).
        let half_d = fd / 2.0;
        let slope_len = (half_d * half_d + ridge_height * ridge_height).sqrt();
        let ovh = overhang;

        // South slope: from ridge (y=fd/2, z=base+ridge_h) down to eave (y=0, z=base)
        // North slope: mirror
        for side in [-1.0f32, 1.0] {
            let mesh = make_sloped_quad(
                fw + 2.0 * ovh, slope_len + ovh,
            );
            // Position: centered on footprint X, at midpoint of slope Y, at midpoint of slope Z
            let mid_y = fd / 2.0 + side * half_d / 2.0;
            let mid_z = base_z + ridge_height / 2.0;
            // Rotation: tilt around X axis
            let angle = (ridge_height / half_d).atan() * side;
            let rot = nalgebra::UnitQuaternion::from_axis_angle(
                &nalgebra::Vector3::x_axis(), angle,
            );
            let model = Matrix4::new_translation(&Vector3::new(
                offset.x + fw / 2.0,
                offset.y + mid_y,
                mid_z,
            )) * rot.to_homogeneous();

            meshes.push(BuildingMesh {
                mesh,
                model_matrix: model,
                color,
            });
        }
    } else {
        // Ridge along Y. Two planes slope east and west.
        let half_w = fw / 2.0;
        let slope_len = (half_w * half_w + ridge_height * ridge_height).sqrt();
        let ovh = overhang;

        for side in [-1.0f32, 1.0] {
            let mesh = make_sloped_quad(
                slope_len + ovh, fd + 2.0 * ovh,
            );
            let mid_x = fw / 2.0 + side * half_w / 2.0;
            let mid_z = base_z + ridge_height / 2.0;
            let angle = -(ridge_height / half_w).atan() * side;
            let rot = nalgebra::UnitQuaternion::from_axis_angle(
                &nalgebra::Vector3::y_axis(), angle,
            );
            let model = Matrix4::new_translation(&Vector3::new(
                offset.x + mid_x,
                offset.y + fd / 2.0,
                mid_z,
            )) * rot.to_homogeneous();

            meshes.push(BuildingMesh {
                mesh,
                model_matrix: model,
                color,
            });
        }
    }

    meshes
}

/// Generate a flat quad (thin box) for a roof plane.
fn make_sloped_quad(width: f32, depth: f32) -> MeshData {
    MeshData::box_mesh(width, depth, 0.05)
}

/// Generate a triangular gable wall to fill the triangle above the rectangular wall.
fn make_gable_wall(
    x: f32, y: f32, base_z: f32, half_span: f32, ridge_height: f32,
    ridge_along_x: bool, is_near_side: bool,
    offset: Vector3<f32>, color: [f32; 3],
) -> BuildingMesh {
    // Triangle vertices: base-left, base-right, apex
    let (positions, normals) = if ridge_along_x {
        // Gable on east or west face (YZ plane)
        let normal_dir = if is_near_side { -1.0f32 } else { 1.0 };
        let bl = [offset.x + x, offset.y + y - half_span, base_z];
        let br = [offset.x + x, offset.y + y + half_span, base_z];
        let apex = [offset.x + x, offset.y + y, base_z + ridge_height];
        let n = [normal_dir, 0.0, 0.0];
        // Two triangles (front and back face)
        (
            vec![bl, br, apex, apex, br, bl],
            vec![n, n, n, [-n[0], 0.0, 0.0], [-n[0], 0.0, 0.0], [-n[0], 0.0, 0.0]],
        )
    } else {
        // Gable on north or south face (XZ plane)
        let normal_dir = if is_near_side { -1.0f32 } else { 1.0 };
        let bl = [offset.x + x - half_span, offset.y + y, base_z];
        let br = [offset.x + x + half_span, offset.y + y, base_z];
        let apex = [offset.x + x, offset.y + y, base_z + ridge_height];
        let n = [0.0, normal_dir, 0.0];
        (
            vec![bl, br, apex, apex, br, bl],
            vec![n, n, n, [0.0, -n[1], 0.0], [0.0, -n[1], 0.0], [0.0, -n[1], 0.0]],
        )
    };

    let indices = vec![0, 1, 2, 3, 4, 5];

    BuildingMesh {
        mesh: MeshData {
            positions,
            normals,
            indices,
            edges: None,
        },
        model_matrix: Matrix4::identity(),
        color,
    }
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
