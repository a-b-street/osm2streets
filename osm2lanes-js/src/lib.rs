use wasm_bindgen::prelude::*;

use abstutil::Tags;
use osm2lanes::{MapConfig, get_lane_specs_ltr};

#[wasm_bindgen(js_name = getLaneSpecs)]
pub fn get_lane_specs(tags: JsValue, config: JsValue) -> Result<String, JsValue> {
    abstutil::logger::setup();
    // Panics shouldn't happen, but if they do, console.log them.
    console_error_panic_hook::set_once();

    let tags: Tags = serde_wasm_bindgen::from_value(tags)?;
    let config: MapConfig = serde_wasm_bindgen::from_value(config)?;

    Ok(serde_json::to_string_pretty(&get_lane_specs_ltr(&tags, &config)).unwrap())
}
