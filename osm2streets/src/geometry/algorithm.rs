use std::collections::BTreeMap;

use anyhow::Result;

use geom::{Distance, InfiniteLine, PolyLine, Polygon, Pt2D, Ring};

use super::{close_off_polygon, Results, RoadLine};
use crate::road::RoadEdge;
use crate::{InputRoad, IntersectionID, RoadID};

/// Trims back all roads connected to the intersection, and generates a polygon for the
/// intersection. The trimmed roads should meet this polygon at a right angle. The input is assumed
/// to be untrimmed (based on the original reference geometry), and the roads must be ordered clockwise.
pub fn intersection_polygon(
    intersection_id: IntersectionID,
    input_roads: Vec<InputRoad>,
    trim_roads_for_merging: &BTreeMap<(RoadID, bool), Pt2D>,
) -> Result<Results> {
    // TODO Possibly take this as input in the first place
    let mut roads: BTreeMap<RoadID, InputRoad> = BTreeMap::new();
    let mut sorted_roads: Vec<RoadID> = Vec::new();
    for r in input_roads {
        sorted_roads.push(r.id);
        roads.insert(r.id, r);
    }

    if roads.is_empty() {
        bail!("{intersection_id} has no roads");
    }

    // First pre-trim roads if it's a consolidated intersection.
    for road in roads.values_mut() {
        if let Some(endpt) = trim_roads_for_merging.get(&(road.id, road.src_i == intersection_id)) {
            if road.src_i == intersection_id {
                match road.center_line.safe_get_slice_starting_at(*endpt) {
                    Some(pl) => {
                        road.center_line = pl;
                    }
                    None => {
                        error!("{}'s trimmed points start past the endpt {endpt}", road.id);
                        // Just skip. See https://github.com/a-b-street/abstreet/issues/654 for a
                        // start to diagnose. Repro at https://www.openstreetmap.org/node/53211693.
                    }
                }
            } else {
                assert_eq!(road.dst_i, intersection_id);
                match road.center_line.safe_get_slice_ending_at(*endpt) {
                    Some(pl) => {
                        road.center_line = pl;
                    }
                    None => {
                        error!("{}'s trimmed points end before the endpt {endpt}", road.id);
                    }
                }
            }
        }
    }

    // TODO Can we get rid of RoadLine?
    let mut road_lines = Vec::new();
    for id in sorted_roads.clone() {
        let road = &roads[&id];
        let center_pl = road.center_line_pointed_at(intersection_id);
        road_lines.push(RoadLine {
            id,
            fwd_pl: center_pl.shift_right(road.half_width())?,
            back_pl: center_pl.shift_left(road.half_width())?,
        });
    }

    let results = Results {
        intersection_id,
        intersection_polygon: Polygon::dummy(),
        debug: Vec::new(),
        trimmed_center_pts: BTreeMap::new(),
    };

    if road_lines.len() == 1 {
        super::terminus::terminus(results, roads.into_values().next().unwrap())
    } else if road_lines.len() == 2 {
        let mut iter = roads.into_values();
        super::degenerate::degenerate(results, iter.next().unwrap(), iter.next().unwrap())
    } else if !trim_roads_for_merging.is_empty() {
        pretrimmed_geometry(results, roads, &road_lines)
    } else if let Some(result) =
        super::on_off_ramp::on_off_ramp(results.clone(), roads.clone(), road_lines.clone())
    {
        Ok(result)
    } else {
        trim_to_corners(results, roads, sorted_roads)
    }
}

