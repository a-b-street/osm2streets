use std::collections::BTreeMap;

use anyhow::Result;

use abstutil::wraparound_get;
use geom::{Circle, Distance, InfiniteLine, PolyLine, Polygon, Pt2D, Ring, EPSILON_DIST};

use super::Results;
use crate::{IntersectionID, Road, RoadID};

const DEGENERATE_INTERSECTION_HALF_LENGTH: Distance = Distance::const_meters(2.5);

pub fn intersection_polygon(
    intersection_id: IntersectionID,
    // These must be sorted clockwise already
    input_roads: Vec<Road>,
    trim_roads_for_merging: &BTreeMap<(RoadID, bool), Pt2D>,
) -> Result<Results> {
    let mut results = Results {
        intersection_id,
        intersection_polygon: Polygon::dummy(),
        debug: Vec::new(),
        roads: BTreeMap::new(),
    };

    let mut sorted_roads: Vec<RoadID> = Vec::new();
    for r in input_roads {
        sorted_roads.push(r.id);
        results.roads.insert(r.id, r);
    }

    if results.roads.is_empty() {
        bail!("{intersection_id} has no roads");
    }

    // First pre-trim roads if it's a consolidated intersection.
    for road in results.roads.values_mut() {
        if let Some(endpt) = trim_roads_for_merging.get(&(road.id, road.src_i == intersection_id)) {
            if road.src_i == intersection_id {
                match road.trimmed_center_line.safe_get_slice_starting_at(*endpt) {
                    Some(pl) => {
                        road.trimmed_center_line = pl;
                    }
                    None => {
                        error!("{}'s trimmed points start past the endpt {endpt}", road.id);
                        // Just skip. See https://github.com/a-b-street/abstreet/issues/654 for a
                        // start to diagnose. Repro at https://www.openstreetmap.org/node/53211693.
                    }
                }
            } else {
                assert_eq!(road.dst_i, intersection_id);
                match road.trimmed_center_line.safe_get_slice_ending_at(*endpt) {
                    Some(pl) => {
                        road.trimmed_center_line = pl;
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
        let road = &results.roads[&id];
        let center_pl = if road.src_i == intersection_id {
            road.trimmed_center_line.reversed()
        } else if road.dst_i == intersection_id {
            road.trimmed_center_line.clone()
        } else {
            panic!("Incident road {id} doesn't have an endpoint at {intersection_id}");
        };
        road_lines.push(RoadLine {
            id,
            fwd_pl: center_pl.shift_right(road.half_width())?,
            back_pl: center_pl.shift_left(road.half_width())?,
        });
    }

    if road_lines.len() == 1 {
        return deadend(results, &road_lines);
    }

    if !trim_roads_for_merging.is_empty() {
        pretrimmed_geometry(results, &road_lines)
    } else if let Some(result) = on_off_ramp(results.clone(), road_lines.clone()) {
        Ok(result)
    } else {
        generalized_trim_back(results, &road_lines)
    }
}

// TODO Dedupe with Piece!
#[derive(Clone)]
struct RoadLine {
    id: RoadID,
    // Both are oriented to be incoming to the intersection (ending at it).
    // TODO Maybe express as the "right" and "left"
    fwd_pl: PolyLine,
    back_pl: PolyLine,
}

fn generalized_trim_back(mut results: Results, input_road_lines: &[RoadLine]) -> Result<Results> {
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

    // Intersect every road's boundary lines with all the other lines. Update new_road_centers as
    // we go. Do NOT modify trimmed_center_line in results yet, because every road gets trimmed
    // multiple times below (every road has two sides and thus two corners) and we need to use the
    // original line.
    let mut new_road_centers: BTreeMap<RoadID, PolyLine> = BTreeMap::new();
    for (r1, pl1) in &road_lines {
        let road1 = &results.roads[r1];

        // Make road_center end at the intersection
        let road_center = if road1.dst_i == i {
            road1.trimmed_center_line.clone()
        } else {
            road1.trimmed_center_line.reversed()
        };

        // Always trim back a minimum amount, if possible.
        // TODO Should we only do this for 2 roads?
        let mut shortest_center = road_center.clone();

        if road_center.length() >= DEGENERATE_INTERSECTION_HALF_LENGTH + 3.0 * EPSILON_DIST {
            // Don't repeatedly apply this. If the untrimmed and trimmed endpoints are the
            // same, do trim.
            // TODO This'll be easier to detect with trim_start and trim_end
            let untrimmed = road1.untrimmed_road_geometry().0;
            let do_trim = if road1.dst_i == i {
                road1.trimmed_center_line.last_pt() == untrimmed.last_pt()
            } else {
                road1.trimmed_center_line.first_pt() == untrimmed.first_pt()
            };
            if do_trim {
                shortest_center = road_center.exact_slice(
                    Distance::ZERO,
                    road_center.length() - DEGENERATE_INTERSECTION_HALF_LENGTH,
                );
            }
        }

        for (r2, pl2) in &road_lines {
            if r1 == r2 {
                continue;
            }

            // If two roads go between the same intersections, they'll likely hit at the wrong
            // side. Just use the second half of the polyline to circumvent this. But sadly, doing
            // this in general breaks other cases -- sometimes we want to find the collision
            // farther away from the intersection in question.
            let same_endpoints = {
                let ii1 = road1.src_i;
                let ii2 = road1.dst_i;
                let ii3 = results.roads[r2].src_i;
                let ii4 = results.roads[r2].dst_i;
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
                     two ways meet at exactly one node.");
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
                         or the trimmed thing is empty");
                }

                // We could also do the update for r2, but we'll just get to it later.
            }
        }

        let new_center = if road1.dst_i == i {
            shortest_center
        } else {
            shortest_center.reversed()
        };

        // Never lengthen something from a previous pass
        if new_road_centers
            .get(r1)
            .map(|pl| new_center.length() < pl.length())
            .unwrap_or(true)
        {
            new_road_centers.insert(*r1, new_center);
        }
    }
    for (id, pl) in new_road_centers {
        results.roads.get_mut(&id).unwrap().trimmed_center_line = pl;
    }

    calculate_polygon(results, input_road_lines)
}

fn calculate_polygon(mut results: Results, input_road_lines: &[RoadLine]) -> Result<Results> {
    let mut endpoints: Vec<Pt2D> = Vec::new();
    for idx in 0..input_road_lines.len() as isize {
        let (id, fwd_pl, back_pl) = {
            let r = wraparound_get(input_road_lines, idx);
            (r.id, &r.fwd_pl, &r.back_pl)
        };
        // TODO Ahhh these names are confusing. Adjacent to the fwd_pl, but it's a back pl.
        let adj_back_pl = &wraparound_get(input_road_lines, idx + 1).fwd_pl;
        let adj_fwd_pl = &wraparound_get(input_road_lines, idx - 1).back_pl;

        let r = &results.roads[&id];

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
        if r.dst_i == results.intersection_id {
            endpoints.push(r.trimmed_center_line.shift_right(r.half_width())?.last_pt());
            endpoints.push(r.trimmed_center_line.shift_left(r.half_width())?.last_pt());
        } else {
            endpoints.push(r.trimmed_center_line.shift_left(r.half_width())?.first_pt());
            endpoints.push(
                r.trimmed_center_line
                    .shift_right(r.half_width())?
                    .first_pt(),
            );
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
        warn!(
            "{}'s polygon has weird repeats, forcibly removing points",
            results.intersection_id
        );
        Ring::must_new(deduped).into_polygon()
    };

    // TODO Or always sort points? Helps some cases, hurts other for downtown Seattle.
    /*endpoints.sort_by_key(|pt| pt.to_hashable());
    endpoints = Pt2D::approx_dedupe(endpoints, Distance::meters(0.1));
    let center = Pt2D::center(&endpoints);
    endpoints.sort_by_key(|pt| pt.angle_to(center).normalized_degrees() as i64);
    close_off_polygon(endpoints)*/

    Ok(results)
}

fn pretrimmed_geometry(mut results: Results, road_lines: &[RoadLine]) -> Result<Results> {
    let mut endpoints: Vec<Pt2D> = Vec::new();
    for r in road_lines {
        let r = &results.roads[&r.id];
        // Shift those final centers out again to find the main endpoints for the polygon.
        if r.dst_i == results.intersection_id {
            endpoints.push(r.trimmed_center_line.shift_right(r.half_width())?.last_pt());
            endpoints.push(r.trimmed_center_line.shift_left(r.half_width())?.last_pt());
        } else {
            endpoints.push(r.trimmed_center_line.shift_left(r.half_width())?.first_pt());
            endpoints.push(
                r.trimmed_center_line
                    .shift_right(r.half_width())?
                    .first_pt(),
            );
        }
    }

    // TODO Do all of the crazy deduping that calculate_polygon does?
    results.intersection_polygon = Ring::new(close_off_polygon(Pt2D::approx_dedupe(
        endpoints,
        Distance::meters(0.1),
    )))?
    .into_polygon();
    Ok(results)
}

fn deadend(mut results: Results, road_lines: &[RoadLine]) -> Result<Results> {
    let len = DEGENERATE_INTERSECTION_HALF_LENGTH * 4.0;

    let id = road_lines[0].id;
    let mut pl_a = road_lines[0].fwd_pl.clone();
    let mut pl_b = road_lines[0].back_pl.clone();
    // If the lines are too short (usually due to the boundary polygon clipping roads too
    // much), just extend them.
    // TODO Not sure why we need +1.5x more, but this looks better. Some math is definitely off
    // somewhere.
    pl_a = pl_a.extend_to_length(len + 1.5 * DEGENERATE_INTERSECTION_HALF_LENGTH);
    pl_b = pl_b.extend_to_length(len + 1.5 * DEGENERATE_INTERSECTION_HALF_LENGTH);

    let r = results.roads.get_mut(&id).unwrap();
    let len_with_buffer = len + 3.0 * EPSILON_DIST;
    let trimmed = if r.trimmed_center_line.length() >= len_with_buffer {
        if r.src_i == results.intersection_id {
            r.trimmed_center_line = r
                .trimmed_center_line
                .exact_slice(len, r.trimmed_center_line.length());
        } else {
            r.trimmed_center_line = r
                .trimmed_center_line
                .exact_slice(Distance::ZERO, r.trimmed_center_line.length() - len);
        }
        r.trimmed_center_line.clone()
    } else if r.src_i == results.intersection_id {
        r.trimmed_center_line.extend_to_length(len_with_buffer)
    } else {
        r.trimmed_center_line
            .reversed()
            .extend_to_length(len_with_buffer)
            .reversed()
    };

    // After trimming the center points, the two sides of the road may be at different
    // points, so shift the center out again to find the endpoints.
    // TODO Refactor with generalized_trim_back.
    let mut endpts = vec![pl_b.last_pt(), pl_a.last_pt()];
    if r.dst_i == results.intersection_id {
        endpts.push(trimmed.shift_right(r.half_width())?.last_pt());
        endpts.push(trimmed.shift_left(r.half_width())?.last_pt());
    } else {
        endpts.push(trimmed.shift_left(r.half_width())?.first_pt());
        endpts.push(trimmed.shift_right(r.half_width())?.first_pt());
    }

    endpts.dedup();
    results.intersection_polygon = Ring::must_new(close_off_polygon(endpts)).into_polygon();
    Ok(results)
}

fn close_off_polygon(mut pts: Vec<Pt2D>) -> Vec<Pt2D> {
    if pts.last().unwrap().approx_eq(pts[0], Distance::meters(0.1)) {
        pts.pop();
    }
    pts.push(pts[0]);
    pts
}

// The lines all end at the intersection
struct Piece {
    id: RoadID,
    dst_i: IntersectionID,
    left: PolyLine,
    center: PolyLine,
    right: PolyLine,
}

// The normal generalized_trim_back approach produces huge intersections when 3 roads meet at
// certain angles. It usually happens for highway on/off ramps. Try something different here. In
// lieu of proper docs, see https://twitter.com/CarlinoDustin/status/1290799086036111360.
fn on_off_ramp(mut results: Results, road_lines: Vec<RoadLine>) -> Option<Results> {
    if road_lines.len() != 3 {
        return None;
    }
    // TODO Really this should apply based on some geometric consideration (one of the endpoints
    // totally inside the other thick road's polygon), but for the moment, this is an OK filter.
    //
    // Example candidate: https://www.openstreetmap.org/node/32177767
    if !road_lines.iter().any(|r| {
        [
            "motorway",
            "motorway_link",
            "primary_link",
            "secondary_link",
            "tertiary_link",
            "trunk_link",
        ]
        .contains(&results.roads[&r.id].highway_type.as_str())
    }) {
        return None;
    }

    let mut pieces = Vec::new();
    // TODO Use this abstraction for all the code here?
    for r in road_lines {
        let road = &results.roads[&r.id];
        let center = if road.dst_i == results.intersection_id {
            road.trimmed_center_line.clone()
        } else {
            road.trimmed_center_line.reversed()
        };
        pieces.push(Piece {
            id: road.id,
            dst_i: road.dst_i,
            left: r.back_pl,
            center,
            right: r.fwd_pl,
        });
    }

    // Break ties by preferring the outbound roads for thin
    pieces.sort_by_key(|r| {
        (
            results.roads[&r.id].half_width(),
            r.dst_i == results.intersection_id,
        )
    });
    let thick1 = pieces.pop().unwrap();
    let thick2 = pieces.pop().unwrap();
    let thin = pieces.pop().unwrap();

    // Find where the thin hits the thick farthest along.
    // (trimmed thin center, trimmed thick center, the thick road we hit)
    let mut best_hit: Option<(PolyLine, PolyLine, RoadID)> = None;
    for thin_pl in [&thin.left, &thin.right] {
        for thick in [&thick1, &thick2] {
            for thick_pl in [&thick.left, &thick.right] {
                if thin_pl == thick_pl {
                    // How? Just bail.
                    return None;
                }
                if let Some((hit, angle)) = thin_pl.intersection(thick_pl) {
                    // Find where the perpendicular hits the original road line
                    // TODO Refactor something to go from a hit+angle on a left/right to a trimmed
                    // center.
                    let perp = InfiniteLine::from_pt_angle(hit, angle.rotate_degs(90.0));
                    let trimmed_thin = thin
                        .center
                        .reversed()
                        .intersection_infinite(&perp)
                        .and_then(|trim_to| thin.center.get_slice_ending_at(trim_to))?;

                    // Do the same for the thick road
                    let (_, angle) = thick_pl.dist_along_of_point(hit)?;
                    let perp = InfiniteLine::from_pt_angle(hit, angle.rotate_degs(90.0));
                    let trimmed_thick = thick
                        .center
                        .reversed()
                        .intersection_infinite(&perp)
                        .and_then(|trim_to| thick.center.get_slice_ending_at(trim_to))?;

                    if false {
                        results.debug.push((
                            "1".to_string(),
                            Circle::new(hit, Distance::meters(3.0)).to_polygon(),
                        ));
                        results.debug.push((
                            "2".to_string(),
                            Circle::new(trimmed_thin.last_pt(), Distance::meters(3.0)).to_polygon(),
                        ));
                        results.debug.push((
                            "3".to_string(),
                            Circle::new(trimmed_thick.last_pt(), Distance::meters(3.0))
                                .to_polygon(),
                        ));
                    }
                    if best_hit
                        .as_ref()
                        .map(|(pl, _, _)| trimmed_thin.length() < pl.length())
                        .unwrap_or(true)
                    {
                        best_hit = Some((trimmed_thin, trimmed_thick, thick.id));
                    }
                }
            }
        }
    }

    {
        // Trim the thin
        let (mut trimmed_thin, mut trimmed_thick, thick_id) = best_hit?;
        if thin.dst_i != results.intersection_id {
            trimmed_thin = trimmed_thin.reversed();
        }
        results.roads.get_mut(&thin.id).unwrap().trimmed_center_line = trimmed_thin;

        // Trim the thick extra ends at the intersection
        let extra = if results.roads[&thick_id].dst_i == results.intersection_id {
            results.roads[&thick_id]
                .trimmed_center_line
                .get_slice_starting_at(trimmed_thick.last_pt())?
        } else {
            trimmed_thick = trimmed_thick.reversed();
            results.roads[&thick_id]
                .trimmed_center_line
                .get_slice_ending_at(trimmed_thick.first_pt())?
                .reversed()
        };
        results
            .roads
            .get_mut(&thick_id)
            .unwrap()
            .trimmed_center_line = trimmed_thick;
        // Give the merge point some length
        if extra.length() <= 2.0 * DEGENERATE_INTERSECTION_HALF_LENGTH + 3.0 * EPSILON_DIST {
            return None;
        }
        let extra = extra.exact_slice(2.0 * DEGENERATE_INTERSECTION_HALF_LENGTH, extra.length());

        // Now the crazy part -- take the other thick, and LENGTHEN it
        let other = results
            .roads
            .get_mut(if thick1.id == thick_id {
                &thick2.id
            } else {
                &thick1.id
            })
            .unwrap();
        if other.dst_i == results.intersection_id {
            other.trimmed_center_line = other
                .trimmed_center_line
                .clone()
                .extend(extra.reversed())
                .ok()?;
        } else {
            other.trimmed_center_line = extra.extend(other.trimmed_center_line.clone()).ok()?;
        }
    }

    // Now build the actual polygon
    let mut endpoints = Vec::new();
    for id in [thin.id, thick1.id, thick2.id] {
        let r = &results.roads[&id];
        // Shift those final centers out again to find the main endpoints for the polygon.
        if r.dst_i == results.intersection_id {
            endpoints.push(
                r.trimmed_center_line
                    .shift_right(r.half_width())
                    .ok()?
                    .last_pt(),
            );
            endpoints.push(
                r.trimmed_center_line
                    .shift_left(r.half_width())
                    .ok()?
                    .last_pt(),
            );
        } else {
            endpoints.push(
                r.trimmed_center_line
                    .shift_left(r.half_width())
                    .ok()?
                    .first_pt(),
            );
            endpoints.push(
                r.trimmed_center_line
                    .shift_right(r.half_width())
                    .ok()?
                    .first_pt(),
            );
        }
    }
    /*for (idx, pt) in endpoints.iter().enumerate() {
        debug.push((format!("{}", idx), Circle::new(*pt, Distance::meters(2.0)).to_polygon()));
    }*/

    endpoints.sort_by_key(|pt| pt.to_hashable());
    endpoints.dedup();
    let center = Pt2D::center(&endpoints);
    endpoints.sort_by_key(|pt| pt.angle_to(center).normalized_degrees() as i64);
    endpoints.dedup();
    results.intersection_polygon = Ring::must_new(close_off_polygon(endpoints)).into_polygon();
    Some(results)
}
