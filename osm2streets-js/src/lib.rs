use abstutil::Timer;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct ImportOptions {
    driving_side: street_network::DrivingSide,
    debug_each_step: bool,
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
        // TODO Assuming defaults here; probably do take in Input
        let transformations = street_network::Transformation::standard_for_clipped_areas();
        if input.debug_each_step {
            street_network.apply_transformations_stepwise_debugging(transformations, &mut timer);
        } else {
            street_network.apply_transformations(transformations, &mut timer);
        }

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

    #[wasm_bindgen(js_name = toGraphviz)]
    pub fn to_graphviz(&self) -> String {
        // TODO Should we make the caller do the clone? Is that weird from JS?
        let road_network: streets::RoadNetwork = self.inner.clone().into();
        road_network.to_dot()
    }

    #[wasm_bindgen(js_name = getDebugSteps)]
    pub fn get_debug_steps(&self) -> Vec<JsValue> {
        // TODO Figure out how to borrow from the RefCell instead of cloning
        self.inner
            .debug_steps
            .borrow()
            .iter()
            .map(|x| JsValue::from(JsDebugStreets { inner: x.clone() }))
            .collect()
    }
}

#[wasm_bindgen]
pub struct JsDebugStreets {
    inner: street_network::DebugStreets,
}

#[wasm_bindgen]
impl JsDebugStreets {
    // TODO Can we borrow?
    #[wasm_bindgen(js_name = getLabel)]
    pub fn get_label(&self) -> String {
        self.inner.label.clone()
    }

    #[wasm_bindgen(js_name = getNetwork)]
    pub fn get_network(&self) -> JsValue {
        JsValue::from(JsStreetNetwork {
            inner: self.inner.streets.clone(),
        })
    }

    #[wasm_bindgen(js_name = toDebugGeojson)]
    pub fn to_debug_geojson(&self) -> Option<String> {
        self.inner.to_debug_geojson()
    }
}
