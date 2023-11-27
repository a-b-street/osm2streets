use std::sync::Once;

use wasm_bindgen::prelude::*;

use abstutil::Tags;
use osm2lanes::{get_lane_specs_ltr, MapConfig};

static SETUP_LOGGER: Once = Once::new();

#[wasm_bindgen(js_name = getLaneSpecs)]
pub fn get_lane_specs(tags: JsValue, config: JsValue) -> Result<String, JsValue> {
    SETUP_LOGGER.call_once(|| console_log::init_with_level(log::Level::Info).unwrap());
    // Panics shouldn't happen, but if they do, console.log them.
    console_error_panic_hook::set_once();

    let tags: Tags = serde_wasm_bindgen::from_value(tags)?;
    let config: MapConfig = serde_wasm_bindgen::from_value(config)?;

    Ok(serde_json::to_string_pretty(&get_lane_specs_ltr(&tags, &config)).unwrap())
}
