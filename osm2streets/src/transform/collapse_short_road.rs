use std::collections::VecDeque;

use crate::{Debugger, RoadID, StreetNetwork};

/// Collapse all roads marked with `junction=intersection`
pub fn collapse_all_junction_roads(streets: &mut StreetNetwork, debugger: &mut Debugger) {
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
        debugger.start_debug_step(streets, format!("collapse road {i}"));
        debugger.debug_road(id, "collapse");
        if let Err(err) = streets.collapse_short_road(id) {
            warn!("Not collapsing short road / junction=intersection: {}", err);
        }
    }
}
