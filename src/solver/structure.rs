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
/// Generate black edge outline geometry for all building boxes.
pub fn generate_edge_meshes(building: &SolvedBuilding) -> Vec<BuildingMesh> {
    let mut edges = Vec::new();
    let fw = building.footprint_width as f32;
    let fd = building.footprint_depth as f32;
    let ext = building.style.exterior_wall_thickness as f32;
    let slab_t = building.style.floor_thickness as f32;
    let ox = -fw / 2.0;
    let oy = -fd / 2.0;
    let t = 0.03; // edge strip thickness in meters
    let black = [0.0, 0.0, 0.0];

    for floor in &building.floors {
        let z = floor.elevation as f32;
        let h = floor.ceiling_height as f32;

        // Foundation slab edges
        edges.extend(box_edges(fw, fd, slab_t, ox + fw/2.0, oy + fd/2.0, z - slab_t/2.0, t, black));
        // South wall
        edges.extend(box_edges(fw, ext, h, ox + fw/2.0, oy + ext/2.0, z + h/2.0, t, black));
        // North wall
        edges.extend(box_edges(fw, ext, h, ox + fw/2.0, oy + fd - ext/2.0, z + h/2.0, t, black));
        // West wall
        edges.extend(box_edges(ext, fd - 2.0*ext, h, ox + ext/2.0, oy + fd/2.0, z + h/2.0, t, black));
        // East wall
        edges.extend(box_edges(ext, fd - 2.0*ext, h, ox + fw - ext/2.0, oy + fd/2.0, z + h/2.0, t, black));
        // Ceiling
        edges.extend(box_edges(fw, fd, slab_t, ox + fw/2.0, oy + fd/2.0, z + h + slab_t/2.0, t, black));
    }

    edges
}

/// Generate 12 edge strips for a box (w=X, d=Y, h=Z) centered at (cx,cy,cz).
fn box_edges(w: f32, d: f32, h: f32, cx: f32, cy: f32, cz: f32, t: f32, color: [f32; 3]) -> Vec<BuildingMesh> {
    let hw = w / 2.0;
    let hd = d / 2.0;
    let hh = h / 2.0;
    let mut edges = Vec::new();

    // 4 edges along X
    for &dy in &[-hd, hd] {
        for &dz in &[-hh, hh] {
            edges.push(make_box(w + t, t, t, cx, cy + dy, cz + dz, color));
        }
    }
    // 4 edges along Y
    for &dx in &[-hw, hw] {
        for &dz in &[-hh, hh] {
            edges.push(make_box(t, d + t, t, cx + dx, cy, cz + dz, color));
        }
    }
    // 4 edges along Z
    for &dx in &[-hw, hw] {
        for &dy in &[-hd, hd] {
            edges.push(make_box(t, t, h + t, cx + dx, cy + dy, cz, color));
        }
    }
    edges
}

fn make_box(w: f32, d: f32, h: f32, cx: f32, cy: f32, cz: f32, color: [f32; 3]) -> BuildingMesh {
    // box_mesh params: (width=X, height=Z, depth=Y)
    BuildingMesh {
        mesh: MeshData::box_mesh(w, h, d),
        model_matrix: Matrix4::new_translation(&Vector3::new(cx, cy, cz)),
        color,
    }
}

