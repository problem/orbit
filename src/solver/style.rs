use crate::oil::ast::*;
use crate::oil::types::*;
use super::types::*;
use crate::oil::types::RoofForm;

/// Resolve style defaults from the AST.
pub fn resolve_style(house: &HouseBlock) -> ResolvedStyle {
    let name = house
        .style
        .as_ref()
        .map(|s| s.name.clone())
        .unwrap_or_else(|| "default".to_string());

    // Start with base defaults
    let mut style = ResolvedStyle {
        name,
        ceiling_height: 2.7,
        wall_thickness: 0.15,
        exterior_wall_thickness: 0.2,
        floor_thickness: 0.3,
        exterior_color: [0.93, 0.90, 0.83],   // cream stucco
        interior_wall_color: [0.95, 0.95, 0.93], // off-white
        floor_color: [0.45, 0.35, 0.25],       // dark wood
        roof_color: [0.35, 0.28, 0.22],        // dark slate/shingle
        roof_overhang: 0.4,                     // 400mm overhang
    };

    // Apply style overrides
    if let Some(ref style_block) = house.style {
        for prop in &style_block.overrides {
            match prop.key.as_str() {
                "ceiling_height" => {
                    if let StyleValue::Text(ref t) = prop.value {
                        if let Ok(dim) = parse_inline_dimension(t) {
                            style.ceiling_height = dim;
                        }
                    }
                }
                "facade_material" => {
                    if let StyleValue::Material(ref m) = prop.value {
                        style.exterior_color = material_to_color(&m.material_type, m.color.as_deref());
                    }
                }
                _ => {}
            }
        }
    }

    style
}

/// Resolve the footprint dimensions from the site block (meters).
pub fn resolve_footprint(house: &HouseBlock) -> (f64, f64) {
    if let Some(ref site) = house.site {
        if let Some((w, d)) = site.footprint {
            return (w.to_meters(), d.to_meters());
        }
    }
    // Default: estimate from total room area
    let total_area: f64 = house
        .floors
        .iter()
        .flat_map(|f| &f.rooms)
        .map(|r| resolve_room_area(r))
        .sum();
    let per_floor = total_area / house.floors.len().max(1) as f64;
    let side = (per_floor * 1.2).sqrt(); // 20% overhead for walls/circulation
    (side, side * 0.75)
}

/// Resolve a room's target area in sqm.
fn resolve_room_area(room: &RoomBlock) -> f64 {
    let room_type = RoomType::infer_from_name(&room.name);
    let (default_area, _) = room_defaults(room_type);

    match &room.area {
        Some(ApproxValue::Approximate(v, u)) if u.is_area() => *v,
        Some(ApproxValue::Approximate(v, _)) => *v,
        Some(ApproxValue::Exact(v, u)) if u.is_area() => *v,
        Some(ApproxValue::Exact(v, _)) => *v,
        Some(ApproxValue::Range(lo, hi, _)) => (lo + hi) / 2.0,
        Some(ApproxValue::Qualitative(q)) => match q.as_str() {
            "large" => default_area * 1.3,
            "small" => default_area * 0.7,
            "extra_large" => default_area * 1.6,
            _ => default_area,
        },
        None => default_area,
    }
}

/// Resolve rooms from a floor block into ResolvedRooms.
pub fn resolve_rooms(floor: &FloorBlock, _style: &ResolvedStyle) -> Vec<ResolvedRoom> {
    floor
        .rooms
        .iter()
        .map(|room| {
            let room_type = RoomType::infer_from_name(&room.name);
            let (default_area, default_aspect) = room_defaults(room_type);

            let (target_area, area_tolerance) = match &room.area {
                Some(ApproxValue::Approximate(v, _)) => (*v, 0.2),
                Some(ApproxValue::Exact(v, _)) => (*v, 0.05),
                Some(ApproxValue::Range(lo, hi, _)) => ((lo + hi) / 2.0, (hi - lo) / (lo + hi)),
                Some(ApproxValue::Qualitative(q)) => {
                    let scale = match q.as_str() {
                        "large" => 1.3,
                        "small" => 0.7,
                        "extra_large" => 1.6,
                        _ => 1.0,
                    };
                    (default_area * scale, 0.2)
                }
                None => (default_area, 0.3),
            };

            let aspect_target = match &room.aspect {
                Some(ApproxValue::Approximate(v, _)) | Some(ApproxValue::Exact(v, _)) => *v,
                _ => default_aspect,
            };

            ResolvedRoom {
                name: room.name.clone(),
                room_type,
                target_area,
                area_tolerance,
                aspect_target,
                side_pin: room.side,
                windows: room.windows.clone(),
                features: room.features.clone(),
                adjacent_to: room.adjacent_to.clone(),
                connects: room.connects.clone(),
            }
        })
        .collect()
}

/// Resolve roof specification from the AST.
pub fn resolve_roof(house: &HouseBlock) -> Option<SolvedRoof> {
    let roof_block = house.roof.as_ref()?;

    let (form, ridge_along_x) = match &roof_block.primary {
        Some(primary) => {
            let ridge_along_x = primary.params.iter().any(|(k, v)| {
                k == "ridge" && (v.contains("east") || v.contains("west"))
            });
            (primary.form.clone(), ridge_along_x)
        }
        None => (RoofForm::Gable, true), // default: gable with east-west ridge
    };

    let pitch_ratio = match &roof_block.pitch {
        Some(p) => {
            if p.run > 0.0 { p.rise / p.run } else { 0.5 }
        }
        None => {
            // Try style overrides for pitch
            house.style.as_ref()
                .and_then(|s| s.overrides.iter().find(|p| p.key == "roof_pitch"))
                .and_then(|p| match &p.value {
                    StyleValue::Pitch(pitch) => {
                        if pitch.run > 0.0 { Some(pitch.rise / pitch.run) } else { None }
                    }
                    _ => None,
                })
                .unwrap_or(0.5) // default 6:12
        }
    };

    Some(SolvedRoof { form, ridge_along_x, pitch_ratio })
}

fn material_to_color(material_type: &str, color: Option<&str>) -> [f32; 3] {
    match (material_type, color) {
        ("stucco", Some("cream")) => [0.96, 0.93, 0.85],
        ("stucco", Some("white")) => [0.97, 0.97, 0.95],
        ("stucco", _) => [0.93, 0.90, 0.83],
        ("brick", Some("red")) => [0.65, 0.25, 0.20],
        ("brick", _) => [0.70, 0.40, 0.30],
        ("timber", _) => [0.40, 0.28, 0.18],
        ("stone", _) => [0.70, 0.68, 0.65],
        _ => [0.85, 0.85, 0.85],
    }
}

fn parse_inline_dimension(text: &str) -> Result<f64, ()> {
    let text = text.trim();
    if let Some(val) = text.strip_suffix("mm") {
        val.trim().parse::<f64>().map(|v| v / 1000.0).map_err(|_| ())
    } else if let Some(val) = text.strip_suffix('m') {
        val.trim().parse::<f64>().map_err(|_| ())
    } else {
        text.parse::<f64>().map_err(|_| ())
    }
}
