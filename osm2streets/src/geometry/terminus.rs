use anyhow::Result;
use geom::{Distance, Ring};

use super::Results;
use crate::{InputRoad, IntersectionKind};

/// For dead-ends and map edges, just use a piece of the road as the intersection.
pub(crate) fn terminus(
    mut results: Results,
    road: InputRoad,
    kind: IntersectionKind,
) -> Result<Results> {
    // Point at the intersection, to simplify logic below
    let mut center = road.center_line_pointed_at(results.intersection_id);

    let intersection_len = if kind == IntersectionKind::MapEdge {
        // Arbitrarily require the rest of the road to be at least this long, before trimming
        let min_road_len = 3.0 * road.total_width;

        // If the road is too short, extend it. Depending on the boundary, MapEdges can become way
        // too small otherwise.
        //
        // 1) This kind of makes sense for a MapEdge, but is weird to do for a Terminus.
        // 2) We might've trimmed the other side of this road already or not. If we haven't, we might
        //    trim too much later and still wind up with something too short.
        if center.length() < min_road_len {
            center = center.extend_to_length(min_road_len);
        }

        road.total_width
    } else {
        // Make the intersection roughly square if possible
        if center.length() > road.total_width + Distance::meters(1.0) {
            road.total_width
        } else {
            0.4 * center.length()
        }
    };

    // Before trimming, remember the left and right endpoint.
    // TODO This logic isn't idempotent; it assumes the center_line starts untrimmed.
    let mut endpts = vec![
        center.shift_left(road.half_width())?.last_pt(),
        center.shift_right(road.half_width())?.last_pt(),
    ];

    // Trim
    center = center.maybe_exact_slice(Distance::ZERO, center.length() - intersection_len)?;

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
