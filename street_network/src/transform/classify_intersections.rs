use crate::osm::NodeID;
use crate::IntersectionComplexity::*;
use crate::{Direction, IntersectionComplexity, StreetNetwork};

/// Determines the initial complexity of all intersections. Intersections marked "Crossing" are
/// considered "unclassified" and will be updated with a guess, others will be left unchanged.
pub fn classify_intersections(streets: &mut StreetNetwork) {
    let mut changes: Vec<(NodeID, IntersectionComplexity)> = Vec::new();
    for (id, inter) in &streets.intersections {
        if let Crossing = inter.complexity {
            changes.push((*id, guess_complexity(streets, id)));
        }
    }

    for (id, complexity) in changes.into_iter() {
        streets.intersections.get_mut(&id).unwrap().complexity = complexity;
    }
}

/// Guesses the complexity of the intersection based on the connecting roads and their lanes.
///
/// The existing complexity field is ignored, so be careful how you use the guessed value.
fn guess_complexity(streets: &StreetNetwork, intersection_id: &NodeID) -> IntersectionComplexity {
    let roads = streets.roads_per_intersection(*intersection_id);

    // A terminus is characterised by a single connected road.
    if roads.len() == 1 {
        return Terminus;
    }

    // A Connection is characterised by exactly two connected roads.
    if roads.len() == 2 {
        return Connection;
    }

    // A MultiConnection is characterised by exactly one dividing line traveling through it (the
    // line that separates traffic in different directions) and no traffic that crosses it.
    if roads.len() == 3 {
        let mut _num_roads_in = 0;
        let mut _num_roads_out = 0;
        let mut num_roads_inout = 0;
        for road_id in roads {
            let is_outward = road_id.i1 == *intersection_id;
            let road = streets.roads.get(&road_id).unwrap();
            match road.oneway_for_driving() {
                Some(Direction::Fwd) => {
                    if is_outward {
                        _num_roads_out += 1
                    } else {
                        _num_roads_in += 1
                    }
                }

                Some(Direction::Back) => {
                    if is_outward {
                        _num_roads_in += 1
                    } else {
                        _num_roads_out += 1
                    }
                }
                None => num_roads_inout += 1,
            }
        }

        if num_roads_inout == 0 {
            // The simplest MultiConnect
            return MultiConnection;
        }

        // detect the simple case of a single road splitting
        if num_roads_inout == 1 {
            // TODO Determine if it is possible to turn from the 'in' road to the 'out' road.
            // If not, then we have a dual carriageway split.
            if false {
                return MultiConnection;
            }
        }
    }

    // A Merge is characterised by an uninterrupted road (that would qualify as a Connection or
    // MultiConnection), with additional connected roads that yield.
    // TODO Combine roads into corridors and count them.

    return Crossing;
}
