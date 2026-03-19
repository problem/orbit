use super::types::*;

/// A 2D rectangle for floor plan layout.
#[derive(Debug, Clone, Copy)]
struct Rect {
    x: f64,
    y: f64,
    width: f64,
    depth: f64,
}

/// Solve the floor plan layout using adjacency-guided binary space partition.
///
/// Returns solved rooms with 2D positions in meters, origin at footprint SW corner.
pub fn solve_floor_plan(
    rooms: &[ResolvedRoom],
    footprint_width: f64,
    footprint_depth: f64,
    style: &ResolvedStyle,
) -> Vec<SolvedRoom> {
    if rooms.is_empty() {
        return Vec::new();
    }

    // Inset footprint by exterior wall thickness
    let inset = style.exterior_wall_thickness;
    let usable = Rect {
        x: inset,
        y: inset,
        width: footprint_width - 2.0 * inset,
        depth: footprint_depth - 2.0 * inset,
    };

    // Order rooms for good adjacency placement
    let ordered = order_rooms_for_layout(rooms);

    // Recursive binary space partition
    partition(usable, &ordered)
}

/// Order rooms to maximize adjacency satisfaction in BSP.
/// Uses a BFS from the entry/most-connected room.
fn order_rooms_for_layout(rooms: &[ResolvedRoom]) -> Vec<ResolvedRoom> {
    if rooms.len() <= 1 {
        return rooms.to_vec();
    }

    // Find the start room: prefer entry (has front_door), else most connections
    let start = rooms
        .iter()
        .enumerate()
        .max_by_key(|(_, r)| {
            let is_entry = r.features.iter().any(|f| {
                matches!(f, crate::oil::types::Feature::FrontDoor)
            });
            let connectivity = r.adjacent_to.len() + r.connects.len();
            (is_entry as usize * 100, connectivity)
        })
        .map(|(i, _)| i)
        .unwrap_or(0);

    // BFS traversal
    let mut visited = vec![false; rooms.len()];
    let mut order = Vec::with_capacity(rooms.len());
    let mut queue = std::collections::VecDeque::new();

    visited[start] = true;
    queue.push_back(start);

    while let Some(idx) = queue.pop_front() {
        order.push(rooms[idx].clone());

        // Find neighbors by adjacency/connectivity
        let neighbors: Vec<&str> = rooms[idx]
            .adjacent_to
            .iter()
            .chain(rooms[idx].connects.iter())
            .map(|s| s.as_str())
            .collect();

        for (j, room) in rooms.iter().enumerate() {
            if !visited[j] && neighbors.contains(&room.name.as_str()) {
                visited[j] = true;
                queue.push_back(j);
            }
        }
    }

    // Add any unvisited rooms (disconnected from the graph)
    for (j, room) in rooms.iter().enumerate() {
        if !visited[j] {
            order.push(room.clone());
        }
    }

    // Move side-pinned rooms to appropriate positions
    // Rooms pinned to west should be early (left in BSP), east should be late
    let mut pinned_west: Vec<ResolvedRoom> = Vec::new();
    let mut unpinned: Vec<ResolvedRoom> = Vec::new();
    let mut pinned_east: Vec<ResolvedRoom> = Vec::new();

    for room in order {
        match room.side_pin {
            Some(crate::oil::types::Cardinal::West | crate::oil::types::Cardinal::Northwest | crate::oil::types::Cardinal::Southwest) => {
                pinned_west.push(room);
            }
            Some(crate::oil::types::Cardinal::East | crate::oil::types::Cardinal::Northeast | crate::oil::types::Cardinal::Southeast) => {
                pinned_east.push(room);
            }
            _ => unpinned.push(room),
        }
    }

    let mut result = Vec::new();
    result.extend(pinned_west);
    result.extend(unpinned);
    result.extend(pinned_east);
    result
}

/// Recursively partition a rectangle among rooms, proportional to target areas.
fn partition(rect: Rect, rooms: &[ResolvedRoom]) -> Vec<SolvedRoom> {
    if rooms.len() == 1 {
        return vec![SolvedRoom {
            name: rooms[0].name.clone(),
            room_type: rooms[0].room_type,
            x: rect.x,
            y: rect.y,
            width: rect.width,
            depth: rect.depth,
            windows: rooms[0].windows.clone(),
            features: rooms[0].features.clone(),
        }];
    }

    if rooms.is_empty() {
        return Vec::new();
    }

    // Split rooms into two groups as close to 50/50 area as possible
    let total_area: f64 = rooms.iter().map(|r| r.target_area).sum();
    let half_target = total_area / 2.0;

    let mut best_split = 1;
    let mut best_diff = f64::MAX;
    let mut running = 0.0;
    for i in 0..rooms.len() - 1 {
        running += rooms[i].target_area;
        let diff = (running - half_target).abs();
        if diff < best_diff {
            best_diff = diff;
            best_split = i + 1;
        }
    }

    let (left_rooms, right_rooms) = rooms.split_at(best_split);
    let left_area: f64 = left_rooms.iter().map(|r| r.target_area).sum();
    let fraction = left_area / total_area;

    // Split along the longer dimension
    if rect.width >= rect.depth {
        // Vertical split (left | right)
        let split_w = rect.width * fraction;
        let left_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: split_w,
            depth: rect.depth,
        };
        let right_rect = Rect {
            x: rect.x + split_w,
            y: rect.y,
            width: rect.width - split_w,
            depth: rect.depth,
        };
        let mut result = partition(left_rect, left_rooms);
        result.extend(partition(right_rect, right_rooms));
        result
    } else {
        // Horizontal split (bottom / top)
        let split_d = rect.depth * fraction;
        let bottom_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            depth: split_d,
        };
        let top_rect = Rect {
            x: rect.x,
            y: rect.y + split_d,
            width: rect.width,
            depth: rect.depth - split_d,
        };
        let mut result = partition(bottom_rect, left_rooms);
        result.extend(partition(top_rect, right_rooms));
        result
    }
}
