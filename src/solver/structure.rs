use nalgebra::{Matrix4, Vector3};

use crate::oil::types::{Cardinal, Feature};
use crate::orb::mesh::MeshData;
use super::types::*;

pub struct BuildingMesh {
    pub mesh: MeshData,
    pub model_matrix: Matrix4<f32>,
    pub color: [f32; 3],
    /// If true, skip this mesh in the shadow pass (prevents self-shadow acne on thin quads)
    pub no_shadow: bool,
}

// Window/door dimensions
const WIN_W: f32 = 1.0;
const WIN_H: f32 = 1.2;
const WIN_SILL: f32 = 0.9;
const DOOR_W: f32 = 0.9;
const DOOR_H: f32 = 2.1;
const FRAME_T: f32 = 0.05;
const FRAME_COLOR: [f32; 3] = [0.30, 0.25, 0.20];
const GLASS_COLOR: [f32; 3] = [0.65, 0.75, 0.85];

#[derive(Debug, Clone, Copy, PartialEq)]
enum WallFace { South, North, West, East }

#[derive(Debug, Clone)]
struct WallOpening {
    center: f32,  // position along wall's primary axis
    width: f32,
    height: f32,
    sill: f32,    // bottom of opening above floor
}

/// Generate building geometry with wall openings, interior walls, roof.
pub fn generate_building_meshes(building: &SolvedBuilding) -> Vec<BuildingMesh> {
    let mut meshes = Vec::new();
    let fw = building.footprint_width as f32;
    let fd = building.footprint_depth as f32;
    let ext = building.style.exterior_wall_thickness as f32;
    let int_t = building.style.wall_thickness as f32;
    let slab_t = building.style.floor_thickness as f32;
    let ox = -fw / 2.0;
    let oy = -fd / 2.0;

    for floor in &building.floors {
        let z = floor.elevation as f32;
        let h = floor.ceiling_height as f32;

        // Floor slab
        meshes.push(make_box(fw, fd, slab_t, ox + fw/2.0, oy + fd/2.0, z - slab_t/2.0, building.style.floor_color));

        // Collect openings per wall face
        let openings = collect_openings(&floor.rooms, fw, fd, ext);

        // Exterior walls with openings
        // South wall (along X, y = ext/2)
        meshes.extend(wall_with_openings(
            0.0, fw, ext, h, z, oy + ext/2.0, ox,
            WallFace::South, &openings, building.style.exterior_color,
        ));
        // North wall (along X, y = fd - ext/2)
        meshes.extend(wall_with_openings(
            0.0, fw, ext, h, z, oy + fd - ext/2.0, ox,
            WallFace::North, &openings, building.style.exterior_color,
        ));
        // West wall (along Y, x = ext/2) — fits between S and N
        meshes.extend(wall_with_openings(
            ext, fd - ext, ext, h, z, ox + ext/2.0, oy,
            WallFace::West, &openings, building.style.exterior_color,
        ));
        // East wall (along Y, x = fw - ext/2) — fits between S and N
        meshes.extend(wall_with_openings(
            ext, fd - ext, ext, h, z, ox + fw - ext/2.0, oy,
            WallFace::East, &openings, building.style.exterior_color,
        ));

        // Ceiling (no_shadow to prevent shadowing the roof from below)
        let mut ceiling = make_box(fw, fd, slab_t, ox + fw/2.0, oy + fd/2.0, z + h + slab_t/2.0, [0.92, 0.90, 0.85]);
        ceiling.no_shadow = true;
        meshes.push(ceiling);

        // Interior walls
        meshes.extend(generate_interior_walls(&floor.rooms, fw, fd, ext, int_t, h, z, ox, oy, building.style.interior_wall_color));
    }

    // Ground plane
    let gs = fw.max(fd) * 2.0;
    meshes.push(make_box(gs, gs, 0.05, 0.0, 0.0, -slab_t - 0.05, [0.42, 0.50, 0.32]));

    // Roof
    if let Some(ref roof) = building.roof {
        let slab_t = building.style.floor_thickness as f32;
        let top = building.floors.last().unwrap();
        let base_z = top.elevation as f32 + top.ceiling_height as f32 + slab_t;
        meshes.extend(make_gable_roof(
            fw, fd, base_z, roof.pitch_ratio as f32, building.style.roof_overhang as f32,
            roof.ridge_along_x, ox, oy, building.style.roof_color, building.style.exterior_color,
        ));
    }

    meshes
}

