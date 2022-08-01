use crate::osm::NodeID;
use crate::IntersectionComplexity::*;
use crate::{IntersectionComplexity, OriginalRoad, RawIntersection, StreetNetwork};

/// Determines the initial complexity of all intersections. Intersections marked "Crossing" are
/// considered "unclassified" and will be updated with a guess, others will be left unchanged.
pub fn classify_intersections(streets: &mut StreetNetwork) {
    let mut changes: Vec<(NodeID, IntersectionComplexity)> = Vec::new();
    for (id, inter) in &streets.intersections {
        if let Crossing = inter.complexity {
            changes.push((
                *id,
                guess_complexity(inter, streets.roads_per_intersection(*id)),
            ));
        }
    }

    for (id, complexity) in changes.into_iter() {
        streets.intersections.get_mut(&id).unwrap().complexity = complexity;
    }
}

/// Guesses the complexity of the intersection based on the connecting roads and their lanes.
///
/// The existing complexity field is ignored, so be careful how you use the guessed value.
fn guess_complexity(_inter: &RawIntersection, roads: Vec<OriginalRoad>) -> IntersectionComplexity {
    // A terminus is characterised by a single connected road.
    if roads.len() == 1 {
        return Terminus;
    }

    // A Connection is characterised by exactly two connected roads.
    if roads.len() == 2 {
        return Connection;
    }

    // A MultiConnection is characterised by exactly two connected dividing lines (lines that
    // separate traffic in different directions) and no traffic that crosses them.
    // A Merge is characterised by an uninterrupted road (that would qualify as a Connection or
    // MultiConnection), with additional connected roads that yield.
    // TODO Combine roads into corridors and count them.

    return Crossing;
}
