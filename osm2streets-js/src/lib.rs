use std::collections::BTreeMap;

use abstutil::{Tags, Timer};
use geom::PolyLine;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use osm2streets::{osm, DebugStreets, MapConfig, StreetNetwork, Transformation};

#[derive(Serialize, Deserialize)]
pub struct ImportOptions {
    debug_each_step: bool,
    dual_carriageway_experiment: bool,
    cycletrack_snapping_experiment: bool,
    inferred_sidewalks: bool,
    osm2lanes: bool,
}

#[wasm_bindgen]
pub struct JsStreetNetwork {
    inner: StreetNetwork,
    ways: BTreeMap<osm::WayID, streets_reader::osm_reader::Way>,
}

#[wasm_bindgen]
impl JsStreetNetwork {
    #[wasm_bindgen(constructor)]
    pub fn new(osm_xml_input: &str, input: &JsValue) -> Result<JsStreetNetwork, JsValue> {
        abstutil::logger::setup();
        // Panics shouldn't happen, but if they do, console.log them.
        console_error_panic_hook::set_once();

        let input: ImportOptions = input
            .into_serde()
            .map_err(|err| JsValue::from_str(&err.to_string()))?;

        let mut cfg = MapConfig::default();
        cfg.inferred_sidewalks = input.inferred_sidewalks;
        cfg.osm2lanes = input.osm2lanes;

        let clip_pts = None;
        let mut timer = Timer::throwaway();
        let (mut street_network, doc) =
            streets_reader::osm_to_street_network(osm_xml_input, clip_pts, cfg, &mut timer)
                .map_err(|err| JsValue::from_str(&err.to_string()))?;
        let mut transformations = Transformation::standard_for_clipped_areas();
        if input.dual_carriageway_experiment {
            // Collapsing short roads tries to touch "bridges," making debugging harder
            transformations.retain(|t| !matches!(t, Transformation::CollapseShortRoads));
            transformations.push(Transformation::MergeDualCarriageways);
        }
        if input.cycletrack_snapping_experiment {
            transformations.push(Transformation::SnapCycleways);
            transformations.push(Transformation::TrimDeadendCycleways);
            transformations.push(Transformation::CollapseDegenerateIntersections);
            // TODO Indeed it'd be much nicer to recalculate this as the above transformations
            // modify things
            transformations.push(Transformation::GenerateIntersectionGeometry);
        }
        if input.debug_each_step {
            street_network.apply_transformations_stepwise_debugging(transformations, &mut timer);

            // For all but the last step, generate intersection geometry, so these
            // intermediate states can be rendered.
            // TODO Revisit this -- rendering should use untrimmed geometry, or an initial guess of
            // trimmed geometry.
            let mut steps = street_network.debug_steps.borrow_mut();
            for i in 0..steps.len() - 1 {
                steps[i].streets.apply_transformations(
                    vec![Transformation::GenerateIntersectionGeometry],
                    &mut timer,
                );
            }
        } else {
            street_network.apply_transformations(transformations, &mut timer);
        }

        Ok(Self {
            inner: street_network,
            ways: doc.ways,
        })
    }
    #[wasm_bindgen(js_name = toGeojsonPlain)]
    pub fn to_geojson_plain(&self) -> String {
        self.inner.to_geojson().unwrap()
    }

    #[wasm_bindgen(js_name = toLanePolygonsGeojson)]
    pub fn to_lane_polygons_geojson(&self) -> String {
        self.inner.to_lane_polygons_geojson().unwrap()
    }

    #[wasm_bindgen(js_name = toLaneMarkingsGeojson)]
    pub fn to_lane_markings_geojson(&self) -> String {
        self.inner.to_lane_markings_geojson().unwrap()
    }

    #[wasm_bindgen(js_name = toIntersectionMarkingsGeojson)]
    pub fn to_intersection_markings_geojson(&self) -> String {
        self.inner.to_intersection_markings_geojson().unwrap()
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

    #[wasm_bindgen(js_name = debugClockwiseOrderingGeojson)]
    pub fn debug_clockwise_ordering_geojson(&self) -> String {
        self.inner.debug_clockwise_ordering_geojson().unwrap()
    }

    #[wasm_bindgen(js_name = debugMovementsGeojson)]
    pub fn debug_movements_geojson(&self) -> String {
        self.inner.debug_movements_geojson().unwrap()
    }

    // TODO I think https://github.com/cloudflare/serde-wasm-bindgen would let us just return a
    // HashMap
    #[wasm_bindgen(js_name = getOsmTagsForWay)]
    pub fn get_osm_tags_for_way(&self, id: i64) -> String {
        abstutil::to_json(&self.ways[&osm::WayID(id)].tags)
    }

    /// Returns a GeoJSON Polygon showing a wide buffer around the way's original geometry
    #[wasm_bindgen(js_name = getGeometryForWay)]
    pub fn get_geometry_for_way(&self, id: i64) -> String {
        let id = osm::WayID(id);

        // The lanes, and thus width, will be the same for every road belonging to the way
        let width = self
            .inner
            .roads
            .values()
            .find(|r| r.osm_ids.iter().any(|x| x.osm_way_id == id))
            .map(|r| r.total_width())
            .unwrap();

        // Show a wide buffer around the way
        let polygon =
            PolyLine::unchecked_new(self.ways[&id].pts.clone()).make_polygons(1.5 * width);

        abstutil::to_json(&polygon.to_geojson(Some(&self.inner.gps_bounds)))
    }

    /// Modifies all affected roads and only reruns `Transformation::GenerateIntersectionGeometry`.
    #[wasm_bindgen(js_name = overwriteOsmTagsForWay)]
    pub fn overwrite_osm_tags_for_way(&mut self, id: i64, tags: String) {
        let id = osm::WayID(id);
        let tags: Tags = abstutil::from_json(tags.as_bytes()).unwrap();

        for road in self.inner.roads.values_mut() {
            if road.osm_ids.iter().any(|x| x.osm_way_id == id) {
                // TODO This could panic, for example if the user removes the highway tag
                road.lane_specs_ltr = osm2streets::get_lane_specs_ltr(&tags, &self.inner.config);
            }
        }
        self.inner.apply_transformations(
            vec![Transformation::GenerateIntersectionGeometry],
            &mut Timer::throwaway(),
        );

        self.ways.get_mut(&id).unwrap().tags = tags;
    }

    /// Returns the XML string representing a way. Any OSM tags changed via
    /// `overwrite_osm_tags_for_way` are reflected.
    #[wasm_bindgen(js_name = wayToXml)]
    pub fn way_to_xml(&mut self, id: i64) -> String {
        let way = &self.ways[&osm::WayID(id)];
        let mut out = format!(r#"<way id="{id}""#);
        if let Some(version) = way.version {
            out.push_str(&format!(r#" version="{version}""#));
        }
        out.push_str(">\n");
        for node in &way.nodes {
            out.push_str(&format!(r#"  <nd ref="{}"/>"#, node.0));
            out.push('\n');
        }
        for (k, v) in way.tags.inner() {
            out.push_str(&format!(r#"  <tag k="{k}" v="{v}"/>"#));
            out.push('\n');
        }
        out.push_str("</way>");
        out
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
            ways: BTreeMap::new(),
        })
    }

    #[wasm_bindgen(js_name = toDebugGeojson)]
    pub fn to_debug_geojson(&self) -> Option<String> {
        self.inner.to_debug_geojson()
    }
}
