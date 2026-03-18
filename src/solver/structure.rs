use nalgebra::{Matrix4, Vector3};

use crate::orb::mesh::MeshData;
use super::types::*;

pub struct BuildingMesh {
    pub mesh: MeshData,
    pub model_matrix: Matrix4<f32>,
    pub color: [f32; 3],
}

/// Generate building geometry. All coordinates in meters, centered at origin.
pub fn generate_building_meshes(building: &SolvedBuilding) -> Vec<BuildingMesh> {
    let mut meshes = Vec::new();
    let fw = building.footprint_width as f32;
    let fd = building.footprint_depth as f32;
    let ext = building.style.exterior_wall_thickness as f32;
    let slab_t = building.style.floor_thickness as f32;

    // Building is centered at origin. All positions relative to SW corner then offset.
    let ox = -fw / 2.0;
    let oy = -fd / 2.0;

    for floor in &building.floors {
        let z = floor.elevation as f32;
        let h = floor.ceiling_height as f32;

        // Foundation/floor slab — full footprint, sits below floor level
        meshes.push(make_box(
            fw, fd, slab_t,
            ox + fw / 2.0, oy + fd / 2.0, z - slab_t / 2.0,
            building.style.floor_color,
        ));

        // 4 exterior walls as solid boxes, meeting at corners:
        // South and North span full X width.
        // East and West fit between them (height of fd - 2*ext).

        // South wall
        meshes.push(make_box(
            fw, ext, h,
            ox + fw / 2.0, oy + ext / 2.0, z + h / 2.0,
            building.style.exterior_color,
        ));
        // North wall
        meshes.push(make_box(
            fw, ext, h,
            ox + fw / 2.0, oy + fd - ext / 2.0, z + h / 2.0,
            building.style.exterior_color,
        ));
        // West wall (between S and N)
        meshes.push(make_box(
            ext, fd - 2.0 * ext, h,
            ox + ext / 2.0, oy + fd / 2.0, z + h / 2.0,
            building.style.exterior_color,
        ));
        // East wall (between S and N)
        meshes.push(make_box(
            ext, fd - 2.0 * ext, h,
            ox + fw - ext / 2.0, oy + fd / 2.0, z + h / 2.0,
            building.style.exterior_color,
        ));

        // Ceiling — full footprint, sits at top of walls
        meshes.push(make_box(
            fw, fd, slab_t,
            ox + fw / 2.0, oy + fd / 2.0, z + h + slab_t / 2.0,
            [0.92, 0.90, 0.85],
        ));
    }

    // Roof
    if let Some(ref roof) = building.roof {
        let top = building.floors.last().unwrap();
        let base_z = top.elevation as f32 + top.ceiling_height as f32 + slab_t;
        meshes.extend(make_gable_roof(
            fw, fd, base_z,
            roof.pitch_ratio as f32,
            building.style.roof_overhang as f32,
            roof.ridge_along_x,
            ox, oy,
            building.style.roof_color,
            building.style.exterior_color,
        ));
    }

    // Ground plane
    let gs = fw.max(fd) * 2.0;
    meshes.push(make_box(
        gs, gs, 0.05,
        0.0, 0.0, -slab_t - 0.05,
        [0.42, 0.50, 0.32],
    ));

    meshes
}

/// Create a box with w=X-width, d=Y-depth, h=Z-height centered at (cx,cy,cz).
fn make_box(w: f32, d: f32, h: f32, cx: f32, cy: f32, cz: f32, color: [f32; 3]) -> BuildingMesh {
    // box_mesh params: (width=X, height=Z, depth=Y)
    BuildingMesh {
        mesh: MeshData::box_mesh(w, h, d),
        model_matrix: Matrix4::new_translation(&Vector3::new(cx, cy, cz)),
        color,
    }
}