/// Handles intersections with at least 3 roads.
fn trim_to_corners(
    mut results: Results,
    mut roads: BTreeMap<RoadID, InputRoad>,
    sorted_road_ids: Vec<RoadID>,
) -> Result<Results> {
    // TODO Take Road instead of InputRoad to avoid this
    let mut sorted_roads = Vec::new();
    let mut orig_centers = BTreeMap::new();
    for id in &sorted_road_ids {
        let road = &roads[id];
        sorted_roads.push(road.to_road());
        orig_centers.insert(*id, road.center_line.clone());
    }

    // Look at every adjacent pair of edges
    let mut edges = RoadEdge::calculate(sorted_roads.iter().collect(), results.intersection_id);
    edges.push(edges[0].clone());

    for pair in edges.windows(2) {
        let one = &pair[0];
        let two = &pair[1];

        // Only want corners between two roads
        if one.road == two.road {
            continue;
        }

        // Look for where the two road edges collide, closest to the intersection.
        if let Some((pt, _)) = one.pl.reversed().intersection(&two.pl.reversed()) {
            // For both edges, project perpendicularly back to the original center, and trim back
            // to that point.
            for side in [one, two] {
                if let Some((_, angle)) = side.pl.dist_along_of_point(pt) {
                    let perp = InfiniteLine::from_pt_angle(pt, angle.rotate_degs(90.0));

                    // Make the road center point away from the intersection, so we find the hit
                    // closest to the intersection. Note this needs to use the original
                    // center_line, not anything trimmed in a previous iteration of this loop!
                    let mut center_away = orig_centers[&side.road].clone();
                    if roads[&side.road].dst_i == results.intersection_id {
                        center_away = center_away.reversed();
                    }

                    let mut trim_candidates = Vec::new();
                    for trim_to in all_intersection_infinite(&center_away, &perp) {
                        trim_candidates.extend(center_away.get_slice_starting_at(trim_to));
                    }
                    // Find the candidate producing the minimal trim, aka, the hit closest to the
                    // intersection.
                    if let Some(mut trimmed) =
                        trim_candidates.into_iter().max_by_key(|pl| pl.length())
                    {
                        // Every road has two sides, so we'll generate two potential trims. Take
                        // the shortest.
                        if trimmed.length() < roads[&side.road].center_line.length() {
                            // Don't forget to match orientation!
                            if roads[&side.road].dst_i == results.intersection_id {
                                trimmed = trimmed.reversed();
                            }
                            roads.get_mut(&side.road).unwrap().center_line = trimmed;
                        }
                    }
                }
            }
        }
        // TODO If there's no hit, consider extending both lines and seeing if they hit
    }

    // After trimming all the roads, look at the edges again
    let mut sorted_roads = Vec::new();
    for id in sorted_road_ids {
        sorted_roads.push(roads[&id].to_road());
    }
    let mut edges = RoadEdge::calculate(sorted_roads.iter().collect(), results.intersection_id);
    edges.push(edges[0].clone());

    // Form the intersection polygon by using the endpoints of each road edge.
    let mut endpts = Vec::new();
    for pair in edges.windows(2) {
        let one = &pair[0];
        let two = &pair[1];

        endpts.push(one.pl.last_pt());

        if one.road != two.road {
            // But also, we want to use the original points where untrimmed road edges collided.
            // We didn't retain those in the main loop above. So instead, let's use the trimmed
            // edges. If the other side of a road produced a larger trim, this side won't collide.
            // So extend the side until it has the same length as the original untrimmed line. Note
            // all the reversing is to find the hit closest to the intersection.
            if let Some((corner, _)) = one
                .pl
                .extend_to_length(orig_centers[&one.road].length())
                .reversed()
                .intersection(
                    &two.pl
                        .extend_to_length(orig_centers[&two.road].length())
                        .reversed(),
                )
            {
                endpts.push(corner);
            }
        }
    }
    endpts.push(endpts[0]);
    if let Ok(ring) = Ring::deduping_new(endpts) {
        results.intersection_polygon = ring.into_polygon();
    }

    for road in roads.into_values() {
        results.trimmed_center_pts.insert(road.id, road.center_line);
    }

    Ok(results)
}

fn pretrimmed_geometry(
    mut results: Results,
    roads: BTreeMap<RoadID, InputRoad>,
    road_lines: &[RoadLine],
) -> Result<Results> {
    let mut endpoints: Vec<Pt2D> = Vec::new();
    for r in road_lines {
        let r = &roads[&r.id];
        // Shift those final centers out again to find the main endpoints for the polygon.
        if r.dst_i == results.intersection_id {
            endpoints.push(r.center_line.shift_right(r.half_width())?.last_pt());
            endpoints.push(r.center_line.shift_left(r.half_width())?.last_pt());
        } else {
            endpoints.push(r.center_line.shift_left(r.half_width())?.first_pt());
            endpoints.push(r.center_line.shift_right(r.half_width())?.first_pt());
        }
    }

    results.intersection_polygon = Ring::new(close_off_polygon(Pt2D::approx_dedupe(
        endpoints,
        Distance::meters(0.1),
    )))?
    .into_polygon();
    for (id, r) in roads {
        results.trimmed_center_pts.insert(id, r.center_line);
    }
    Ok(results)
}

// A PolyLine could hit an InfiniteLine at multiple points. Return all of those points.
fn all_intersection_infinite(pl: &PolyLine, other: &InfiniteLine) -> Vec<Pt2D> {
    let mut hits = Vec::new();
    for l in pl.lines() {
        hits.extend(l.intersection_infinite(other));
    }
    hits
}
