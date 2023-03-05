use abstutil::Timer;

use crate::StreetNetwork;

/// Collapse all roads marked with `junction=intersection`
pub fn collapse_all_junction_roads(streets: &mut StreetNetwork, timer: &mut Timer) {
    let mut queue = Vec::new();
    for (id, road) in &streets.roads {
        if road.internal_junction_road {
            queue.push(*id);
        }
    }

    timer.start_iter("collapse short roads", queue.len());
    for (idx, id) in queue.into_iter().enumerate() {
        timer.next();
        if !streets.roads.contains_key(&id) {
            continue;
        }
        streets.maybe_start_debug_step(format!("collapse road {idx}"));
        streets.debug_road(id, "collapse");
        if let Err(err) = streets.collapse_short_road(id) {
            warn!("Not collapsing short road / junction=intersection: {}", err);
        }
    }
}