/// Collect window/door openings from rooms, mapped to wall faces.
fn collect_openings(rooms: &[SolvedRoom], fw: f32, fd: f32, ext: f32) -> Vec<(WallFace, WallOpening)> {
    let mut openings = Vec::new();
    let eps = 0.05;

    for room in rooms {
        let rx = room.x as f32;
        let ry = room.y as f32;
        let rw = room.width as f32;
        let rd = room.depth as f32;

        // Which walls does this room touch?
        let touches_south = ry <= ext + eps;
        let touches_north = ry + rd >= fd - ext - eps;
        let touches_west = rx <= ext + eps;
        let touches_east = rx + rw >= fw - ext - eps;

        // Place windows
        for wspec in &room.windows {
            let face = match wspec.direction {
                Cardinal::South if touches_south => Some(WallFace::South),
                Cardinal::North if touches_north => Some(WallFace::North),
                Cardinal::West if touches_west => Some(WallFace::West),
                Cardinal::East if touches_east => Some(WallFace::East),
                _ => None, // room doesn't touch the requested wall
            };

            if let Some(face) = face {
                let (span_start, span_end) = room_span_on_wall(face, rx, ry, rw, rd);
                let span = span_end - span_start;
                let count = wspec.count.min((span / (WIN_W + 0.3)) as u32).max(1);
                let segment = span / count as f32;

                for i in 0..count {
                    openings.push((face, WallOpening {
                        center: span_start + segment * (i as f32 + 0.5),
                        width: WIN_W,
                        height: WIN_H,
                        sill: WIN_SILL,
                    }));
                }
            }
        }

        // Place doors
        for feat in &room.features {
            let (face, wall_touches) = match feat {
                Feature::FrontDoor => (WallFace::South, touches_south),
                Feature::BackDoor => (WallFace::North, touches_north),
                _ => continue,
            };
            if wall_touches {
                let (span_start, span_end) = room_span_on_wall(face, rx, ry, rw, rd);
                let center = (span_start + span_end) / 2.0;
                openings.push((face, WallOpening {
                    center,
                    width: DOOR_W,
                    height: DOOR_H,
                    sill: 0.0,
                }));
            }
        }
    }

    openings
}

/// Get the span of a room along a wall's primary axis.
fn room_span_on_wall(face: WallFace, rx: f32, ry: f32, rw: f32, rd: f32) -> (f32, f32) {
    match face {
        WallFace::South | WallFace::North => (rx, rx + rw),
        WallFace::West | WallFace::East => (ry, ry + rd),
    }
}

