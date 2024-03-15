use std::collections::{hash_map::Entry, HashMap};

use abstutil::Timer;
use geom::{HashablePt2D, PolyLine, Pt2D};
use osm2streets::{
    Direction, IntersectionControl, IntersectionID, IntersectionKind, Road, RoadID, StreetNetwork,
    TrafficInterruption,
};

use super::OsmExtract;

/// Also returns a mapping of all points to the split road. Some internal points on roads get
/// removed here, so this mapping isn't redundant.
pub fn split_up_roads(
    streets: &mut StreetNetwork,
    mut input: OsmExtract,
    timer: &mut Timer,
) -> HashMap<HashablePt2D, RoadID> {
    timer.start("splitting up roads");

    // Note all logic here is based on treating points as HashablePt2D, not as OSM node IDs. That's
    // because some members of way.pts might be synthetic, from clipping.

    // Create intersections for any points shared by at least 2 roads, and for endpoints of every
    // road.
    let mut count_per_pt = HashMap::new();
    let mut pt_to_intersection_id: HashMap<HashablePt2D, IntersectionID> = HashMap::new();
    timer.start_iter("look for common points", input.roads.len());
    for (_, pts, _) in &input.roads {
        timer.next();
        for (idx, pt) in pts.iter().enumerate() {
            let hash_pt = pt.to_hashable();
            let entry = count_per_pt.entry(hash_pt).or_insert(0);
            *entry += 1;
            let count = *entry;

            if count == 2 || idx == 0 || idx == pts.len() - 1 {
                if let Entry::Vacant(entry) = pt_to_intersection_id.entry(hash_pt) {
                    // Clipped points won't have any OSM ID.
                    let mut osm_ids = Vec::new();
                    if let Some(node_id) = input.osm_node_ids.get(&hash_pt) {
                        osm_ids.push(*node_id);
                    }

                    // TODO If there happens to be an OSM node defined RIGHT where a boundary is
                    // drawn, we might not detect it as a MapEdge?
                    let kind = if osm_ids.is_empty() {
                        IntersectionKind::MapEdge
                    } else {
                        // Assume a complicated intersection, until we determine otherwise
                        IntersectionKind::Intersection
                    };
                    let control = if osm_ids.is_empty() {
                        IntersectionControl::Uncontrolled
                    } else if input.traffic_signals.remove(&hash_pt).is_some() {
                        // This is a node; don't expect a direction
                        IntersectionControl::Signalled
                    } else {
                        // TODO default to uncontrolled, guess StopSign as a transform
                        IntersectionControl::Signed
                    };

                    let id = streets.insert_intersection(osm_ids, *pt, kind, control);
                    entry.insert(id);
                }
            }
        }
    }

    let mut pt_to_road: HashMap<HashablePt2D, RoadID> = HashMap::new();

    // Now actually split up the roads based on the intersections
    timer.start_iter("split roads", input.roads.len());
    for (osm_way_id, orig_pts, orig_tags) in &input.roads {
        timer.next();
        let mut tags = orig_tags.clone();
        let mut pts = Vec::new();
        let mut i1 = pt_to_intersection_id[&orig_pts[0].to_hashable()];

        for pt in orig_pts {
            pts.push(*pt);
            if pts.len() == 1 {
                continue;
            }
            if let Some(i2) = pt_to_intersection_id.get(&pt.to_hashable()) {
                let id = streets.next_road_id();

                // Note we populate this before simplify_linestring, so even if some points are
                // removed, we can still associate them to the road.
                for (idx, pt) in pts.iter().enumerate() {
                    if idx != 0 && idx != pts.len() - 1 {
                        pt_to_road.insert(pt.to_hashable(), id);
                    }
                }

                let untrimmed_center_line = simplify_linestring(std::mem::take(&mut pts));
                match PolyLine::new(untrimmed_center_line) {
                    Ok(pl) => {
                        streets.roads.insert(
                            id,
                            Road::new(id, vec![*osm_way_id], i1, *i2, pl, tags, &streets.config),
                        );
                        streets.intersections.get_mut(&i1).unwrap().roads.push(id);
                        streets.intersections.get_mut(&i2).unwrap().roads.push(id);
                    }
                    Err(err) => {
                        error!("Skipping {id}: {err}");
                        // There may be an orphaned intersection left around; a later
                        // transformation should clean it up
                    }
                }

                // Start a new road
                tags = orig_tags.clone();
                i1 = *i2;
                pts.push(*pt);
            }
        }
        assert!(pts.len() == 1);
    }

    // Resolve simple turn restrictions (via a node)
    let mut restrictions = Vec::new();
    timer.start_iter(
        "resolve simple turn restrictions",
        input.simple_turn_restrictions.len(),
    );
    for (restriction, from_osm, via_osm, to_osm) in input.simple_turn_restrictions {
        timer.next();
        // A via node might not be an intersection
        let via_id = if let Some(i) = streets
            .intersections
            .values()
            .find(|i| i.osm_ids.contains(&via_osm))
        {
            i.id
        } else {
            continue;
        };
        if !streets.intersections.contains_key(&via_id) {
            continue;
        }
        let roads = streets.roads_per_intersection(via_id);
        // If some of the roads are missing, they were likely filtered out -- usually service
        // roads.
        if let (Some(from), Some(to)) = (
            roads.iter().find(|r| r.from_osm_way(from_osm)),
            roads.iter().find(|r| r.from_osm_way(to_osm)),
        ) {
            restrictions.push((from.id, restriction, to.id));
        }
    }
    for (from, rt, to) in restrictions {
        streets
            .roads
            .get_mut(&from)
            .unwrap()
            .turn_restrictions
            .push((rt, to));
    }

    // Resolve complicated turn restrictions (via a way). TODO Only handle via ways immediately
    // connected to both roads, for now
    let mut complicated_restrictions = Vec::new();
    timer.start_iter(
        "resolve complicated turn restrictions",
        input.complicated_turn_restrictions.len(),
    );
    for (rel_osm, from_osm, via_osm, to_osm) in input.complicated_turn_restrictions {
        timer.next();
        let via_candidates: Vec<&Road> = streets
            .roads
            .values()
            .filter(|r| r.from_osm_way(via_osm))
            .collect();
        if via_candidates.len() != 1 {
            warn!(
                "Couldn't resolve turn restriction from way {from_osm} to way {to_osm} via way {via_osm}. Candidate roads for via: {:?}. See {rel_osm}", via_candidates
            );
            continue;
        }
        let via = via_candidates[0];

        let maybe_from = streets
            .roads_per_intersection(via.src_i)
            .into_iter()
            .chain(streets.roads_per_intersection(via.dst_i).into_iter())
            .find(|r| r.from_osm_way(from_osm));
        let maybe_to = streets
            .roads_per_intersection(via.src_i)
            .into_iter()
            .chain(streets.roads_per_intersection(via.dst_i).into_iter())
            .find(|r| r.from_osm_way(to_osm));
        match (maybe_from, maybe_to) {
            (Some(from), Some(to)) => {
                complicated_restrictions.push((from.id, via.id, to.id));
            }
            _ => {
                warn!(
                    "Couldn't resolve turn restriction from {from_osm} to {to_osm} via {:?}",
                    via
                );
            }
        }
    }
    for (from, via, to) in complicated_restrictions {
        streets
            .roads
            .get_mut(&from)
            .unwrap()
            .complicated_turn_restrictions
            .push((via, to));
    }

    timer.start_iter(
        "match traffic signals to intersections",
        input.traffic_signals.len(),
    );
    // Handle traffic signals tagged on incoming ways and not at intersections
    // (https://wiki.openstreetmap.org/wiki/Tag:highway=traffic%20signals?uselang=en#Tag_all_incoming_ways).
    for (pt, dir) in input.traffic_signals {
        timer.next();
        if let Some(r) = pt_to_road.get(&pt) {
            // The road might've crossed the boundary and been clipped
            if let Some(road) = streets.roads.get_mut(r) {
                // On a one-way road, specifying direction is redundant, so infer from there too
                if let Some(dir) = dir.or_else(|| road.oneway_for_driving()) {
                    // Update the intersection control type
                    let i = if dir == Direction::Forward {
                        road.dst_i
                    } else {
                        road.src_i
                    };
                    let i = streets.intersections.get_mut(&i).unwrap();
                    // TODO Maybe we should do this later, as a consequence of TrafficInterruption
                    // on incoming roads?
                    if !i.is_map_edge() {
                        i.control = IntersectionControl::Signalled;
                    }

                    // Specify the explicit vehicle stop line
                    if let Some((dist, _)) = road.reference_line.dist_along_of_point(pt.to_pt2d()) {
                        let stop_line = if dir == Direction::Forward {
                            &mut road.stop_line_end
                        } else {
                            &mut road.stop_line_start
                        };
                        stop_line.vehicle_distance = Some(dist);
                        stop_line.interruption = TrafficInterruption::Signal;
                    }
                    // TODO If dist_along_of_point fails, it's because we smoothed the line. This
                    // is a great reason to instead just find the closest point on the line and
                    // then the distance.
                }
                // TODO What should we do more generally with traffic signals on ways that don't
                // specify a direction?
            }
        }
    }

    // Do the same for cycleway ASLs
    timer.start_iter("match cycleway stop lines", input.cycleway_stop_lines.len());
    for (pt, dir) in input.cycleway_stop_lines {
        timer.next();
        if let Some(road) = pt_to_road.get(&pt).and_then(|r| streets.roads.get_mut(r)) {
            if let Some(dir) = dir {
                if let Some((dist, _)) = road.reference_line.dist_along_of_point(pt.to_pt2d()) {
                    let stop_line = if dir == Direction::Forward {
                        &mut road.stop_line_end
                    } else {
                        &mut road.stop_line_start
                    };
                    stop_line.bike_distance = Some(dist);

                    // Inherit the interruption type from the intersection
                    let i = if dir == Direction::Forward {
                        road.dst_i
                    } else {
                        road.src_i
                    };
                    if streets.intersections[&i].control == IntersectionControl::Signalled {
                        stop_line.interruption = TrafficInterruption::Signal;
                    }
                }
            }
        }
    }

    timer.start_iter(
        "match signalized crossings",
        input.signalized_crossings.len(),
    );
    for pt in input.signalized_crossings {
        timer.next();
        if let Some(road) = pt_to_road.get(&pt).and_then(|r| streets.roads.get_mut(r)) {
            if let Some((dist, _)) = road.reference_line.dist_along_of_point(pt.to_pt2d()) {
                // We don't know the direction. Arbitrarily snap to the start or end if it's within
                // 30% of the length. If it's in the middle 40%, it might be a mid-block crossing?
                let pct = dist / road.reference_line.length();
                if pct < 0.3 {
                    road.stop_line_start.vehicle_distance = Some(dist);
                    road.stop_line_start.interruption = TrafficInterruption::Signal;
                } else if pct > 0.7 {
                    road.stop_line_end.vehicle_distance = Some(dist);
                    road.stop_line_end.interruption = TrafficInterruption::Signal;
                }
            }
        }
    }

    let intersection_ids: Vec<_> = streets.intersections.keys().cloned().collect();
    timer.start_iter(
        "calculate intersection geometry and movements",
        intersection_ids.len(),
    );
    for i in intersection_ids {
        timer.next();
        streets.sort_roads(i);
        streets.update_i(i);
    }

    timer.stop("splitting up roads");
    pt_to_road
}

// TODO Consider doing this in PolyLine::new always. Also in extend() -- it attempts to dedupe
// angles.
fn simplify_linestring(pts: Vec<Pt2D>) -> Vec<Pt2D> {
    // Reduce the number of points along curves. They're wasteful, and when they're too close
    // together, actually break PolyLine shifting:
    // https://github.com/a-b-street/abstreet/issues/833
    //
    // The epsilon is in units of meters; points closer than this will get simplified. 0.1 is too
    // loose -- a curve with too many points was still broken, but 1.0 was too aggressive -- curves
    // got noticeably flattened. At 0.5, some intersetion polygons get a bit worse, but only in
    // places where they were already pretty broken.
    let epsilon = 0.5;
    Pt2D::simplify_rdp(pts, epsilon)
}
