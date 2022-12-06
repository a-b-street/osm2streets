use std::collections::BTreeMap;

use geom::{Circle, Distance, InfiniteLine, PolyLine, Pt2D, Ring, EPSILON_DIST};

use super::{close_off_polygon, Results, RoadLine, DEGENERATE_INTERSECTION_HALF_LENGTH};
use crate::{InputRoad, IntersectionID, RoadID};

// The normal generalized_trim_back approach produces huge intersections when 3 roads meet at
// certain angles. It usually happens for highway on/off ramps. Try something different here. In
// lieu of proper docs, see https://twitter.com/CarlinoDustin/status/1290799086036111360.
pub(crate) fn on_off_ramp(
    mut results: Results,
    mut roads: BTreeMap<RoadID, InputRoad>,
    road_lines: Vec<RoadLine>,
) -> Option<Results> {
    if road_lines.len() != 3 {
        return None;
    }
    // TODO Really this should apply based on some geometric consideration (one of the endpoints
    // totally inside the other thick road's polygon) or Connections, but for the moment, this is
    // an OK filter.
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
        .contains(&roads[&r.id].highway_type.as_str())
    }) {
        return None;
    }

    let mut pieces = Vec::new();
    // TODO Use this abstraction for all the code here?
    for r in road_lines {
        let road = &roads[&r.id];
        let center = if road.dst_i == results.intersection_id {
            road.center_line.clone()
        } else {
            road.center_line.reversed()
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
            roads[&r.id].half_width(),
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
        roads.get_mut(&thin.id).unwrap().center_line = trimmed_thin;

        // Trim the thick extra ends at the intersection
        let extra = if roads[&thick_id].dst_i == results.intersection_id {
            roads[&thick_id]
                .center_line
                .get_slice_starting_at(trimmed_thick.last_pt())?
        } else {
            trimmed_thick = trimmed_thick.reversed();
            roads[&thick_id]
                .center_line
                .get_slice_ending_at(trimmed_thick.first_pt())?
                .reversed()
        };
        roads.get_mut(&thick_id).unwrap().center_line = trimmed_thick;
        // Give the merge point some length
        if extra.length() <= 2.0 * DEGENERATE_INTERSECTION_HALF_LENGTH + 3.0 * EPSILON_DIST {
            return None;
        }
        let extra = extra.exact_slice(2.0 * DEGENERATE_INTERSECTION_HALF_LENGTH, extra.length());

        // Now the crazy part -- take the other thick, and LENGTHEN it
        let other = roads
            .get_mut(if thick1.id == thick_id {
                &thick2.id
            } else {
                &thick1.id
            })
            .unwrap();
        if other.dst_i == results.intersection_id {
            other.center_line = other.center_line.clone().extend(extra.reversed()).ok()?;
        } else {
            other.center_line = extra.extend(other.center_line.clone()).ok()?;
        }
    }

    // Now build the actual polygon
    let mut endpoints = Vec::new();
    for id in [thin.id, thick1.id, thick2.id] {
        let r = &roads[&id];
        // Shift those final centers out again to find the main endpoints for the polygon.
        if r.dst_i == results.intersection_id {
            endpoints.push(r.center_line.shift_right(r.half_width()).ok()?.last_pt());
            endpoints.push(r.center_line.shift_left(r.half_width()).ok()?.last_pt());
        } else {
            endpoints.push(r.center_line.shift_left(r.half_width()).ok()?.first_pt());
            endpoints.push(r.center_line.shift_right(r.half_width()).ok()?.first_pt());
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
    for (id, r) in roads {
        results.trimmed_center_pts.insert(id, r.center_line);
    }
    Some(results)
}

// The lines all end at the intersection
struct Piece {
    id: RoadID,
    dst_i: IntersectionID,
    left: PolyLine,
    center: PolyLine,
    right: PolyLine,
}
