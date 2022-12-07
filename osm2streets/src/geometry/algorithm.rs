use std::collections::BTreeMap;

use anyhow::Result;

use geom::{InfiniteLine, PolyLine, Polygon, Pt2D, Ring};

use super::Results;
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

    let results = Results {
        intersection_id,
        intersection_polygon: Polygon::dummy(),
        debug: Vec::new(),
        trimmed_center_pts: BTreeMap::new(),
    };

    if roads.len() == 1 {
        super::terminus::terminus(results, roads.into_values().next().unwrap())
    } else if roads.len() == 2 {
        let mut iter = roads.into_values();
        super::degenerate::degenerate(results, iter.next().unwrap(), iter.next().unwrap())
    } else if !trim_roads_for_merging.is_empty() {
        super::pretrimmed::pretrimmed_geometry(results, roads, sorted_roads, trim_roads_for_merging)
    } else if let Some(result) =
        super::on_off_ramp::on_off_ramp(results.clone(), roads.clone(), &sorted_roads)
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

// A PolyLine could hit an InfiniteLine at multiple points. Return all of those points.
fn all_intersection_infinite(pl: &PolyLine, other: &InfiniteLine) -> Vec<Pt2D> {
    let mut hits = Vec::new();
    for l in pl.lines() {
        hits.extend(l.intersection_infinite(other));
    }
    hits
}
