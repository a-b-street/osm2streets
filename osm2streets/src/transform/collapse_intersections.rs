use std::collections::BTreeSet;

use anyhow::Result;

use geom::{Distance, PolyLine, Pt2D};

use crate::{osm, ControlType, IntersectionID, Road, RoadID, StreetNetwork};

/// Collapse degenerate intersections:
/// - between two cycleways
/// - when the lane specs match and only "unimportant" OSM tags differ
pub fn collapse(streets: &mut StreetNetwork) {
    let mut merge: Vec<IntersectionID> = Vec::new();
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

pub fn collapse_intersection(streets: &mut StreetNetwork, i: IntersectionID) {
    let roads = streets.intersections[&i].roads.clone();
    assert_eq!(roads.len(), 2);
    // Arbitrarily keep the first and delete the second
    let keep_r = roads[0];
    let destroy_r = roads[1];
    assert_ne!(keep_r, destroy_r);

    // Skip loops; they break. Easiest way to detect is see how many total vertices we've got.
    {
        let mut endpts = BTreeSet::new();
        endpts.extend(streets.roads[&keep_r].endpoints());
        endpts.extend(streets.roads[&destroy_r].endpoints());
        if endpts.len() != 3 {
            info!("Not collapsing degenerate {i}, because it's a loop");
            return;
        }
    }

    // We could be more careful merging percent_incline and osm_tags, but in practice, it doesn't
    // matter for the short segments we're merging.
    let mut keep_road = streets.remove_road(keep_r);
    let destroy_road = streets.remove_road(destroy_r);
    streets.intersections.remove(&i).unwrap();

    // Remember the merge
    keep_road.osm_ids.extend(destroy_road.osm_ids);

    // There are 4 cases, easy to understand on paper. Preserve the original direction of keep_r.
    // Work with points, not PolyLine::extend. We want to RDP simplify before finalizing.
    let mut new_pts;
    let (new_src_i, new_dst_i) = if keep_road.dst_i == destroy_road.src_i {
        new_pts = keep_road.untrimmed_center_line.clone().into_points();
        new_pts.extend(destroy_road.untrimmed_center_line.into_points());
        (keep_road.src_i, destroy_road.dst_i)
    } else if keep_road.dst_i == destroy_road.dst_i {
        new_pts = keep_road.untrimmed_center_line.clone().into_points();
        new_pts.extend(destroy_road.untrimmed_center_line.reversed().into_points());
        (keep_road.src_i, destroy_road.src_i)
    } else if keep_road.src_i == destroy_road.src_i {
        new_pts = destroy_road.untrimmed_center_line.into_points();
        new_pts.reverse();
        new_pts.extend(keep_road.untrimmed_center_line.clone().into_points());
        (destroy_road.dst_i, keep_road.dst_i)
    } else if keep_road.src_i == destroy_road.dst_i {
        new_pts = destroy_road.untrimmed_center_line.into_points();
        new_pts.extend(keep_road.untrimmed_center_line.clone().into_points());
        (destroy_road.src_i, keep_road.dst_i)
    } else {
        unreachable!()
    };
    // Sanity check
    assert!(i != new_src_i && i != new_dst_i);
    // Simplify curves and dedupe points. The epsilon was tuned for only one location that was
    // breaking
    let epsilon = 1.0;
    keep_road.untrimmed_center_line = PolyLine::must_new(Pt2D::simplify_rdp(new_pts, epsilon));

    // Keep the same ID, but fix the endpoints
    keep_road.src_i = new_src_i;
    keep_road.dst_i = new_dst_i;
    streets.insert_road(keep_road);

    // We may need to fix up turn restrictions. destroy_r becomes keep_r.
    let rewrite = |x: &mut RoadID| {
        if *x == destroy_r {
            *x = keep_r;
        }
    };
    for road in streets.roads.values_mut() {
        for (_, id) in &mut road.turn_restrictions {
            rewrite(id);
        }

        for (id1, id2) in &mut road.complicated_turn_restrictions {
            rewrite(id1);
            rewrite(id2);
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
    for i in streets.intersections.values() {
        let roads = streets.roads_per_intersection(i.id);
        if roads.len() != 1 || i.control == ControlType::Border {
            continue;
        }
        let road = &roads[0];
        if road.untrimmed_length() < SHORT_THRESHOLD
            && (road.is_cycleway() || road.osm_tags.is(osm::HIGHWAY, "service"))
        {
            remove_roads.insert(roads[0].id);
            remove_intersections.insert(i.id);
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
