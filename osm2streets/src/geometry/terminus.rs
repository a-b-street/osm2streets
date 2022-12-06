use std::collections::BTreeMap;

use anyhow::Result;
use geom::{Distance, Ring, EPSILON_DIST};

use super::{close_off_polygon, Results, RoadLine, DEGENERATE_INTERSECTION_HALF_LENGTH};
use crate::{InputRoad, RoadID};

pub(crate) fn terminus(
    mut results: Results,
    mut roads: BTreeMap<RoadID, InputRoad>,
    road_lines: &[RoadLine],
) -> Result<Results> {
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

    let r = roads.get_mut(&id).unwrap();
    let len_with_buffer = len + 3.0 * EPSILON_DIST;
    let trimmed = if r.center_line.length() >= len_with_buffer {
        if r.src_i == results.intersection_id {
            r.center_line = r.center_line.exact_slice(len, r.center_line.length());
        } else {
            r.center_line = r
                .center_line
                .exact_slice(Distance::ZERO, r.center_line.length() - len);
        }
        r.center_line.clone()
    } else if r.src_i == results.intersection_id {
        r.center_line.extend_to_length(len_with_buffer)
    } else {
        r.center_line
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
    for (id, r) in roads {
        results.trimmed_center_pts.insert(id, r.center_line);
    }
    Ok(results)
}
