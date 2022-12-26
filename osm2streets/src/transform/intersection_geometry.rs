use abstutil::Timer;

use crate::StreetNetwork;

pub fn generate(streets: &mut StreetNetwork, timer: &mut Timer) {
    timer.start_iter(
        "find each intersection polygon",
        streets.intersections.len(),
    );

    let ids = streets.intersections.keys().cloned().collect::<Vec<_>>();
    for i in ids {
        timer.next();
        streets.update_geometry(i);
    }
}
