use anyhow::Result;

use crate::{IntersectionID, Placement, Road, StreetNetwork};

/// Collapse degenerate intersections:
/// - between two cycleways
/// - when the lane specs, name, and layer match
/// - when there's no crossing
pub fn collapse(streets: &mut StreetNetwork) {
    let mut merge: Vec<IntersectionID> = Vec::new();
    for intersection in streets.intersections.values() {
        if intersection.crossing.is_some() {
            continue;
        }
        let roads = streets.roads_per_intersection(intersection.id);
        if roads.len() != 2 {
            continue;
        }
        match should_collapse(roads[0], roads[1]) {
            Ok(()) => {
                merge.push(intersection.id);
            }
            Err(err) => {
                warn!(
                    "Not collapsing degenerate intersection {}: {}",
                    intersection.id, err
                );
            }
        }
    }

    for i in merge {
        streets.collapse_intersection(i);
    }

    // It's possible we need to do this in a fixed-point until there are no changes, but meh.
    // Results look good so far.
}

fn should_collapse(road1: &Road, road2: &Road) -> Result<()> {
    // Don't attempt to merge roads with these.
    if !road1.turn_restrictions.is_empty() || !road1.complicated_turn_restrictions.is_empty() {
        bail!("one road has turn restrictions");
    }
    if !road2.turn_restrictions.is_empty() || !road2.complicated_turn_restrictions.is_empty() {
        bail!("one road has turn restrictions");
    }

    // Avoid two one-ways that point at each other. https://www.openstreetmap.org/node/440979339 is
    // a bizarre example. These are actually blackholed, some problem with service roads.
    if road1.oneway_for_driving().is_some()
        && road2.oneway_for_driving().is_some()
        && road1.dst_i == road2.dst_i
    {
        bail!("oneway roads point at each other");
    }

    if road1.lane_specs_ltr != road2.lane_specs_ltr {
        bail!("lane specs don't match");
    }

    if road1.name != road2.name {
        bail!("names don't match");
    }

    if road1.highway_type != road2.highway_type {
        bail!("highway_type don't match");
    }

    if road1.layer != road2.layer {
        bail!("layers don't match");
    }

    match (
        road1.reference_line_placement,
        road2.reference_line_placement,
    ) {
        (Placement::Consistent(p1), Placement::Consistent(p2)) => {
            if p1 != p2 {
                bail!("placements don't match")
            }
        }
        _ => bail!("one of the placements isn't consistent"),
    }

    Ok(())
}