/// Generate a wall with openings cut out. Returns wall segments + frame meshes.
fn wall_with_openings(
    along_start: f32, along_end: f32,
    thickness: f32, height: f32, floor_z: f32,
    perp_pos: f32,  // position on the perpendicular axis (world coords)
    axis_offset: f32, // offset to convert along-axis to world coords
    face: WallFace,
    all_openings: &[(WallFace, WallOpening)],
    color: [f32; 3],
) -> Vec<BuildingMesh> {
    let mut meshes = Vec::new();

    // Filter and sort openings for this face
    let mut face_openings: Vec<&WallOpening> = all_openings
        .iter()
        .filter(|(f, _)| *f == face)
        .map(|(_, o)| o)
        .collect();
    face_openings.sort_by(|a, b| a.center.partial_cmp(&b.center).unwrap());

    if face_openings.is_empty() {
        // Solid wall, no openings
        let len = along_end - along_start;
        let mid = (along_start + along_end) / 2.0;
        meshes.push(emit_wall_box(face, mid, 0.0, height, len, thickness, floor_z, perp_pos, axis_offset, color));
        return meshes;
    }

    // Split wall into segments around openings
    let mut cursor = along_start;

    for opening in &face_openings {
        let o_left = (opening.center - opening.width / 2.0).max(along_start);
        let o_right = (opening.center + opening.width / 2.0).min(along_end);
        let o_bottom = opening.sill;
        let o_top = opening.sill + opening.height;

        // Solid wall from cursor to opening left (full height)
        if o_left > cursor + 0.01 {
            let len = o_left - cursor;
            let mid = (cursor + o_left) / 2.0;
            meshes.push(emit_wall_box(face, mid, 0.0, height, len, thickness, floor_z, perp_pos, axis_offset, color));
        }

        // Below opening (sill wall)
        if o_bottom > 0.01 {
            let len = o_right - o_left;
            let mid = (o_left + o_right) / 2.0;
            meshes.push(emit_wall_box(face, mid, 0.0, o_bottom, len, thickness, floor_z, perp_pos, axis_offset, color));
        }

        // Above opening (lintel wall)
        if o_top < height - 0.01 {
            let len = o_right - o_left;
            let mid = (o_left + o_right) / 2.0;
            meshes.push(emit_wall_box(face, mid, o_top, height, len, thickness, floor_z, perp_pos, axis_offset, color));
        }

        // Window/door frame + glass
        meshes.extend(emit_opening_frame(face, opening, thickness, floor_z, perp_pos, axis_offset));

        cursor = o_right;
    }

    // Remaining wall after last opening
    if cursor < along_end - 0.01 {
        let len = along_end - cursor;
        let mid = (cursor + along_end) / 2.0;
        meshes.push(emit_wall_box(face, mid, 0.0, height, len, thickness, floor_z, perp_pos, axis_offset, color));
    }

    meshes
}

/// Emit a wall box segment given wall-local coordinates.
fn emit_wall_box(
    face: WallFace, along_mid: f32, z_bottom: f32, z_top: f32,
    length: f32, thickness: f32, floor_z: f32,
    perp_pos: f32, axis_offset: f32, color: [f32; 3],
) -> BuildingMesh {
    let h = z_top - z_bottom;
    let cz = floor_z + (z_bottom + z_top) / 2.0;
    let along_world = axis_offset + along_mid;

    match face {
        WallFace::South | WallFace::North => {
            make_box(length, thickness, h, along_world, perp_pos, cz, color)
        }
        WallFace::West | WallFace::East => {
            make_box(thickness, length, h, perp_pos, along_world, cz, color)
        }
    }
}

/// Emit window/door frame geometry.
fn emit_opening_frame(
    face: WallFace, opening: &WallOpening,
    wall_thickness: f32, floor_z: f32,
    perp_pos: f32, axis_offset: f32,
) -> Vec<BuildingMesh> {
    let mut meshes = Vec::new();
    let along_world = axis_offset + opening.center;
    let cz_mid = floor_z + opening.sill + opening.height / 2.0;

    // Glass pane (thin flat box at the center of the wall)
    let glass_w = opening.width - FRAME_T * 2.0;
    let glass_h = opening.height - FRAME_T * 2.0;
    if glass_w > 0.0 && glass_h > 0.0 {
        match face {
            WallFace::South | WallFace::North => {
                meshes.push(make_box(glass_w, 0.02, glass_h, along_world, perp_pos, cz_mid, GLASS_COLOR));
            }
            WallFace::West | WallFace::East => {
                meshes.push(make_box(0.02, glass_w, glass_h, perp_pos, along_world, cz_mid, GLASS_COLOR));
            }
        }
    }

    // Frame: 4 border pieces around the opening
    let fw = opening.width;
    let fh = opening.height;
    let ft = FRAME_T;
    let z_bottom = floor_z + opening.sill;
    let z_top = z_bottom + fh;

    // Top and bottom frame bars (along the wall)
    for &z in &[z_bottom + ft/2.0, z_top - ft/2.0] {
        match face {
            WallFace::South | WallFace::North => {
                meshes.push(make_box(fw, ft, ft, along_world, perp_pos, z, FRAME_COLOR));
            }
            WallFace::West | WallFace::East => {
                meshes.push(make_box(ft, fw, ft, perp_pos, along_world, z, FRAME_COLOR));
            }
        }
    }

    // Left and right frame bars (vertical)
    let left_along = axis_offset + opening.center - fw/2.0 + ft/2.0;
    let right_along = axis_offset + opening.center + fw/2.0 - ft/2.0;
    let bar_h = fh - ft * 2.0;
    if bar_h > 0.0 {
        for &along in &[left_along, right_along] {
            match face {
                WallFace::South | WallFace::North => {
                    meshes.push(make_box(ft, ft, bar_h, along, perp_pos, cz_mid, FRAME_COLOR));
                }
                WallFace::West | WallFace::East => {
                    meshes.push(make_box(ft, ft, bar_h, perp_pos, along, cz_mid, FRAME_COLOR));
                }
            }
        }
    }

    meshes
}

