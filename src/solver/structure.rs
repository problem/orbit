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
/// All geometry is non-overlapping: walls, slabs, and rooms occupy exclusive volumes.
pub fn generate_building_meshes(building: &SolvedBuilding) -> Vec<BuildingMesh> {
    let mut meshes = Vec::new();
    let fw = building.footprint_width as f32;
    let fd = building.footprint_depth as f32;
    let ext = building.style.exterior_wall_thickness as f32;
    let int = building.style.wall_thickness as f32;
    let slab_t = building.style.floor_thickness as f32;

    // Offset to center the building at the origin
    let offset = Vector3::new(-fw / 2.0, -fd / 2.0, 0.0);

    for floor in &building.floors {
        let z0 = floor.elevation as f32;
        let ch = floor.ceiling_height as f32;

        // --- Floor slab: inset by exterior wall thickness so it doesn't overlap walls ---
        let slab_w = fw - 2.0 * ext;
        let slab_d = fd - 2.0 * ext;
        meshes.push(BuildingMesh {
            mesh: MeshData::box_mesh(slab_w, slab_d, slab_t),
            model_matrix: Matrix4::new_translation(&Vector3::new(
                offset.x + fw / 2.0,
                offset.y + fd / 2.0,
                z0 - slab_t / 2.0,
            )),
            color: building.style.floor_color,
        });

        // --- Room ground planes (colored per room type) ---
        for room in &floor.rooms {
            let w = room.width as f32 - 0.02;
            let d = room.depth as f32 - 0.02;
            meshes.push(BuildingMesh {
                mesh: MeshData::box_mesh(w, d, 0.01),
                model_matrix: Matrix4::new_translation(&Vector3::new(
                    offset.x + room.x as f32 + room.width as f32 / 2.0,
                    offset.y + room.y as f32 + room.depth as f32 / 2.0,
                    z0 + 0.005,
                )),
                color: room_color(room.room_type),
            });
        }

        // --- Exterior walls: 4 panels that meet at corners without overlap ---
        // Each wall is a solid box. South/North walls span the full width.
        // East/West walls fit between the South and North walls (shorter by 2*ext).

        // South wall (full width, at y=ext/2)
        meshes.push(BuildingMesh {
            mesh: MeshData::box_mesh(fw, ext, ch),
            model_matrix: Matrix4::new_translation(&Vector3::new(
                offset.x + fw / 2.0,
                offset.y + ext / 2.0,
                z0 + ch / 2.0,
            )),
            color: building.style.exterior_color,
        });
        // North wall (full width, at y=fd-ext/2)
        meshes.push(BuildingMesh {
            mesh: MeshData::box_mesh(fw, ext, ch),
            model_matrix: Matrix4::new_translation(&Vector3::new(
                offset.x + fw / 2.0,
                offset.y + fd - ext / 2.0,
                z0 + ch / 2.0,
            )),
            color: building.style.exterior_color,
        });
        // West wall (fits between S and N walls)
        meshes.push(BuildingMesh {
            mesh: MeshData::box_mesh(ext, fd - 2.0 * ext, ch),
            model_matrix: Matrix4::new_translation(&Vector3::new(
                offset.x + ext / 2.0,
                offset.y + fd / 2.0,
                z0 + ch / 2.0,
            )),
            color: building.style.exterior_color,
        });
        // East wall (fits between S and N walls)
        meshes.push(BuildingMesh {
            mesh: MeshData::box_mesh(ext, fd - 2.0 * ext, ch),
            model_matrix: Matrix4::new_translation(&Vector3::new(
                offset.x + fw - ext / 2.0,
                offset.y + fd / 2.0,
                z0 + ch / 2.0,
            )),
            color: building.style.exterior_color,
        });

        // --- Interior walls: deduplicated, clipped to interior zone ---
        let interior_walls = collect_interior_walls(floor, building);
        for wall in &interior_walls {
            let dx = (wall.x2 - wall.x1) as f32;
            let dy = (wall.y2 - wall.y1) as f32;
            let length = (dx * dx + dy * dy).sqrt();
            let cx = (wall.x1 as f32 + wall.x2 as f32) / 2.0;
            let cy = (wall.y1 as f32 + wall.y2 as f32) / 2.0;

            let (w, d) = if dy.abs() < 0.001 {
                (length, int) // horizontal
            } else {
                (int, length) // vertical
            };

            meshes.push(BuildingMesh {
                mesh: MeshData::box_mesh(w, d, ch),
                model_matrix: Matrix4::new_translation(&Vector3::new(
                    offset.x + cx,
                    offset.y + cy,
                    z0 + ch / 2.0,
                )),
                color: building.style.interior_wall_color,
            });
        }
    }

    // --- Ground plane ---
    let ground_size = fw.max(fd) * 1.5;
    meshes.push(BuildingMesh {
        mesh: MeshData::box_mesh(ground_size, ground_size, 0.02),
        model_matrix: Matrix4::new_translation(&Vector3::new(0.0, 0.0, -0.02)),
        color: [0.42, 0.50, 0.32],
    });

    // --- Roof ---
    if let Some(ref roof) = building.roof {
        meshes.extend(make_roof(building, roof, offset));
    }

    meshes
}

