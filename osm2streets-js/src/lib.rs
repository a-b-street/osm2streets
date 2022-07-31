use abstutil::Timer;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct ImportOptions {
    driving_side: street_network::DrivingSide,
}

#[wasm_bindgen]
pub struct JsStreetNetwork {
    inner: street_network::StreetNetwork,
}

#[wasm_bindgen]
impl JsStreetNetwork {
    #[wasm_bindgen(constructor)]
    pub fn new(osm_xml_input: &str, input: &JsValue) -> Result<JsStreetNetwork, JsValue> {
        // Panics shouldn't happen, but if they do, console.log them.
        console_error_panic_hook::set_once();

        let input: ImportOptions = input
            .into_serde()
            .map_err(|err| JsValue::from_str(&err.to_string()))?;

        let clip_pts = None;
        let mut timer = Timer::throwaway();
        let mut street_network = import_streets::osm_to_street_network(
            osm_xml_input,
            clip_pts,
            import_streets::Options::default_for_side(input.driving_side),
            &mut timer,
        )
        .map_err(|err| JsValue::from_str(&err.to_string()))?;
        street_network.apply_transformations(
            // TODO Assuming defaults here; probably do take in Input
            street_network::Transformation::standard_for_clipped_areas(),
            &mut timer,
        );

        Ok(Self {
            inner: street_network,
        })
    }
    #[wasm_bindgen(js_name = toGeojsonPlain)]
    pub fn to_geojson_plain(&self) -> String {
        self.inner.to_geojson(&mut Timer::throwaway()).unwrap()
    }

    #[wasm_bindgen(js_name = toGeojsonDetailed)]
    pub fn to_geojson_detailed(&self) -> String {
        self.inner
            .to_detailed_geojson(&mut Timer::throwaway())
            .unwrap()
    }
}
