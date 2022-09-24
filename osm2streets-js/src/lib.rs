use abstutil::Timer;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use osm2streets::{DebugStreets, DrivingSide, StreetNetwork, Transformation};

#[derive(Serialize, Deserialize)]
pub struct ImportOptions {
    driving_side: DrivingSide,
    debug_each_step: bool,
    dual_carriageway_experiment: bool,
    cycletrack_snapping_experiment: bool,
    inferred_sidewalks: bool,
    osm2lanes: bool,
}

#[wasm_bindgen]
pub struct JsStreetNetwork {
    inner: StreetNetwork,
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

        let mut options = streets_reader::Options::default_for_side(input.driving_side);
        options.map_config.inferred_sidewalks = input.inferred_sidewalks;
        options.map_config.osm2lanes = input.osm2lanes;

        let clip_pts = None;
        let mut timer = Timer::throwaway();
        let mut street_network =
            streets_reader::osm_to_street_network(osm_xml_input, clip_pts, options, &mut timer)
                .map_err(|err| JsValue::from_str(&err.to_string()))?;
        let mut transformations = Transformation::standard_for_clipped_areas();
        if input.dual_carriageway_experiment {
            // Merging short roads tries to touch "bridges," making debugging harder
            transformations.retain(|t| !matches!(t, Transformation::MergeShortRoads));
            transformations.push(Transformation::MergeDualCarriageways);
        }
        if input.cycletrack_snapping_experiment {
            transformations.push(Transformation::SnapCycleways);
            transformations.push(Transformation::TrimDeadendCycleways);
            transformations.push(Transformation::CollapseDegenerateIntersections);
        }
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

    #[wasm_bindgen(js_name = toLanePolygonsGeojson)]
    pub fn to_lane_polygons_geojson(&self) -> String {
        self.inner
            .to_lane_polygons_geojson(&mut Timer::throwaway())
            .unwrap()
    }

    #[wasm_bindgen(js_name = toLaneMarkingsGeojson)]
    pub fn to_lane_markings_geojson(&self) -> String {
        self.inner
            .to_lane_markings_geojson(&mut Timer::throwaway())
            .unwrap()
    }

    #[wasm_bindgen(js_name = toGraphviz)]
    pub fn to_graphviz(&self) -> String {
        // TODO Should we make the caller do the clone? Is that weird from JS?
        let road_network: experimental::RoadNetwork = self.inner.clone().into();
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
    inner: DebugStreets,
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
