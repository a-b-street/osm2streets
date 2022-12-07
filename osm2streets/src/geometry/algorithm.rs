use std::collections::BTreeMap;

use anyhow::Result;

use abstutil::wraparound_get;
use geom::{Distance, InfiniteLine, PolyLine, Polygon, Pt2D, Ring, EPSILON_DIST};

use super::{close_off_polygon, Results, RoadLine, DEGENERATE_INTERSECTION_HALF_LENGTH};
use crate::{InputRoad, IntersectionID, RoadID};

pub fn intersection_polygon(
    intersection_id: IntersectionID,
    // These must be sorted clockwise already
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

    // Sorted clockwise
    let mut road_lines = Vec::new();
    for id in sorted_roads {
        let road = &roads[&id];
        let center_pl = if road.src_i == intersection_id {
            road.center_line.reversed()
        } else if road.dst_i == intersection_id {
            road.center_line.clone()
        } else {
            panic!("Incident road {id} doesn't have an endpoint at {intersection_id}");
        };
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
        return super::terminus::terminus(results, roads.into_values().next().unwrap());
    } else if road_lines.len() == 2 {
        let mut iter = roads.into_values();
        return super::degenerate::degenerate(results, iter.next().unwrap(), iter.next().unwrap());
    }

    if !trim_roads_for_merging.is_empty() {
        pretrimmed_geometry(results, roads, &road_lines)
    } else if let Some(result) =
        super::on_off_ramp::on_off_ramp(results.clone(), roads.clone(), road_lines.clone())
    {
        Ok(result)
    } else {
        generalized_trim_back(results, roads, &road_lines)
    }
}