/// Generate interior walls from room boundaries.
fn generate_interior_walls(
    rooms: &[SolvedRoom], fw: f32, fd: f32, ext: f32, int_t: f32,
    h: f32, z: f32, ox: f32, oy: f32, color: [f32; 3],
) -> Vec<BuildingMesh> {
    let mut meshes = Vec::new();
    let eps = 0.05;
    let mut segments: Vec<(f32, f32, f32, f32)> = Vec::new(); // (x1,y1,x2,y2)

    for room in rooms {
        let rx = room.x as f32;
        let ry = room.y as f32;
        let rw = room.width as f32;
        let rd = room.depth as f32;

        let edges = [
            (rx, ry, rx + rw, ry),           // south edge
            (rx, ry + rd, rx + rw, ry + rd),  // north edge
            (rx, ry, rx, ry + rd),            // west edge
            (rx + rw, ry, rx + rw, ry + rd),  // east edge
        ];

        for (ex1, ey1, ex2, ey2) in edges {
            // Skip edges at footprint boundary (handled by exterior walls)
            let near_boundary = ex1 < ext + eps && ex2 < ext + eps
                || (fw - ex1) < ext + eps && (fw - ex2) < ext + eps
                || ey1 < ext + eps && ey2 < ext + eps
                || (fd - ey1) < ext + eps && (fd - ey2) < ext + eps;
            if near_boundary { continue; }

            // Deduplicate (same segment from adjacent room)
            let is_dup = segments.iter().any(|&(sx1, sy1, sx2, sy2)| {
                let same_h = (ey1 - ey2).abs() < eps && (sy1 - sy2).abs() < eps && (ey1 - sy1).abs() < eps;
                let same_v = (ex1 - ex2).abs() < eps && (sx1 - sx2).abs() < eps && (ex1 - sx1).abs() < eps;
                if same_h {
                    let (a1, a2) = (ex1.min(ex2), ex1.max(ex2));
                    let (b1, b2) = (sx1.min(sx2), sx1.max(sx2));
                    a1 < b2 + eps && b1 < a2 + eps
                } else if same_v {
                    let (a1, a2) = (ey1.min(ey2), ey1.max(ey2));
                    let (b1, b2) = (sy1.min(sy2), sy1.max(sy2));
                    a1 < b2 + eps && b1 < a2 + eps
                } else {
                    false
                }
            });
            if is_dup { continue; }

            segments.push((ex1, ey1, ex2, ey2));

            // Clip to interior zone
            let cx1 = ex1.clamp(ext, fw - ext);
            let cy1 = ey1.clamp(ext, fd - ext);
            let cx2 = ex2.clamp(ext, fw - ext);
            let cy2 = ey2.clamp(ext, fd - ext);

            let dx = (cx2 - cx1).abs();
            let dy = (cy2 - cy1).abs();
            if dx < eps && dy < eps { continue; }

            let mid_x = (cx1 + cx2) / 2.0;
            let mid_y = (cy1 + cy2) / 2.0;
            let (w, d) = if dy < eps {
                (dx, int_t) // horizontal
            } else {
                (int_t, dy) // vertical
            };

            meshes.push(make_box(w, d, h, ox + mid_x, oy + mid_y, z + h/2.0, color));
        }
    }

    meshes
}

