use crate::osm::NodeID;
use crate::types::IndexedMovement;
use crate::Direction;
use crate::IntersectionComplexity::*;
use crate::{
    ConflictType, DrivingSide, IntersectionComplexity, OriginalRoad, RestrictionType, Road,
    StreetNetwork,
};
use std::cmp::{max, min};
use std::collections::BTreeMap;

/// Determines the initial complexity of all intersections. Intersections marked "Crossing" are
/// considered "unclassified" and will be updated with a guess, others will be left unchanged.
pub fn classify_intersections(streets: &mut StreetNetwork) {
    let mut changes: Vec<_> = Vec::new();
    for (id, inter) in &streets.intersections {
        if let Crossing = inter.complexity {
            changes.push((*id, guess_complexity(streets, id)));
        }
    }

    for (id, (complexity, conflict_level, movements)) in changes {
        let intersection = streets.intersections.get_mut(&id).unwrap();
        intersection.complexity = complexity;
        intersection.conflict_level = conflict_level;
        intersection.movements = movements;
    }
}

/// Guesses the complexity of the intersection based on the connecting roads and their lanes.
///
/// The existing complexity field is ignored, so be careful how you use the guessed value.
fn guess_complexity(
    streets: &StreetNetwork,
    intersection_id: &NodeID,
) -> (IntersectionComplexity, ConflictType, Vec<IndexedMovement>) {
    use ConflictType::*;
    let roads: Vec<_> = streets
        .roads_per_intersection(*intersection_id)
        .iter()
        .map(|id| streets.roads.get_key_value(id).unwrap())
        .filter(|(_id, road)| road.is_driveable())
        .collect();

    // A terminus is characterised by a single connected road.
    if roads.len() == 1 {
        return (Terminus, Uncontested, Vec::new());
    }

    // A Connection is characterised by exactly two connected roads.
    if roads.len() == 2 {
        let mut movements = Vec::new();
        if can_drive_out_of(roads[0], *intersection_id)
            && can_drive_into(roads[1], *intersection_id)
        {
            movements.push((0, 1));
        }
        if can_drive_out_of(roads[1], *intersection_id)
            && can_drive_into(roads[0], *intersection_id)
        {
            movements.push((1, 0));
        }
        return (Connection, Uncontested, movements);
    }

    // Calculate all the possible movements, (except U-turns, for now).
    let mut connections = Vec::new();
    // Consider all pairs of roads, from s to d. Identify them using their index in the list - which
    // is sorted in clockwise order - so that we can compare their position later.
    for s in 0..roads.len() {
        for d in 0..roads.len() {
            if s == d {
                continue; // Ignore U-turns.
            }

            // Calculate if it is possible to emerge from s into the intersection.
            let src_road = roads[s];
            if !can_drive_out_of(src_road, *intersection_id) {
                continue;
            }

            // Calculate if it is possible to leave the intersection into d.
            let dst_road = roads[d];
            if !can_drive_into(dst_road, *intersection_id) {
                continue;
            }

            // Check for any turn restrictions.
            if turn_is_allowed(src_road, dst_road) {
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
            let conflict = calc_conflict(small_con, large_con, streets.config.driving_side);
            worst_conflict = max(worst_conflict, conflict);
            conflicts.insert((small_con, large_con), conflict);
        }
    }

    match worst_conflict {
        Cross => (Crossing, Cross, connections),
        c => (MultiConnection, c, connections),
    }
}

fn can_drive_out_of(id_road: (&OriginalRoad, &Road), which_end: NodeID) -> bool {
    if let Some(driving_dir) = id_road.1.oneway_for_driving() {
        let required_dir = if id_road.0.i2 == which_end {
            Direction::Fwd
        } else {
            Direction::Back
        };
        return driving_dir == required_dir;
    }
    return true;
}

fn can_drive_into(id_road: (&OriginalRoad, &Road), which_end: NodeID) -> bool {
    if let Some(driving_dir) = id_road.1.oneway_for_driving() {
        let required_dir = if id_road.0.i1 == which_end {
            Direction::Fwd
        } else {
            Direction::Back
        };
        return driving_dir == required_dir;
    }
    return true;
}

fn turn_is_allowed(src: (&OriginalRoad, &Road), dst: (&OriginalRoad, &Road)) -> bool {
    let mut has_exclusive_allows = false;
    for (t, other) in src.1.turn_restrictions.iter() {
        match t {
            RestrictionType::BanTurns => {
                if *other == *dst.0 {
                    return false;
                }
            }
            RestrictionType::OnlyAllowTurns => {
                if *other == *dst.0 {
                    return true;
                }
                has_exclusive_allows = true;
            }
        }
    }
    !has_exclusive_allows
}

fn calc_conflict(a: &(usize, usize), b: &(usize, usize), side: DrivingSide) -> ConflictType {
    use ConflictType::*;

    // If the traffic starts of ends at the same place in the same direction...
    if a.0 == b.0 && a.1 == b.1 {
        return Uncontested;
    }
    if a.0 == b.0 {
        return Diverge;
    }
    if a.1 == b.1 {
        return Merge;
    }

    // The intersection has a boundary that we have labelled 0 to n-1 in clockwise order (from an
    // arbitrary point), like a string laying in a circle. If we represent `a` as an arc from one
    // point on the string to another, then there is a section of the string between the two points,
    // connecting them the two points and two ends of string "on the outside". A second arc, `b`,
    // crosses `a` if and only if `b` has one end between the points and one end outside.
    //     ______
    //    /  |   \
    //   |   |a   n
    //   |   |    0
    //    \__|___/

    // What if the traffic meets going in opposite directions?
    // It depends on where the traffic came from, and which side we're driving on.

    // Below: If a movement going in the other direction, `b`, joins the indicated LHT movement `a`
    // (at either end), it will join the road on the dotted side. Whether the other end of `b` is
    // between the endpoints of `a` or not corresponds to the crossing of the road.
    // Therefore, if `a` is drawn pointing upwards from low .0 to high .1,
    // then LHT would be crossed by movements joining from the "inside".
    //     ______          ______
    //    /  ^:  \        /  :|  \
    //   |  a|:   n      |   :|   n
    //   |   |:   0      |   :|a  0
    //    \__|:__/        \__:V__/

    // This equation (hopefully) works. Once it does, just trust it:
    // TODO unit test these three equations.
    let is_driving_side_between = (side == DrivingSide::Left) ^ (a.0 < a.1); // `==` or `^`?

    if a.0 == b.1 {
        return if is_driving_side_between ^ is_between(b.0, a) {
            Cross
        } else {
            Uncontested
        };
    }
    if a.1 == b.0 {
        return if is_driving_side_between ^ is_between(b.1, a) {
            Cross
        } else {
            Uncontested
        };
    }

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
