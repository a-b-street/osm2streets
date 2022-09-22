use crate::osm::NodeID;
use crate::IntersectionComplexity::*;
use crate::{
    ConflictType, IntersectionComplexity, OriginalRoad, RestrictionType, Road, StreetNetwork,
};
use std::cmp::{max, min};
use std::collections::BTreeMap;

/// Determines the initial complexity of all intersections. Intersections marked "Crossing" are
/// considered "unclassified" and will be updated with a guess, others will be left unchanged.
pub fn classify_intersections(streets: &mut StreetNetwork) {
    let mut changes: Vec<(NodeID, (IntersectionComplexity, ConflictType))> = Vec::new();
    for (id, inter) in &streets.intersections {
        if let Crossing = inter.complexity {
            changes.push((*id, guess_complexity(streets, id)));
        }
    }

    for (id, complexity) in changes.into_iter() {
        let intersection = streets.intersections.get_mut(&id).unwrap();
        intersection.complexity = complexity.0;
        intersection.conflict_level = complexity.1;
    }
}

/// Guesses the complexity of the intersection based on the connecting roads and their lanes.
///
/// The existing complexity field is ignored, so be careful how you use the guessed value.
fn guess_complexity(
    streets: &StreetNetwork,
    intersection_id: &NodeID,
) -> (IntersectionComplexity, ConflictType) {
    use ConflictType::*;
    let road_ids = streets.roads_per_intersection(*intersection_id);
    let roads: Vec<&Road> = road_ids
        .iter()
        .map(|id| streets.roads.get(id).unwrap())
        .collect();

    // A terminus is characterised by a single connected road.
    if road_ids.len() == 1 {
        return (Terminus, Uncontested);
    }

    // A Connection is characterised by exactly two connected roads.
    if road_ids.len() == 2 {
        return (Connection, Uncontested);
    }

    // Calculate all the possible movements, except (U-turns).
    //FIXME assert!(roads is sorted clockwise), which it isn't
    let mut connections = Vec::new();
    // Consider turns pairs of roads, from s to d, using their position as index, so we can them later.
    for s in 0..road_ids.len() {
        for d in 0..road_ids.len() {
            if s == d {
                continue;
            }
            // FIXME check if we can travel into the intersection on s and out of the intersection on d
            if turn_is_allowed(roads.get(s).unwrap(), road_ids.get(d).unwrap()) {
                connections.push((s, d));
            }
        }
    }

    // Calculate all the collisions.
    let mut conflicts = BTreeMap::new();
    let mut worst_conflict = Uncontested;
    // Compare every pair of connections. Use the order of the roads around the intersection to
    // detect if they diverge, merge, or cross.
    // assert!(connections is sorted) so small_con large_con makes sense.
    let mut each_con = connections.iter();
    while let Some(small_con) = each_con.next() {
        for large_con in each_con.clone() {
            let conflict = calc_conflict(small_con, large_con);
            worst_conflict = max(worst_conflict, conflict);
            conflicts.insert((small_con, large_con), conflict);
        }
    }

    match worst_conflict {
        Cross => (Crossing, Cross),
        c => (MultiConnection, c),
    }
}

fn turn_is_allowed(src: &Road, dst_id: &OriginalRoad) -> bool {
    let mut has_exclusive_allows = false;
    for (t, other) in &src.turn_restrictions {
        match t {
            RestrictionType::BanTurns => {
                if other == dst_id {
                    return false;
                }
            }
            RestrictionType::OnlyAllowTurns => {
                if other == dst_id {
                    return true;
                }
                has_exclusive_allows = true;
            }
        }
    }
    !has_exclusive_allows
}

fn calc_conflict(a: &(usize, usize), b: &(usize, usize)) -> ConflictType {
    use ConflictType::*;
    if a.0 == b.0 {
        return Diverge;
    }
    if a.1 == b.1 {
        return Merge;
    }
    if a.0 == b.1 || a.1 == b.0 {
        // TODO depends on driving side and arm order?
        // It would be clear if a and b were flows instead of roads.
    }

    // The intersection has a boundary that we have labelled 0 to n in clockwise order (from an
    // arbitrary point), like a string layed in a circle. If we represent `a` as an arc from one point
    // on the string to another, then there is a section of the string that connects the two points
    // and two ends. A second arc, `b`, crosses `a` if and only if `b` has one end between the points
    // and one end outside.
    //     ______
    //    /  |   \
    //   |   |a   n
    //   |   |    0
    //    \__|___/
    if is_between(a.0, b) ^ is_between(a.1, b) {
        return Cross;
    }
    return Uncontested;
}

fn is_between(num: usize, range: &(usize, usize)) -> bool {
    let bot = min(range.0, range.1);
    let top = max(range.0, range.1);
    return bot < num && num < top;
}
