use std::collections::BTreeMap;

use anyhow::Result;
use geom::{InfiniteLine, PolyLine, Pt2D};

use super::{polygon_from_corners, Results};
use crate::road::RoadEdge;
use crate::{InputRoad, RoadID, CommonEndpoint};

/// Handles intersections with at least 3 roads.
pub fn trim_to_corners(
    mut results: Results,
    mut roads: BTreeMap<RoadID, InputRoad>,
    sorted_road_ids: Vec<RoadID>,
) -> Result<Results> {
    // TODO Take Road instead of InputRoad to avoid this
    let mut sorted_roads = Vec::new();
    let mut orig_centers = BTreeMap::new();
    let mut intersection_endpoints = BTreeMap::new();
    for id in &sorted_road_ids {
        let road = &roads[id];
        sorted_roads.push(road.to_road());
        orig_centers.insert(*id, road.center_line.clone());

        if results.intersection_id.0 == 5 {
            //info!("one of sorted input roads is {id}");
        }

        // What does overwriting mean?
        intersection_endpoints.insert(road.src_i, road.center_line.first_pt());
        intersection_endpoints.insert(road.dst_i, road.center_line.last_pt());
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
        if let Some((mut pt, _)) = one.pl.reversed().intersection(&two.pl.reversed()) {
            // TODO Hack. PolyLine intersection appears to be broken when the first points match.
            // Fix upstream.
            if one.pl.last_pt() == two.pl.last_pt() {
                pt = one.pl.last_pt();
                error!("the weird case!!! {}, {}, {}", one.road, two.road, results.intersection_id);
            }


            // Do the roads form a loop?
            if CommonEndpoint::new((roads[&one.road].src_i, roads[&one.road].dst_i), (roads[&two.road].src_i, roads[&two.road].dst_i)) == CommonEndpoint::Both {
                let should_be_small = intersection_endpoints[&results.intersection_id].dist_to(pt);
                // The other intersection
                let other_i = if roads[&one.road].src_i == results.intersection_id {
                    roads[&one.road].dst_i
                } else {
                    roads[&one.road].src_i
                };
                let should_be_large = intersection_endpoints[&other_i].dist_to(pt);

                error!("WARNING! we have a loop. this side {should_be_small}, other side {should_be_large}");

                // hit point closer to the wrong side
                if should_be_small > should_be_large {
                    continue;
                }
            }


            results.debug.push((pt, format!("X from {} and {} at {}", one.road, two.road, results.intersection_id)));
            if results.intersection_id.0 == 5 || results.intersection_id.0 == 9 {
                info!("X from {} {:?} and {} {:?} at {}... {}", one.road, one.side, two.road, two.side, results.intersection_id, pt);
            }

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

                    if results.intersection_id.0 == 5 {
                        //info!("btwn {} {:?} and {} {:?} for {}, we have trim candidates {:?}", one.road, one.side, two.road, two.side, side.road, trim_candidates.iter().map(|pl| pl.length()).collect::<Vec<_>>());
                        for x in &trim_candidates {
                            //results.debug_pl.push((x.clone(), "trimcan".to_string()));
                        }
                    }


                    // Find the candidate producing the minimal trim, aka, the hit closest to the
                    // intersection.
                    if let Some(mut trimmed) =
                        trim_candidates.into_iter().max_by_key(|pl| pl.length())
                        //trim_candidates.into_iter().min_by_key(|pl| pl.first_pt().dist_to(pt))
                    {
                        // Every road has two sides, so we'll generate two potential trims. Take
                        // the shortest.
                        if trimmed.length() < roads[&side.road].center_line.length() {
                            // Don't forget to match orientation!
                            if roads[&side.road].dst_i == results.intersection_id {
                                trimmed = trimmed.reversed();
                            }
                            if results.intersection_id.0 == 5 || results.intersection_id.0 == 9 {
                                info!("trimming {} from {} to {}, due to hit btwn {} {:?} and {} {:?} at {}", side.road, roads[&side.road].center_line.length(), trimmed.length(), one.road, one.side, two.road, two.side, results.intersection_id);
                            }
                            roads.get_mut(&side.road).unwrap().center_line = trimmed;
                        }
                    }
                }
            }
        }
        // TODO If there's no hit, consider extending both lines and seeing if they hit
    }

    results.intersection_polygon = polygon_from_corners(
        &roads,
        &sorted_road_ids,
        &orig_centers,
        results.intersection_id,
    )?;

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
