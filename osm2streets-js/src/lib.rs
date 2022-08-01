use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct Input {
    driving_side: street_network::DrivingSide,
}

#[wasm_bindgen(js_name = importOsm)]
pub fn import_osm(osm_xml_input: &str, input: &JsValue) -> Result<String, JsValue> {
    // Panics shouldn't happen, but if they do, console.log them.
    console_error_panic_hook::set_once();

    inner_import_osm(osm_xml_input, input).map_err(|err| JsValue::from_str(&err.to_string()))
}

fn inner_import_osm(osm_xml_input: &str, input: &JsValue) -> anyhow::Result<String> {
    let input: Input = input.into_serde()?;

    let clip_pts = None;
    let mut timer = abstutil::Timer::throwaway();
    let mut street_network = import_streets::osm_to_street_network(
        osm_xml_input,
        clip_pts,
        import_streets::Options::default_for_side(input.driving_side),
        &mut timer,
    )?;
    street_network.apply_transformations(
        // TODO Assuming defaults here; probably do take in Input
        street_network::Transformation::standard_for_clipped_areas(),
        &mut timer,
    );

    // TODO Return the object and call methods on that instead
    let geojson = street_network.to_geojson(&mut timer)?;
    Ok(geojson)
}
