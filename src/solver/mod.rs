pub mod layout;
pub mod structure;
pub mod style;
pub mod types;

use anyhow::Result;

use crate::oil::ast::Program;
use types::*;

/// Solve an OIL program into a 3D building model.
pub fn solve(program: &Program) -> Result<SolvedBuilding> {
    let house = match program {
        Program::House(h) => h,
        Program::Furniture(_) => anyhow::bail!("furniture solving not yet implemented"),
    };

    let resolved_style = style::resolve_style(house);
    let (fp_width, fp_depth) = style::resolve_footprint(house);

    let mut floors = Vec::new();
    let mut elevation = 0.0;
    let mut diagnostics = Vec::new();

    for (i, floor_block) in house.floors.iter().enumerate() {
        let resolved_rooms = style::resolve_rooms(floor_block, &resolved_style);

        // Check total area fits in footprint
        let total_room_area: f64 = resolved_rooms.iter().map(|r| r.target_area).sum();
        let usable_area = (fp_width - 2.0 * resolved_style.exterior_wall_thickness)
            * (fp_depth - 2.0 * resolved_style.exterior_wall_thickness);
        if total_room_area > usable_area * 1.1 {
            diagnostics.push(Diagnostic {
                level: DiagnosticLevel::Warning,
                message: format!(
                    "floor '{}': total room area ({:.1}sqm) exceeds usable footprint ({:.1}sqm); rooms will be scaled down",
                    floor_block.name, total_room_area, usable_area
                ),
            });
        }

        let ceiling_height = floor_block
            .ceiling_height
            .map(|d| d.to_meters())
            .unwrap_or(if i == 0 {
                resolved_style.ceiling_height
            } else {
                resolved_style.ceiling_height - 0.3 // upper floors slightly lower
            });

        let solved_rooms =
            layout::solve_floor_plan(&resolved_rooms, fp_width, fp_depth, &resolved_style);

        // Log the layout
        for room in &solved_rooms {
            log::info!(
                "  solver: floor '{}' room '{}' -> {:.1}m x {:.1}m at ({:.1}, {:.1}) = {:.1}sqm",
                floor_block.name,
                room.name,
                room.width,
                room.depth,
                room.x,
                room.y,
                room.width * room.depth,
            );
        }

        floors.push(SolvedFloor {
            name: floor_block.name.clone(),
            floor_index: i,
            elevation,
            ceiling_height,
            rooms: solved_rooms,
        });

        elevation += ceiling_height + resolved_style.floor_thickness;
    }

    Ok(SolvedBuilding {
        floors,
        style: resolved_style,
        footprint_width: fp_width,
        footprint_depth: fp_depth,
        diagnostics,
    })
}