/// Generate edge outlines automatically from ALL building meshes.
/// Every box_mesh gets 12 edges. Skips the ground plane (too large).
pub fn generate_edge_meshes(building: &SolvedBuilding, thickness: f32) -> Vec<BuildingMesh> {
    let meshes = generate_building_meshes(building);
    let black = [0.0, 0.0, 0.0];
    let ground_z = -(building.style.floor_thickness as f32) - 0.05;

    let mut edges = Vec::new();
    for m in &meshes {
        // Skip ground plane (edges would be huge)
        let pos = m.model_matrix.column(3);
        if (pos.z - ground_z).abs() < 0.1 && m.mesh.positions.len() == 24 {
            // Heuristic: ground plane is near ground_z and is a box
            let aabb_w = m.mesh.positions.iter().map(|p| p[0]).fold(f32::MIN, f32::max)
                - m.mesh.positions.iter().map(|p| p[0]).fold(f32::MAX, f32::min);
            if aabb_w > 10.0 { continue; } // skip large ground-like meshes
        }

        // Extract AABB from the mesh positions to determine box dimensions
        if m.mesh.positions.len() == 24 {
            // This is a box_mesh (6 faces × 4 vertices)
            let (mut min_x, mut min_y, mut min_z) = (f32::MAX, f32::MAX, f32::MAX);
            let (mut max_x, mut max_y, mut max_z) = (f32::MIN, f32::MIN, f32::MIN);
            for p in &m.mesh.positions {
                min_x = min_x.min(p[0]); max_x = max_x.max(p[0]);
                min_y = min_y.min(p[1]); max_y = max_y.max(p[1]);
                min_z = min_z.min(p[2]); max_z = max_z.max(p[2]);
            }
            let w = max_x - min_x;
            let d = max_y - min_y;
            let h = max_z - min_z;
            // Skip very thin meshes (glass panes, room planes)
            if w < 0.05 || d < 0.05 || h < 0.05 { continue; }
            let cx = m.model_matrix[(0, 3)];
            let cy = m.model_matrix[(1, 3)];
            let cz = m.model_matrix[(2, 3)];
            edges.extend(box_edges(w, d, h, cx, cy, cz, thickness, black));
        }
        // Quads and triangles (non-box meshes) — skip for now
        // TODO: extract edges from arbitrary meshes
    }
    edges
}

fn box_edges(w: f32, d: f32, h: f32, cx: f32, cy: f32, cz: f32, t: f32, color: [f32; 3]) -> Vec<BuildingMesh> {
    let hw = w / 2.0;
    let hd = d / 2.0;
    let hh = h / 2.0;
    let mut edges = Vec::new();
    for &dy in &[-hd, hd] {
        for &dz in &[-hh, hh] {
            edges.push(make_box(w + t, t, t, cx, cy + dy, cz + dz, color));
        }
    }
    for &dx in &[-hw, hw] {
        for &dz in &[-hh, hh] {
            edges.push(make_box(t, d + t, t, cx + dx, cy, cz + dz, color));
        }
    }
    for &dx in &[-hw, hw] {
        for &dy in &[-hd, hd] {
            edges.push(make_box(t, t, h + t, cx + dx, cy + dy, cz, color));
        }
    }
    edges
}

/// Create a thin box edge strip between two 3D points.
fn line_edge(a: [f32; 3], b: [f32; 3], t: f32, color: [f32; 3]) -> BuildingMesh {
    let cx = (a[0] + b[0]) / 2.0;
    let cy = (a[1] + b[1]) / 2.0;
    let cz = (a[2] + b[2]) / 2.0;
    let dx = (b[0] - a[0]).abs();
    let dy = (b[1] - a[1]).abs();
    let dz = (b[2] - a[2]).abs();
    // The box dimensions: use the span along each axis, with minimum t for thin dimensions
    let w = if dx > t { dx } else { t };
    let d = if dy > t { dy } else { t };
    let h = if dz > t { dz } else { t };
    make_box(w, d, h, cx, cy, cz, color)
}