/// Collect interior wall segments, excluding edges near the footprint boundary.
/// Interior walls are clipped so they don't extend into the exterior wall zone.
fn collect_interior_walls(floor: &SolvedFloor, building: &SolvedBuilding) -> Vec<WallSegment> {
    let fw = building.footprint_width;
    let fd = building.footprint_depth;
    let ext = building.style.exterior_wall_thickness;
    let tol = 0.01;
    let mut segments: Vec<WallSegment> = Vec::new();

    for room in &floor.rooms {
        let edges = [
            (room.x, room.y, room.x + room.width, room.y),
            (room.x, room.y + room.depth, room.x + room.width, room.y + room.depth),
            (room.x, room.y, room.x, room.y + room.depth),
            (room.x + room.width, room.y, room.x + room.width, room.y + room.depth),
        ];

        for (mut ex1, mut ey1, mut ex2, mut ey2) in edges {
            // Skip edges at the footprint boundary (within ext zone)
            let near_left = ex1 < ext + tol && ex2 < ext + tol;
            let near_right = (fw - ex1) < ext + tol && (fw - ex2) < ext + tol;
            let near_bottom = ey1 < ext + tol && ey2 < ext + tol;
            let near_top = (fd - ey1) < ext + tol && (fd - ey2) < ext + tol;
            if near_left || near_right || near_bottom || near_top {
                continue;
            }

            // Clip endpoints to stay within interior zone (inside exterior walls)
            ex1 = ex1.clamp(ext, fw - ext);
            ey1 = ey1.clamp(ext, fd - ext);
            ex2 = ex2.clamp(ext, fw - ext);
            ey2 = ey2.clamp(ext, fd - ext);

            // Skip zero-length after clipping
            let dx = (ex2 - ex1).abs();
            let dy = (ey2 - ey1).abs();
            if dx < tol && dy < tol {
                continue;
            }

            // Deduplicate
            let is_dup = segments.iter().any(|s| {
                segments_overlap(s.x1, s.y1, s.x2, s.y2, ex1, ey1, ex2, ey2, tol)
            });

            if !is_dup {
                segments.push(WallSegment {
                    x1: ex1,
                    y1: ey1,
                    x2: ex2,
                    y2: ey2,
                });
            }
        }
    }

    segments
}