/// Gable roof with exact vertex positions — no rotation math.
/// Every vertex snaps precisely to the building geometry.
fn make_gable_roof(
    fw: f32, fd: f32, base_z: f32, pitch: f32, overhang: f32,
    ridge_along_x: bool, ox: f32, oy: f32,
    roof_color: [f32; 3], wall_color: [f32; 3],
) -> Vec<BuildingMesh> {
    let mut out = Vec::new();

    if ridge_along_x {
        // Ridge runs along X. Slopes face south and north.
        let half_span = fd / 2.0;
        let ridge_h = pitch * half_span;
        let ridge_cy = oy + fd / 2.0;

        // Exact vertex positions:
        // Eave south: y=oy-overhang, z=base_z (with overhang extension)
        // Eave north: y=oy+fd+overhang, z=base_z
        // Ridge: y=ridge_cy, z=base_z+ridge_h
        // X range: ox-overhang to ox+fw+overhang
        let x0 = ox - overhang;
        let x1 = ox + fw + overhang;
        let eave_s = oy - overhang;
        let eave_n = oy + fd + overhang;
        let rz = base_z + ridge_h;

        // Compute slope normal for south face
        let slope_dy = half_span + overhang;
        let slope_dz = ridge_h;
        let slope_normal_len = (slope_dy * slope_dy + slope_dz * slope_dz).sqrt();
        let sn_y = -slope_dz / slope_normal_len;
        let sn_z = slope_dy / slope_normal_len;

        // South slope: quad from south eave to ridge
        out.push(make_quad(
            [x0, eave_s, base_z], [x1, eave_s, base_z],
            [x1, ridge_cy, rz],   [x0, ridge_cy, rz],
            [0.0, sn_y, sn_z],
            roof_color,
        ));
        // North slope: quad from ridge to north eave
        out.push(make_quad(
            [x0, ridge_cy, rz],   [x1, ridge_cy, rz],
            [x1, eave_n, base_z], [x0, eave_n, base_z],
            [0.0, -sn_y, sn_z],
            roof_color,
        ));

        // Gable end walls — extend to match roof overhang in both directions
        for &x in &[x0, x1] {
            let nx = if x < ox + fw / 2.0 { -1.0 } else { 1.0 };
            out.push(make_triangle(
                [x, eave_s, base_z],
                [x, eave_n, base_z],
                [x, ridge_cy, rz],
                [nx, 0.0, 0.0],
                wall_color,
            ));
        }
    } else {
        // Ridge runs along Y. Slopes face east and west.
        let half_span = fw / 2.0;
        let ridge_h = pitch * half_span;
        let ridge_cx = ox + fw / 2.0;

        let y0 = oy - overhang;
        let y1 = oy + fd + overhang;
        let eave_w = ox - overhang;
        let eave_e = ox + fw + overhang;
        let rz = base_z + ridge_h;

        let slope_dx = half_span + overhang;
        let slope_dz = ridge_h;
        let slope_normal_len = (slope_dx * slope_dx + slope_dz * slope_dz).sqrt();
        let sn_x = -slope_dz / slope_normal_len;
        let sn_z = slope_dx / slope_normal_len;

        // West slope
        out.push(make_quad(
            [eave_w, y0, base_z], [eave_w, y1, base_z],
            [ridge_cx, y1, rz],   [ridge_cx, y0, rz],
            [sn_x, 0.0, sn_z],
            roof_color,
        ));
        // East slope
        out.push(make_quad(
            [ridge_cx, y0, rz],   [ridge_cx, y1, rz],
            [eave_e, y1, base_z], [eave_e, y0, base_z],
            [-sn_x, 0.0, sn_z],
            roof_color,
        ));

        // Gable end walls — extend to match roof overhang in both directions
        for &y in &[y0, y1] {
            let ny = if y < oy + fd / 2.0 { -1.0 } else { 1.0 };
            out.push(make_triangle(
                [eave_w, y, base_z],
                [eave_e, y, base_z],
                [ridge_cx, y, rz],
                [0.0, ny, 0.0],
                wall_color,
            ));
        }
    }

    out
}

/// Double-sided quad mesh with explicit vertices. a,b,c,d in CCW order for the front face.
fn make_quad(
    a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3],
    normal: [f32; 3], color: [f32; 3],
) -> BuildingMesh {
    let back = [-normal[0], -normal[1], -normal[2]];
    BuildingMesh {
        mesh: MeshData {
            positions: vec![a, b, c, d, d, c, b, a],
            normals: vec![normal, normal, normal, normal, back, back, back, back],
            indices: vec![0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7],
            edges: None,
        },
        model_matrix: Matrix4::identity(),
        color,
    }
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