/// Gable roof: two sloped planes + two triangular gable end walls + two side infill triangles.
fn make_gable_roof(
    fw: f32, fd: f32, base_z: f32, pitch: f32, overhang: f32,
    ridge_along_x: bool, ox: f32, oy: f32,
    roof_color: [f32; 3], wall_color: [f32; 3],
) -> Vec<BuildingMesh> {
    let mut out = Vec::new();

    let (span, length) = if ridge_along_x { (fd, fw) } else { (fw, fd) };
    let half_span = span / 2.0;
    let ridge_h = pitch * half_span;
    let slope_len = (half_span * half_span + ridge_h * ridge_h).sqrt();

    // Ridge center position
    let ridge_cx = ox + fw / 2.0;
    let ridge_cy = oy + fd / 2.0;

    // Two sloped roof planes
    for side in [-1.0f32, 1.0] {
        let mesh = MeshData::box_mesh(
            if ridge_along_x { length + 2.0 * overhang } else { slope_len + overhang },
            if ridge_along_x { slope_len + overhang } else { length + 2.0 * overhang },
            0.05,
        );

        let slope_angle = (ridge_h / half_span).atan();

        let model = if ridge_along_x {
            // Slopes face N/S, rotate around X
            let mid_y = ridge_cy + side * half_span / 2.0;
            let mid_z = base_z + ridge_h / 2.0;
            let rot = nalgebra::UnitQuaternion::from_axis_angle(
                &nalgebra::Vector3::x_axis(), slope_angle * side,
            );
            Matrix4::new_translation(&Vector3::new(ridge_cx, mid_y, mid_z))
                * rot.to_homogeneous()
        } else {
            // Slopes face E/W, rotate around Y
            let mid_x = ridge_cx + side * half_span / 2.0;
            let mid_z = base_z + ridge_h / 2.0;
            let rot = nalgebra::UnitQuaternion::from_axis_angle(
                &nalgebra::Vector3::y_axis(), -slope_angle * side,
            );
            Matrix4::new_translation(&Vector3::new(mid_x, ridge_cy, mid_z))
                * rot.to_homogeneous()
        };

        out.push(BuildingMesh { mesh, model_matrix: model, color: roof_color });
    }

    // Gable end walls (triangles at the two ends perpendicular to the ridge)
    // + Side infill walls (triangles on the two sides parallel to the ridge)
    if ridge_along_x {
        // Gable ends: east (x=fw) and west (x=0) — triangles in YZ plane
        for &x in &[ox, ox + fw] {
            let nx = if x < ridge_cx { -1.0 } else { 1.0 };
            out.push(make_triangle(
                [x, oy, base_z],
                [x, oy + fd, base_z],
                [x, ridge_cy, base_z + ridge_h],
                [nx, 0.0, 0.0],
                wall_color,
            ));
        }
        // Side infill: south (y=0) and north (y=fd) — triangles in XZ plane
        for &y in &[oy, oy + fd] {
            let ny = if y < ridge_cy { -1.0 } else { 1.0 };
            out.push(make_triangle(
                [ox, y, base_z],
                [ox + fw, y, base_z],
                [ridge_cx, y, base_z + ridge_h],
                [0.0, ny, 0.0],
                wall_color,
            ));
        }
    } else {
        // Gable ends: south and north
        for &y in &[oy, oy + fd] {
            let ny = if y < ridge_cy { -1.0 } else { 1.0 };
            out.push(make_triangle(
                [ox, y, base_z],
                [ox + fw, y, base_z],
                [ridge_cx, y, base_z + ridge_h],
                [0.0, ny, 0.0],
                wall_color,
            ));
        }
        // Side infill: east and west
        for &x in &[ox, ox + fw] {
            let nx = if x < ridge_cx { -1.0 } else { 1.0 };
            out.push(make_triangle(
                [x, oy, base_z],
                [x, oy + fd, base_z],
                [x, ridge_cy, base_z + ridge_h],
                [nx, 0.0, 0.0],
                wall_color,
            ));
        }
    }

    out
}

/// Double-sided triangle mesh.
fn make_triangle(
    a: [f32; 3], b: [f32; 3], c: [f32; 3],
    normal: [f32; 3], color: [f32; 3],
) -> BuildingMesh {
    let back = [-normal[0], -normal[1], -normal[2]];
    BuildingMesh {
        mesh: MeshData {
            positions: vec![a, b, c, c, b, a],
            normals: vec![normal, normal, normal, back, back, back],
            indices: vec![0, 1, 2, 3, 4, 5],
            edges: None,
        },
        model_matrix: Matrix4::identity(),
        color,
    }
}
