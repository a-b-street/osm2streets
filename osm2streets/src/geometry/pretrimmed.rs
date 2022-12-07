use std::collections::BTreeMap;

use anyhow::Result;

use geom::{Pt2D, Ring};

use super::Results;
use crate::{InputRoad, RoadID};

/// If we previously collapsed a short road, we recorded where adjacent roads got trimmed to. If
/// we're later producing geometry there, don't trim to corners again. Just use the pretrimmed
/// lines.
pub fn pretrimmed_geometry(
    mut results: Results,
    mut roads: BTreeMap<RoadID, InputRoad>,
    sorted_roads: Vec<RoadID>,
    trim_roads_for_merging: &BTreeMap<(RoadID, bool), Pt2D>,
) -> Result<Results> {
    // Use the previous trim values
    for road in roads.values_mut() {
        if let Some(endpt) =
            trim_roads_for_merging.get(&(road.id, road.src_i == results.intersection_id))
        {
            if road.src_i == results.intersection_id {
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
                assert_eq!(road.dst_i, results.intersection_id);
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

    // TODO Use a general procedure based on RoadEdge. Maybe include original corners, from
    // trim_to_corners
    let mut endpts = Vec::new();
    for r in sorted_roads {
        let r = &roads[&r];
        // Shift those centers out again to find the main endpoints for the polygon.
        if r.dst_i == results.intersection_id {
            endpts.push(r.center_line.shift_right(r.half_width())?.last_pt());
            endpts.push(r.center_line.shift_left(r.half_width())?.last_pt());
        } else {
            endpts.push(r.center_line.shift_left(r.half_width())?.first_pt());
            endpts.push(r.center_line.shift_right(r.half_width())?.first_pt());
        }
    }
    endpts.push(endpts[0]);

    results.intersection_polygon = Ring::deduping_new(endpts)?.into_polygon();
    for (id, r) in roads {
        results.trimmed_center_pts.insert(id, r.center_line);
    }
    Ok(results)
}
