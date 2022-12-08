use std::collections::BTreeMap;

use geom::{Distance, InfiniteLine, PolyLine, Ring, EPSILON_DIST};

use super::Results;
use crate::{InputRoad, IntersectionID, RoadID};

const MERGE_POINT_LENGTH: Distance = Distance::const_meters(5.0);

// The normal generalized_trim_back approach produces huge intersections when 3 roads meet at
// certain angles. It usually happens for highway on/off ramps. Try something different here. In
// lieu of proper docs, see https://twitter.com/CarlinoDustin/status/1290799086036111360.
pub(crate) fn on_off_ramp(
    mut results: Results,
    mut roads: BTreeMap<RoadID, InputRoad>,
    sorted_roads: &Vec<RoadID>,
) -> Option<Results> {
    if roads.len() != 3 {
        return None;
    }
    // TODO Really this should apply based on some geometric consideration (one of the endpoints
    // totally inside the other thick road's polygon) or Connections, but for the moment, this is
    // an OK filter.
    //
    // Example candidate: https://www.openstreetmap.org/node/32177767
    if !roads.values().any(|r| {
        [
            "motorway",
            "motorway_link",
            "primary_link",
            "secondary_link",
            "tertiary_link",
            "trunk_link",
        ]
        .contains(&r.highway_type.as_str())
    }) {
        return None;
    }

    let mut pieces = Vec::new();
    for r in sorted_roads {
        let road = &roads[r];
        let center = road.center_line_pointed_at(results.intersection_id);
        let left = center.shift_left(road.half_width()).ok()?;
        let right = center.shift_right(road.half_width()).ok()?;
        pieces.push(Piece {
            id: road.id,
            dst_i: road.dst_i,
            left,
            center,
            right,
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

                    results.debug.push((hit, "1".to_string()));
                    results
                        .debug
                        .push((trimmed_thin.last_pt(), "2".to_string()));
                    results
                        .debug
                        .push((trimmed_thick.last_pt(), "3".to_string()));

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
        if extra.length() <= MERGE_POINT_LENGTH + 3.0 * EPSILON_DIST {
            return None;
        }
        let extra = extra.exact_slice(MERGE_POINT_LENGTH, extra.length());

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

    // Don't use polygon_from_corners() here. It looks for additional corners, which don't make
    // sense here.
    let mut endpts = Vec::new();
    for id in sorted_roads {
        let r = &roads[&id];
        // Shift those final centers out again to find the main endpoints for the polygon.
        if r.dst_i == results.intersection_id {
            endpts.push(r.center_line.shift_right(r.half_width()).ok()?.last_pt());
            endpts.push(r.center_line.shift_left(r.half_width()).ok()?.last_pt());
        } else {
            endpts.push(r.center_line.shift_left(r.half_width()).ok()?.first_pt());
            endpts.push(r.center_line.shift_right(r.half_width()).ok()?.first_pt());
        }
    }
    endpts.push(endpts[0]);

    results.intersection_polygon = Ring::deduping_new(endpts).ok()?.into_polygon();
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
