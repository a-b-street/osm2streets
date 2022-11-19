use std::collections::BTreeSet;

use anyhow::Result;

use geom::{Distance, PolyLine, Pt2D};

use crate::osm::NodeID;
use crate::{osm, ControlType, OriginalRoad, Road, StreetNetwork};

/// Collapse degenerate intersections:
/// - between two cycleways
/// - when the lane specs match and only "unimportant" OSM tags differ
pub fn collapse(streets: &mut StreetNetwork) {
    let mut merge: Vec<NodeID> = Vec::new();
    for id in streets.intersections.keys() {
        let roads = streets.roads_per_intersection(*id);
        if roads.len() != 2 {
            continue;
        }
        match should_collapse(roads[0], roads[1]) {
            Ok(()) => {
                merge.push(*id);
            }
            Err(err) => {
                warn!("Not collapsing degenerate intersection {}: {}", id, err);
            }
        }
    }

    for i in merge {
        collapse_intersection(streets, i);
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

    if road1.get_zorder() != road2.get_zorder() {
        bail!("zorders don't match");
    }

    if road1.is_cycleway() && road2.is_cycleway() {
        return Ok(());
    }

    // Check what OSM tags differ. Explicitly allow some keys. Note that lanes tagging doesn't
    // actually matter, because we check that LaneSpecs match. Nor do things indicating a zorder
    // indirectly, like bridge/tunnel.
    // TODO I get the feeling I'll end up swapping this to explicitly list tags that SHOULD block
    // merging.
    for (k, v1, v2) in road1.osm_tags.diff(&road2.osm_tags) {
        if [
            osm::OSM_WAY_ID,
            osm::PARKING_BOTH,
            osm::PARKING_LEFT,
            osm::PARKING_RIGHT,
            "bicycle",
            "bridge",
            "covered",
            "cycleway",
            "cycleway:both",
            "destination",
            "lanes",
            "lanes:backward",
            "lanes:forward",
            "lit",
            "maxheight",
            "maxspeed:advisory",
            "maxweight",
            "note",
            "old_name",
            "short_name",
            "shoulder",
            "sidewalk",
            "surface",
            "tunnel",
            "wikidata",
            "wikimedia_commons",
            "wikipedia",
        ]
        .contains(&k.as_ref())
        {
            continue;
        }

        // Don't worry about ENDPT_FWD and ENDPT_BACK not matching if there are no turn lanes
        // tagged.
        // TODO We could get fancier and copy values over. We'd have to sometimes flip the
        // direction.
        if k == osm::ENDPT_FWD
            && !road1.osm_tags.contains_key("turn:lanes")
            && !road1.osm_tags.contains_key("turn:lanes:forward")
            && !road2.osm_tags.contains_key("turn:lanes")
            && !road2.osm_tags.contains_key("turn:lanes:forward")
        {
            continue;
        }
        if k == osm::ENDPT_BACK
            && !road1.osm_tags.contains_key("turn:lanes:backward")
            && !road2.osm_tags.contains_key("turn:lanes:backward")
        {
            continue;
        }

        bail!("{} = \"{}\" vs \"{}\"", k, v1, v2);
    }

    Ok(())
}

pub fn collapse_intersection(streets: &mut StreetNetwork, i: NodeID) {
    let roads = streets.intersections[&i].roads.clone();
    assert_eq!(roads.len(), 2);
    let mut r1 = roads[0];
    let mut r2 = roads[1];
    assert_ne!(r1, r2);

    // We'll keep r1's way ID, so it's a little more convenient for debugging to guarantee r1 is
    // the longer piece.
    if streets.roads[&r1].untrimmed_length() < streets.roads[&r2].untrimmed_length() {
        std::mem::swap(&mut r1, &mut r2);
    }

    // Skip loops; they break. Easiest way to detect is see how many total vertices we've got.
    {
        let mut endpts = BTreeSet::new();
        endpts.extend(streets.roads[&r1].endpoints());
        endpts.extend(streets.roads[&r2].endpoints());
        if endpts.len() != 3 {
            info!("Not collapsing degenerate {i}, because it's a loop");
            return;
        }
    }

    // We could be more careful merging percent_incline and osm_tags, but in practice, it doesn't
    // matter for the short segments we're merging.
    let mut new_road = streets.remove_road(r1);
    let road2 = streets.remove_road(r2);
    streets.intersections.remove(&i).unwrap();

    // There are 4 cases, easy to understand on paper. Preserve the original direction of r1
    // Work with points, not PolyLine::extend. We want to RDP simplify before finalizing.
    let mut new_pts;
    let (new_i1, new_i2) = if new_road.dst_i == road2.src_i {
        new_pts = new_road.untrimmed_center_line.clone().into_points();
        new_pts.extend(road2.untrimmed_center_line.into_points());
        (new_road.src_i, road2.dst_i)
    } else if new_road.dst_i == road2.dst_i {
        new_pts = new_road.untrimmed_center_line.clone().into_points();
        new_pts.extend(road2.untrimmed_center_line.reversed().into_points());
        (new_road.src_i, road2.src_i)
    } else if new_road.src_i == road2.src_i {
        new_pts = road2.untrimmed_center_line.into_points();
        new_pts.reverse();
        new_pts.extend(new_road.untrimmed_center_line.clone().into_points());
        (road2.dst_i, new_road.dst_i)
    } else if new_road.src_i == road2.dst_i {
        new_pts = road2.untrimmed_center_line.into_points();
        new_pts.extend(new_road.untrimmed_center_line.clone().into_points());
        (road2.src_i, new_road.dst_i)
    } else {
        unreachable!()
    };
    // Sanity check
    assert!(i != new_i1 && i != new_i2);
    // Simplify curves and dedupe points. The epsilon was tuned for only one location that was
    // breaking
    let epsilon = 1.0;
    new_road.untrimmed_center_line = PolyLine::must_new(Pt2D::simplify_rdp(new_pts, epsilon));

    let new_r1 = OriginalRoad {
        osm_way_id: r1.osm_way_id,
        i1: new_i1,
        i2: new_i2,
    };
    new_road.id = new_r1;
    new_road.src_i = new_i1;
    new_road.dst_i = new_i2;
    streets.insert_road(new_road);

    // We may need to fix up turn restrictions. r1 and r2 both become new_r1.
    let rewrite = |x: &OriginalRoad| *x == r1 || *x == r2;
    for road in streets.roads.values_mut() {
        for (_, id) in &mut road.turn_restrictions {
            if rewrite(id) {
                *id = new_r1;
            }
        }

        for (id1, id2) in &mut road.complicated_turn_restrictions {
            if rewrite(id1) {
                *id1 = new_r1;
            }
            if rewrite(id2) {
                *id2 = new_r1;
            }
        }
    }
}

const SHORT_THRESHOLD: Distance = Distance::const_meters(30.0);

/// Some cycleways intersect footways with detailed curb mapping. The current rules for figuring
/// out which walking paths also allow bikes are imperfect, so we wind up with short dead-end
/// "stubs." Trim those.
///
/// Also do the same thing for extremely short dead-end service roads.
pub fn trim_deadends(streets: &mut StreetNetwork) {
    let mut remove_roads = BTreeSet::new();
    let mut remove_intersections = BTreeSet::new();
    for (id, i) in &streets.intersections {
        let roads = streets.roads_per_intersection(*id);
        if roads.len() != 1 || i.control == ControlType::Border {
            continue;
        }
        let road = &roads[0];
        if road.untrimmed_length() < SHORT_THRESHOLD
            && (road.is_cycleway() || road.osm_tags.is(osm::HIGHWAY, "service"))
        {
            remove_roads.insert(roads[0].id);
            remove_intersections.insert(*id);
        }
    }

    for r in remove_roads {
        streets.remove_road(r);
    }
    for i in remove_intersections {
        streets.remove_intersection(i);
    }

    // It's possible we need to do this in a fixed-point until there are no changes, but meh.
    // Results look good so far.
}