fn generalized_trim_back(
    mut results: Results,
    mut roads: BTreeMap<RoadID, InputRoad>,
    input_road_lines: &[RoadLine],
) -> Result<Results> {
    let i = results.intersection_id;

    let mut road_lines: Vec<(RoadID, PolyLine)> = Vec::new();
    for r in input_road_lines {
        road_lines.push((r.id, r.fwd_pl.clone()));
        road_lines.push((r.id, r.back_pl.clone()));

        if false {
            results.debug.push((
                format!("{} fwd", r.id),
                r.fwd_pl.make_polygons(Distance::meters(1.0)),
            ));
            results.debug.push((
                format!("{} back", r.id),
                r.back_pl.make_polygons(Distance::meters(1.0)),
            ));
        }
    }

    // Intersect every road's boundary lines with all the other lines. Only side effect here is to
    // populate new_road_centers.
    let mut new_road_centers: BTreeMap<RoadID, PolyLine> = BTreeMap::new();
    // TODO If Results has a BTreeMap too, we could just fill this out as we go
    for (r1, pl1) in &road_lines {
        // road_center ends at the intersection.
        let road_center = if roads[r1].dst_i == i {
            roads[r1].center_line.clone()
        } else {
            roads[r1].center_line.reversed()
        };

        // Always trim back a minimum amount, if possible.
        let mut shortest_center =
            if road_center.length() >= DEGENERATE_INTERSECTION_HALF_LENGTH + 3.0 * EPSILON_DIST {
                road_center.exact_slice(
                    Distance::ZERO,
                    road_center.length() - DEGENERATE_INTERSECTION_HALF_LENGTH,
                )
            } else {
                road_center.clone()
            };

        for (r2, pl2) in &road_lines {
            if r1 == r2 {
                continue;
            }

            // If two roads go between the same intersections, they'll likely hit at the wrong
            // side. Just use the second half of the polyline to circumvent this. But sadly, doing
            // this in general breaks other cases -- sometimes we want to find the collision
            // farther away from the intersection in question.
            let same_endpoints = {
                let ii1 = roads[r1].src_i;
                let ii2 = roads[r1].dst_i;
                let ii3 = roads[r2].src_i;
                let ii4 = roads[r2].dst_i;
                (ii1 == ii3 && ii2 == ii4) || (ii1 == ii4 && ii2 == ii3)
            };
            let (use_pl1, use_pl2): (PolyLine, PolyLine) = if same_endpoints {
                (pl1.second_half()?, pl2.second_half()?)
            } else {
                (pl1.clone(), pl2.clone())
            };

            if use_pl1 == use_pl2 {
                bail!(
                    "{r1} and {r2} have overlapping segments. You likely need to fix OSM and make the \
                     two ways meet at exactly one node."
                );
            }

            // Sometimes two road PLs may hit at multiple points because they're thick and close
            // together. pl1.intersection(pl2) returns the "first" hit from pl1's
            // perspective, so reverse it, ensuring we find the hit closest to the
            // intersection we're working on.
            // TODO I hoped this would subsume the second_half() hack above, but it sadly doesn't.
            if let Some((hit, angle)) = use_pl1.reversed().intersection(&use_pl2) {
                // Find where the perpendicular hits the original road line
                let perp = InfiniteLine::from_pt_angle(hit, angle.rotate_degs(90.0));
                // How could something perpendicular to a shifted polyline never hit the original
                // polyline? Also, find the hit closest to the intersection -- this matters for
                // very curvy roads, like highway ramps.
                if let Some(trimmed) = road_center
                    .reversed()
                    .intersection_infinite(&perp)
                    .and_then(|trim_to| road_center.get_slice_ending_at(trim_to))
                {
                    if trimmed.length() < shortest_center.length() {
                        shortest_center = trimmed;
                    }
                } else {
                    warn!(
                        "{r1} and {r2} hit, but the perpendicular never hit the original center line, \
                         or the trimmed thing is empty"
                    );
                }

                // We could also do the update for r2, but we'll just get to it later.
            }
        }

        let new_center = if roads[r1].dst_i == i {
            shortest_center
        } else {
            shortest_center.reversed()
        };
        if let Some(existing) = new_road_centers.get(r1) {
            if new_center.length() < existing.length() {
                new_road_centers.insert(*r1, new_center);
            }
        } else {
            new_road_centers.insert(*r1, new_center);
        }
    }

    // After doing all the intersection checks, copy over the new centers. Also fill out the
    // intersection polygon's points along the way.
    let mut endpoints: Vec<Pt2D> = Vec::new();
    for idx in 0..input_road_lines.len() as isize {
        let (id, fwd_pl, back_pl) = {
            let r = wraparound_get(input_road_lines, idx);
            (r.id, &r.fwd_pl, &r.back_pl)
        };
        // TODO Ahhh these names are confusing. Adjacent to the fwd_pl, but it's a back pl.
        let adj_back_pl = &wraparound_get(input_road_lines, idx + 1).fwd_pl;
        let adj_fwd_pl = &wraparound_get(input_road_lines, idx - 1).back_pl;

        roads.get_mut(&id).unwrap().center_line = new_road_centers[&id].clone();
        let r = &roads[&id];

        // Include collisions between polylines of adjacent roads, so the polygon doesn't cover area
        // not originally covered by the thick road bands.
        // Always take the second_half here to handle roads that intersect at multiple points.
        // TODO Should maybe do reversed() to fwd_pl here too. And why not make all the lines
        // passed in point AWAY from the intersection instead?
        if fwd_pl.length() >= EPSILON_DIST * 3.0 && adj_fwd_pl.length() >= EPSILON_DIST * 3.0 {
            if let Some((hit, _)) = fwd_pl
                .second_half()?
                .intersection(&adj_fwd_pl.second_half()?)
            {
                endpoints.push(hit);
            }
        } else {
            warn!(
                "Excluding collision between original polylines of {id} and something, because \
                 stuff's too short"
            );
        }

        // Shift those final centers out again to find the main endpoints for the polygon.
        if r.dst_i == i {
            endpoints.push(r.center_line.shift_right(r.half_width())?.last_pt());
            endpoints.push(r.center_line.shift_left(r.half_width())?.last_pt());
        } else {
            endpoints.push(r.center_line.shift_left(r.half_width())?.first_pt());
            endpoints.push(r.center_line.shift_right(r.half_width())?.first_pt());
        }

        if back_pl.length() >= EPSILON_DIST * 3.0 && adj_back_pl.length() >= EPSILON_DIST * 3.0 {
            if let Some((hit, _)) = back_pl
                .second_half()?
                .intersection(&adj_back_pl.second_half()?)
            {
                endpoints.push(hit);
            }
        } else {
            warn!(
                "Excluding collision between original polylines of {id} and something, because \
                 stuff's too short"
            );
        }
    }

    // There are bad polygons caused by weird short roads. As a temporary workaround, detect cases
    // where polygons dramatically double back on themselves and force the polygon to proceed
    // around its center.
    let main_result = close_off_polygon(Pt2D::approx_dedupe(endpoints, Distance::meters(0.1)));
    let mut deduped = main_result.clone();
    deduped.pop();
    deduped.sort_by_key(|pt| pt.to_hashable());
    deduped = Pt2D::approx_dedupe(deduped, Distance::meters(0.1));
    let center = Pt2D::center(&deduped);
    deduped.sort_by_key(|pt| pt.angle_to(center).normalized_degrees() as i64);
    deduped = Pt2D::approx_dedupe(deduped, Distance::meters(0.1));
    deduped = close_off_polygon(deduped);

    results.intersection_polygon = if main_result.len() == deduped.len() {
        Ring::must_new(main_result).into_polygon()
    } else {
        warn!("{i}'s polygon has weird repeats, forcibly removing points");
        Ring::must_new(deduped).into_polygon()
    };

    // TODO Or always sort points? Helps some cases, hurts other for downtown Seattle.
    /*endpoints.sort_by_key(|pt| pt.to_hashable());
    endpoints = Pt2D::approx_dedupe(endpoints, Distance::meters(0.1));
    let center = Pt2D::center(&endpoints);
    endpoints.sort_by_key(|pt| pt.angle_to(center).normalized_degrees() as i64);
    close_off_polygon(endpoints)*/

    // TODO We always do this. Maybe Results has the InputRoad and we just work in-place
    for (id, r) in roads {
        results.trimmed_center_pts.insert(id, r.center_line);
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

    // TODO Do all of the crazy deduping that generalized_trim_back does?
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
