use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct Input {
    driving_side: street_network::DrivingSide,
}

// TODO Just take a string and bool, maybe remove serde dependency?
#[wasm_bindgen]
pub fn import_osm(osm_xml_input: &str, val: &JsValue) -> String {
    set_panic_hook();

    let input: Input = val.into_serde().unwrap();

    let clip_path = None;
    let mut timer = abstutil::Timer::throwaway();
    let mut street_network = import_streets::osm_to_street_network(
        osm_xml_input,
        clip_path,
        import_streets::Options::default_for_side(input.driving_side),
        &mut timer,
    );
    // TODO Assuming defaults here; probably do take in Input
    let consolidate_all_intersections = false;
    let remove_disconnected = false;
    street_network.run_all_simplifications(
        consolidate_all_intersections,
        remove_disconnected,
        &mut timer,
    );

    // TODO Return GeoJSON, dot, etc. Or even better, the object, and expose methods on that

    "placeholder".to_string()
}

fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
