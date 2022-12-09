use anyhow::Result;
use geom::{Distance, Ring};

use super::Results;
use crate::InputRoad;

/// For dead-ends and map edges, just use a piece of the road as the intersection.
pub(crate) fn terminus(mut results: Results, road: InputRoad) -> Result<Results> {
    // Point at the intersection, to simplify logic below
    let mut center = road.center_line_pointed_at(results.intersection_id);

    // Make the intersection roughly square
    let intersection_len = road.total_width * 0.1;
    // Arbitrarily require the rest of the road to be at least this long, before trimming
    let min_road_len = 3.0 * intersection_len;

    // If the road is too short, extend it. Two caveats:
    //
    // 1) This kind of makes sense for a MapEdge, but is weird to do for a Terminus.
    // 2) We have no idea what'll happen to the other side. We could still wind up with something
    //    too short.
    if center.length() < min_road_len {
        center = center.extend_to_length(min_road_len);
    }

    // Before trimming, remember the left and right endpoint.
    // TODO This logic isn't idempotent; it assumes the center_line starts untrimmed.
    let mut endpts = vec![
        center.shift_left(road.half_width())?.last_pt(),
        center.shift_right(road.half_width())?.last_pt(),
    ];

    // Trim
    center = center.exact_slice(Distance::ZERO, center.length() - intersection_len);

    // Make the square polygon
    endpts.push(center.shift_right(road.half_width())?.last_pt());
    endpts.push(center.shift_left(road.half_width())?.last_pt());
    endpts.push(endpts[0]);

    results.intersection_polygon = Ring::deduping_new(endpts)?.into_polygon();

    // Fix orientation if needed
    if road.src_i == results.intersection_id {
        center = center.reversed();
    }

    results.trimmed_center_pts.insert(road.id, center);
    Ok(results)
}
