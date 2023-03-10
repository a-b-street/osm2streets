use crate::{Sidepath, StreetNetwork};

/// Find sidepath segments that exist as separate objects, parallel to a main road. Zip (or "snap")
/// them into the main road, inserting a buffer lane to represent the physical division.
pub fn zip_sidepaths(streets: &mut StreetNetwork) {
    let mut sidepaths = Vec::new();
    for r in streets.roads.values() {
        // TODO Or footpath
        if r.is_cycleway() {
            sidepaths.extend(Sidepath::new(streets, r.id));
        }
    }

    for (idx, sidepath) in sidepaths.into_iter().enumerate() {
        streets.maybe_start_debug_step(format!("snap sidepath {idx}"));
        sidepath.debug(streets, idx.to_string());
        sidepath.zip(streets);
    }
}