/// Create a box with w=X-width, d=Y-depth, h=Z-height centered at (cx,cy,cz).
fn make_box(w: f32, d: f32, h: f32, cx: f32, cy: f32, cz: f32, color: [f32; 3]) -> BuildingMesh {
    BuildingMesh {
        mesh: MeshData::box_mesh(w, h, d),
        model_matrix: Matrix4::new_translation(&Vector3::new(cx, cy, cz)),
        color,
        no_shadow: false,
    }
}

// === Roof ===

fn make_gable_roof(
    fw: f32, fd: f32, base_z: f32, pitch: f32, overhang: f32,
    ridge_along_x: bool, ox: f32, oy: f32,
    roof_color: [f32; 3], wall_color: [f32; 3],
) -> Vec<BuildingMesh> {
    let mut out = Vec::new();
    let fascia_h = 0.2;    // fascia board height
    let fascia_t = 0.03;   // fascia board thickness
    let ridge_w = 0.15;    // ridge cap width
    let ridge_h = 0.08;    // ridge cap height above slope
    let soffit_t = 0.03;   // soffit panel thickness
    let trim_color = [0.30, 0.25, 0.20]; // dark wood trim

    if ridge_along_x {
        let half_span = fd / 2.0;
        let rh = pitch * half_span;
        let ridge_cy = oy + fd / 2.0;
        let x0 = ox - overhang;
        let x1 = ox + fw + overhang;
        let eave_s = oy - overhang;
        let eave_n = oy + fd + overhang;
        let rz = base_z + rh;
        let roof_len = x1 - x0;

        let slope_dy = half_span + overhang;
        let slope_dz = rh;
        let slen = (slope_dy * slope_dy + slope_dz * slope_dz).sqrt();
        let sn_y = -slope_dz / slen;
        let sn_z = slope_dy / slen;

        // Roof slope surfaces
        out.push(make_quad([x0, eave_s, base_z], [x1, eave_s, base_z], [x1, ridge_cy, rz], [x0, ridge_cy, rz], [0.0, sn_y, sn_z], roof_color));
        out.push(make_quad([x0, ridge_cy, rz], [x1, ridge_cy, rz], [x1, eave_n, base_z], [x0, eave_n, base_z], [0.0, -sn_y, sn_z], roof_color));

        // Gable end walls
        for &x in &[x0, x1] {
            let nx = if x < ox + fw / 2.0 { -1.0 } else { 1.0 };
            out.push(make_triangle([x, eave_s, base_z], [x, eave_n, base_z], [x, ridge_cy, rz], [nx, 0.0, 0.0], wall_color));
        }

        // Ridge cap (box running along ridge)
        let mid_x = (x0 + x1) / 2.0;
        out.push(make_box(roof_len, ridge_w, ridge_h, mid_x, ridge_cy, rz + ridge_h / 2.0, trim_color));

        // Fascia boards along eaves (vertical boards at eave edge)
        out.push(make_box(roof_len, fascia_t, fascia_h, mid_x, eave_s, base_z - fascia_h / 2.0, trim_color));
        out.push(make_box(roof_len, fascia_t, fascia_h, mid_x, eave_n, base_z - fascia_h / 2.0, trim_color));

        // Soffit panels (horizontal under overhang, from wall face to eave)
        let soffit_south_d = oy - eave_s; // overhang depth on south side
        if soffit_south_d > 0.01 {
            let soffit_cy = (oy + eave_s) / 2.0;
            out.push(make_box(roof_len, soffit_south_d, soffit_t, mid_x, soffit_cy, base_z - fascia_h, trim_color));
        }
        let soffit_north_d = eave_n - (oy + fd);
        if soffit_north_d > 0.01 {
            let soffit_cy = (oy + fd + eave_n) / 2.0;
            out.push(make_box(roof_len, soffit_north_d, soffit_t, mid_x, soffit_cy, base_z - fascia_h, trim_color));
        }

        // Rake boards along gable edges (vertical boards at gable ends)
        // These are at x0 and x1, running vertically from eave to near-ridge
        out.push(make_box(fascia_t, fascia_t, rh + fascia_h, x0, ridge_cy, base_z + rh / 2.0 - fascia_h / 2.0, trim_color));
        out.push(make_box(fascia_t, fascia_t, rh + fascia_h, x1, ridge_cy, base_z + rh / 2.0 - fascia_h / 2.0, trim_color));
    } else {
        let half_span = fw / 2.0;
        let rh = pitch * half_span;
        let ridge_cx = ox + fw / 2.0;
        let y0 = oy - overhang;
        let y1 = oy + fd + overhang;
        let eave_w = ox - overhang;
        let eave_e = ox + fw + overhang;
        let rz = base_z + rh;
        let roof_len = y1 - y0;

        let slope_dx = half_span + overhang;
        let slope_dz = rh;
        let slen = (slope_dx * slope_dx + slope_dz * slope_dz).sqrt();
        let sn_x = -slope_dz / slen;
        let sn_z = slope_dx / slen;

        out.push(make_quad([eave_w, y0, base_z], [eave_w, y1, base_z], [ridge_cx, y1, rz], [ridge_cx, y0, rz], [sn_x, 0.0, sn_z], roof_color));
        out.push(make_quad([ridge_cx, y0, rz], [ridge_cx, y1, rz], [eave_e, y1, base_z], [eave_e, y0, base_z], [-sn_x, 0.0, sn_z], roof_color));

        for &y in &[y0, y1] {
            let ny = if y < oy + fd / 2.0 { -1.0 } else { 1.0 };
            out.push(make_triangle([eave_w, y, base_z], [eave_e, y, base_z], [ridge_cx, y, rz], [0.0, ny, 0.0], wall_color));
        }

        let mid_y = (y0 + y1) / 2.0;
        out.push(make_box(ridge_w, roof_len, ridge_h, ridge_cx, mid_y, rz + ridge_h / 2.0, trim_color));
        out.push(make_box(fascia_t, roof_len, fascia_h, eave_w, mid_y, base_z - fascia_h / 2.0, trim_color));
        out.push(make_box(fascia_t, roof_len, fascia_h, eave_e, mid_y, base_z - fascia_h / 2.0, trim_color));

        let soffit_west_d = ox - eave_w;
        if soffit_west_d > 0.01 {
            let soffit_cx = (ox + eave_w) / 2.0;
            out.push(make_box(soffit_west_d, roof_len, soffit_t, soffit_cx, mid_y, base_z - fascia_h, trim_color));
        }
        let soffit_east_d = eave_e - (ox + fw);
        if soffit_east_d > 0.01 {
            let soffit_cx = (ox + fw + eave_e) / 2.0;
            out.push(make_box(soffit_east_d, roof_len, soffit_t, soffit_cx, mid_y, base_z - fascia_h, trim_color));
        }

        out.push(make_box(fascia_t, fascia_t, rh + fascia_h, ridge_cx, y0, base_z + rh / 2.0 - fascia_h / 2.0, trim_color));
        out.push(make_box(fascia_t, fascia_t, rh + fascia_h, ridge_cx, y1, base_z + rh / 2.0 - fascia_h / 2.0, trim_color));
    }
    out
}

fn make_quad(a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3], normal: [f32; 3], color: [f32; 3]) -> BuildingMesh {
    // Single-sided quad. cull_mode: None renders both winding orders.
    // Duplicate back-face vertices caused z-fighting checkerboard artifacts.
    BuildingMesh {
        no_shadow: true,
        mesh: MeshData {
            positions: vec![a, b, c, d],
            normals: vec![normal, normal, normal, normal],
            indices: vec![0, 1, 2, 0, 2, 3],
            edges: None,
        },
        model_matrix: Matrix4::identity(),
        color,
    }
}

fn make_triangle(a: [f32; 3], b: [f32; 3], c: [f32; 3], normal: [f32; 3], color: [f32; 3]) -> BuildingMesh {
    // Single-sided triangle. cull_mode: None handles back-face visibility.
    BuildingMesh {
        no_shadow: true,
        mesh: MeshData {
            positions: vec![a, b, c],
            normals: vec![normal, normal, normal],
            indices: vec![0, 1, 2],
            edges: None,
        },
        model_matrix: Matrix4::identity(),
        color,
    }
}
