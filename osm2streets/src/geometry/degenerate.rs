use anyhow::Result;
use geom::{Distance, Ring};

use super::Results;
use crate::InputRoad;

/// For intersections between exactly 2 roads, just trim back a bit.
pub(crate) fn degenerate(
    mut results: Results,
    road1: InputRoad,
    road2: InputRoad,
) -> Result<Results> {
    // Arbitrary parameters
    let intersection_half_len = Distance::meters(1.0);
    let min_road_len = 2.0 * intersection_half_len;

    // Make both roads point at the intersection, to simplify logic below
    let mut center1 = road1.center_line_pointed_at(results.intersection_id);
    let mut center2 = road2.center_line_pointed_at(results.intersection_id);

    // If either road is too short, just fail outright. What else should we do?
    // TODO Also, if we haven't trimmed the other side yet, we don't have the full picture
    if center1.length() < min_road_len || center2.length() < min_road_len {
        bail!("Road is too short to trim for a degenerate intersection");
    }

    // Trim each.
    center1 = center1.exact_slice(Distance::ZERO, center1.length() - intersection_half_len);
    center2 = center2.exact_slice(Distance::ZERO, center2.length() - intersection_half_len);

    // Make the square polygon
    let mut endpts = vec![
        center1.shift_left(road1.half_width())?.last_pt(),
        center2.shift_right(road2.half_width())?.last_pt(),
        center2.shift_left(road2.half_width())?.last_pt(),
        center1.shift_right(road1.half_width())?.last_pt(),
    ];
    endpts.push(endpts[0]);

    results.intersection_polygon = Ring::deduping_new(endpts)?.into_polygon();

    // Fix orientation if needed
    if road1.src_i == results.intersection_id {
        center1 = center1.reversed();
    }
    if road2.src_i == results.intersection_id {
        center2 = center2.reversed();
    }

    results.trimmed_center_pts.insert(road1.id, center1);
    results.trimmed_center_pts.insert(road2.id, center2);
    Ok(results)
}
