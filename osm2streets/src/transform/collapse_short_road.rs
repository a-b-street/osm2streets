use std::collections::VecDeque;

use crate::{RoadID, StreetNetwork};

/// Collapse all roads marked with `junction=intersection`
pub fn collapse_all_junction_roads(streets: &mut StreetNetwork) {
    let mut queue: VecDeque<RoadID> = VecDeque::new();
    for (id, road) in &streets.roads {
        if road.internal_junction_road {
            queue.push_back(*id);
        }
    }

    let mut i = 0;
    while !queue.is_empty() {
        let id = queue.pop_front().unwrap();
        i += 1;
        if !streets.roads.contains_key(&id) {
            continue;
        }
        streets.maybe_start_debug_step(format!("collapse road {i}"));
        streets.debug_road(id, "collapse");
        if let Err(err) = streets.collapse_short_road(id) {
            warn!("Not collapsing short road / junction=intersection: {}", err);
        }
    }
}
