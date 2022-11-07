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

    #[wasm_bindgen(js_name = debugClockwiseOrderingGeojson)]
    pub fn debug_clockwise_ordering_geojson(&self) -> String {
        self.inner
            .debug_clockwise_ordering_geojson(&mut Timer::throwaway())
            .unwrap()
    }

    #[wasm_bindgen(js_name = snap)]
    pub fn snap(&self) -> String {
        do_snap(&self.inner)
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

fn do_snap(streets: &StreetNetwork) -> String {
    use geom::{Circle, Distance, FindClosest, Line, PolyLine, Polygon};
    use osm2streets::LaneType;

    let input = r###"{"type": "FeatureCollection", "features": [ { "type": "Feature", "properties": {}, "geometry": { "coordinates": [ [ [-0.11145331549823823, 51.48936377244081], [-0.11588958774200364, 51.49078063223098], [-0.11688893065218053, 51.48951092609991], [-0.11545743945600861, 51.48908628139077], [-0.11308737620268516, 51.488119253928176], [-0.11235137365375181, 51.489027419435786], [-0.11150058171699584, 51.48899798842976], [-0.11145331549823823, 51.48936377244081] ] ], "type": "Polygon" } } ]}"###;
    let require_in_bounds = false;
    let ring =
        Polygon::from_geojson_bytes(input.as_bytes(), &streets.gps_bounds, require_in_bounds)
            .unwrap()[0]
            .0
            .clone()
            .into_outer_ring();

    let mut snap_to_intersections = FindClosest::new(&streets.gps_bounds.to_bounds());
    for (id, i) in &streets.intersections {
        // Just focus on vehicle roads right now
        if i.roads.iter().all(|r| !streets.roads[r].is_driveable()) {
            continue;
        }

        // TODO Time to rethink FindClosest. It can't handle a single point, it needs something
        // with a real bbox
        snap_to_intersections.add_polygon(
            *id,
            &Circle::new(i.point, Distance::meters(1.0)).to_polygon(),
        );
    }

    // Algorithm idea: for each point in the input, snap to the closest intersection. Then pathfind
    // between those.
    let threshold = Distance::meters(50.0);

    let mut debug_out = Vec::new();
    debug_out.push((ring.to_geojson(Some(&streets.gps_bounds)), "red"));

    let mut intersection_waypoints = Vec::new();
    for pt in ring.points() {
        if let Some((id, snapped_pt)) = snap_to_intersections.closest_pt(*pt, threshold) {
            intersection_waypoints.push(id);

            // Visualize that snapping...
            if let Ok(line) = Line::new(*pt, snapped_pt) {
                debug_out.push((
                    line.make_polygons(Distance::meters(2.0))
                        .to_geojson(Some(&streets.gps_bounds)),
                    "blue",
                ));
            }
        }
    }

    // Now pathfind between each pair of waypoints
    for pair in intersection_waypoints.windows(2) {
        if let Some(path) = streets.simple_path(pair[0], pair[1], &[LaneType::Driving]) {
            for (r, _) in path {
                debug_out.push((
                    PolyLine::unchecked_new(streets.roads[&r].osm_center_points.clone())
                        .make_polygons(Distance::meters(5.0))
                        .to_geojson(Some(&streets.gps_bounds)),
                    "green",
                ));
            }
        }
    }

    let debug_out = debug_out
        .into_iter()
        .map(|(geometry, color)| {
            let mut props = serde_json::Map::new();
            props.insert("fill".to_string(), color.into());
            props.insert("fillOpacity".to_string(), 0.5.into());
            (geometry, props)
        })
        .collect::<Vec<_>>();
    abstutil::to_json(&geom::geometries_with_properties_to_geojson(debug_out))
}
