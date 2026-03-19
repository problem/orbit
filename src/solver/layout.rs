use super::types::*;

/// A 2D rectangle for floor plan layout.
#[derive(Debug, Clone, Copy)]
struct Rect {
    x: f64,
    y: f64,
    width: f64,
    depth: f64,
}

/// Solve the floor plan layout using BSP + adjacency refinement.
///
/// 1. Order rooms by adjacency (BFS from entry)
/// 2. Recursive BSP partition (area-proportional)
/// 3. Verify adjacency constraints
/// 4. Refine by swapping rooms to improve adjacency score
pub fn solve_floor_plan(
    rooms: &[ResolvedRoom],
    footprint_width: f64,
    footprint_depth: f64,
    style: &ResolvedStyle,
) -> Vec<SolvedRoom> {
    if rooms.is_empty() {
        return Vec::new();
    }

    let inset = style.exterior_wall_thickness;
    let usable = Rect {
        x: inset,
        y: inset,
        width: footprint_width - 2.0 * inset,
        depth: footprint_depth - 2.0 * inset,
    };

    // Phase 1: Order rooms by adjacency graph
    let ordered = order_rooms_for_layout(rooms);

    // Phase 2: Initial BSP layout
    let mut solved = partition(usable, &ordered);

    // Phase 3: Score and log adjacency satisfaction
    let (score, total) = adjacency_score(&solved, rooms);
    log::info!(
        "  layout: initial adjacency score: {}/{} constraints satisfied",
        score, total
    );

    // Phase 4: Refine by swapping room positions to improve adjacency
    if score < total {
        let improved = refine_layout(&solved, rooms, usable);
        let (new_score, _) = adjacency_score(&improved, rooms);
        if new_score > score {
            log::info!(
                "  layout: refined adjacency score: {}/{} (improved by {})",
                new_score, total, new_score - score
            );
            solved = improved;
        }
    }

    solved
}

/// Check if two solved rooms share an edge (are geometrically adjacent).
fn rooms_share_edge(a: &SolvedRoom, b: &SolvedRoom) -> bool {
    let eps = 0.05;
    let min_overlap = 0.1; // at least 10cm of shared edge

    // Vertical shared edge (a's right = b's left, or vice versa)
    let vert_a_right = ((a.x + a.width) - b.x).abs() < eps;
    let vert_b_right = ((b.x + b.width) - a.x).abs() < eps;
    if vert_a_right || vert_b_right {
        // Check Y overlap
        let overlap_start = a.y.max(b.y);
        let overlap_end = (a.y + a.depth).min(b.y + b.depth);
        if overlap_end - overlap_start > min_overlap {
            return true;
        }
    }

    // Horizontal shared edge (a's top = b's bottom, or vice versa)
    let horiz_a_top = ((a.y + a.depth) - b.y).abs() < eps;
    let horiz_b_top = ((b.y + b.depth) - a.y).abs() < eps;
    if horiz_a_top || horiz_b_top {
        // Check X overlap
        let overlap_start = a.x.max(b.x);
        let overlap_end = (a.x + a.width).min(b.x + b.width);
        if overlap_end - overlap_start > min_overlap {
            return true;
        }
    }

    false
}

/// Score a layout: count how many adjacency/connectivity constraints are satisfied.
fn adjacency_score(solved: &[SolvedRoom], original: &[ResolvedRoom]) -> (usize, usize) {
    let mut satisfied = 0;
    let mut total = 0;

    for orig in original {
        let all_required: Vec<&str> = orig
            .adjacent_to
            .iter()
            .chain(orig.connects.iter())
            .map(|s| s.as_str())
            .collect();

        for required_name in &all_required {
            total += 1;
            // Find both rooms in the solved layout
            let room_a = solved.iter().find(|r| r.name == orig.name);
            let room_b = solved.iter().find(|r| r.name.as_str() == *required_name);

            if let (Some(a), Some(b)) = (room_a, room_b) {
                if rooms_share_edge(a, b) {
                    satisfied += 1;
                }
            }
        }
    }

    // Deduplicate bidirectional constraints (A adj B + B adj A = 1 constraint)
    // For simplicity, count each direction separately but report as-is
    (satisfied, total)
}

/// Try to improve the layout by swapping room positions.
/// Simple hill-climbing: try all pairs of swaps, keep the best improvement.
fn refine_layout(
    solved: &[SolvedRoom],
    original: &[ResolvedRoom],
    _usable: Rect,
) -> Vec<SolvedRoom> {
    let mut best = solved.to_vec();
    let (mut best_score, total) = adjacency_score(&best, original);

    // Multiple rounds of improvement
    for _round in 0..10 {
        let mut improved = false;

        for i in 0..best.len() {
            for j in (i + 1)..best.len() {
                // Try swapping rooms i and j's positions
                let mut candidate = best.clone();
                let (xi, yi, wi, di) = (candidate[i].x, candidate[i].y, candidate[i].width, candidate[i].depth);
                let (xj, yj, wj, dj) = (candidate[j].x, candidate[j].y, candidate[j].width, candidate[j].depth);

                // Swap positions: room i gets room j's rectangle, and vice versa
                candidate[i].x = xj;
                candidate[i].y = yj;
                candidate[i].width = wj;
                candidate[i].depth = dj;
                candidate[j].x = xi;
                candidate[j].y = yi;
                candidate[j].width = wi;
                candidate[j].depth = di;

                let (score, _) = adjacency_score(&candidate, original);
                if score > best_score {
                    best = candidate;
                    best_score = score;
                    improved = true;
                }
            }
        }

        if !improved || best_score == total {
            break;
        }
    }

    best
}

/// Order rooms to maximize adjacency satisfaction in BSP.
/// Uses a BFS from the entry/most-connected room.
fn order_rooms_for_layout(rooms: &[ResolvedRoom]) -> Vec<ResolvedRoom> {
    if rooms.len() <= 1 {
        return rooms.to_vec();
    }

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

    let mut visited = vec![false; rooms.len()];
    let mut order = Vec::with_capacity(rooms.len());
    let mut queue = std::collections::VecDeque::new();

    visited[start] = true;
    queue.push_back(start);

    while let Some(idx) = queue.pop_front() {
        order.push(rooms[idx].clone());

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

    for (j, room) in rooms.iter().enumerate() {
        if !visited[j] {
            order.push(room.clone());
        }
    }

    // Move side-pinned rooms to appropriate positions
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

    if rect.width >= rect.depth {
        let split_w = rect.width * fraction;
        let left_rect = Rect { x: rect.x, y: rect.y, width: split_w, depth: rect.depth };
        let right_rect = Rect { x: rect.x + split_w, y: rect.y, width: rect.width - split_w, depth: rect.depth };
        let mut result = partition(left_rect, left_rooms);
        result.extend(partition(right_rect, right_rooms));
        result
    } else {
        let split_d = rect.depth * fraction;
        let bottom_rect = Rect { x: rect.x, y: rect.y, width: rect.width, depth: split_d };
        let top_rect = Rect { x: rect.x, y: rect.y + split_d, width: rect.width, depth: rect.depth - split_d };
        let mut result = partition(bottom_rect, left_rooms);
        result.extend(partition(top_rect, right_rooms));
        result
    }
}