#[derive(Debug, Clone)]
struct WallSegment {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

fn segments_overlap(
    ax1: f64, ay1: f64, ax2: f64, ay2: f64,
    bx1: f64, by1: f64, bx2: f64, by2: f64,
    tol: f64,
) -> bool {
    if (ay1 - ay2).abs() < tol && (by1 - by2).abs() < tol && (ay1 - by1).abs() < tol {
        let (a_min, a_max) = (ax1.min(ax2), ax1.max(ax2));
        let (b_min, b_max) = (bx1.min(bx2), bx1.max(bx2));
        return a_min < b_max - tol && b_min < a_max - tol;
    }
    if (ax1 - ax2).abs() < tol && (bx1 - bx2).abs() < tol && (ax1 - bx1).abs() < tol {
        let (a_min, a_max) = (ay1.min(ay2), ay1.max(ay2));
        let (b_min, b_max) = (by1.min(by2), by1.max(by2));
        return a_min < b_max - tol && b_min < a_max - tol;
    }
    false
}

// --- Roof ---

fn make_roof(
    building: &SolvedBuilding,
    roof: &SolvedRoof,
    offset: Vector3<f32>,
) -> Vec<BuildingMesh> {
    let mut meshes = Vec::new();
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
                let half_span = fd / 2.0;
                let ridge_height = pitch * half_span;
                meshes.extend(make_gable_roof_planes(
                    fw, fd, ridge_height, base_z, overhang, true, offset,
                    building.style.roof_color,
                ));
                // Gable end walls (east and west)
                meshes.push(make_gable_wall(
                    0.0, fd / 2.0, base_z, half_span, ridge_height, true,
                    true, offset, building.style.exterior_color,
                ));
                meshes.push(make_gable_wall(
                    fw, fd / 2.0, base_z, half_span, ridge_height, true,
                    false, offset, building.style.exterior_color,
                ));
            } else {
                let half_span = fw / 2.0;
                let ridge_height = pitch * half_span;
                meshes.extend(make_gable_roof_planes(
                    fw, fd, ridge_height, base_z, overhang, false, offset,
                    building.style.roof_color,
                ));
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

fn make_gable_roof_planes(
    fw: f32, fd: f32, ridge_height: f32, base_z: f32,
    overhang: f32, ridge_along_x: bool, offset: Vector3<f32>,
    color: [f32; 3],
) -> Vec<BuildingMesh> {
    let mut meshes = Vec::new();
    if ridge_along_x {
        let half_d = fd / 2.0;
        let slope_len = (half_d * half_d + ridge_height * ridge_height).sqrt();
        for side in [-1.0f32, 1.0] {
            let mesh = MeshData::box_mesh(fw + 2.0 * overhang, slope_len + overhang, 0.05);
            let mid_y = fd / 2.0 + side * half_d / 2.0;
            let mid_z = base_z + ridge_height / 2.0;
            let angle = (ridge_height / half_d).atan() * side;
            let rot = nalgebra::UnitQuaternion::from_axis_angle(
                &nalgebra::Vector3::x_axis(), angle,
            );
            let model = Matrix4::new_translation(&Vector3::new(
                offset.x + fw / 2.0, offset.y + mid_y, mid_z,
            )) * rot.to_homogeneous();
            meshes.push(BuildingMesh { mesh, model_matrix: model, color });
        }
    } else {
        let half_w = fw / 2.0;
        let slope_len = (half_w * half_w + ridge_height * ridge_height).sqrt();
        for side in [-1.0f32, 1.0] {
            let mesh = MeshData::box_mesh(slope_len + overhang, fd + 2.0 * overhang, 0.05);
            let mid_x = fw / 2.0 + side * half_w / 2.0;
            let mid_z = base_z + ridge_height / 2.0;
            let angle = -(ridge_height / half_w).atan() * side;
            let rot = nalgebra::UnitQuaternion::from_axis_angle(
                &nalgebra::Vector3::y_axis(), angle,
            );
            let model = Matrix4::new_translation(&Vector3::new(
                offset.x + mid_x, offset.y + fd / 2.0, mid_z,
            )) * rot.to_homogeneous();
            meshes.push(BuildingMesh { mesh, model_matrix: model, color });
        }
    }
    meshes
}

fn make_gable_wall(
    x: f32, y: f32, base_z: f32, half_span: f32, ridge_height: f32,
    ridge_along_x: bool, is_near_side: bool,
    offset: Vector3<f32>, color: [f32; 3],
) -> BuildingMesh {
    let (positions, normals) = if ridge_along_x {
        let normal_dir = if is_near_side { -1.0f32 } else { 1.0 };
        let bl = [offset.x + x, offset.y + y - half_span, base_z];
        let br = [offset.x + x, offset.y + y + half_span, base_z];
        let apex = [offset.x + x, offset.y + y, base_z + ridge_height];
        let n = [normal_dir, 0.0, 0.0];
        (
            vec![bl, br, apex, apex, br, bl],
            vec![n, n, n, [-n[0], 0.0, 0.0], [-n[0], 0.0, 0.0], [-n[0], 0.0, 0.0]],
        )
    } else {
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
    BuildingMesh {
        mesh: MeshData { positions, normals, indices: vec![0, 1, 2, 3, 4, 5], edges: None },
        model_matrix: Matrix4::identity(),
        color,
    }
}
