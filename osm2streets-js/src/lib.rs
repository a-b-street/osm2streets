use std::collections::{BTreeMap, BTreeSet};
use std::sync::Once;

use abstutil::{Tags, Timer};
use geom::{Distance, LonLat, PolyLine, Polygon};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use osm2streets::{
    osm, DebugStreets, Filter, IntersectionID, LaneID, MapConfig, Placement, RoadID, Sidepath,
    StreetNetwork, Transformation,
};

static SETUP_LOGGER: Once = Once::new();

#[derive(Serialize, Deserialize)]
pub struct ImportOptions {
    debug_each_step: bool,
    dual_carriageway_experiment: bool,
    sidepath_zipping_experiment: bool,
    inferred_sidewalks: bool,
}

#[wasm_bindgen]
pub struct JsStreetNetwork {
    inner: StreetNetwork,
    ways: BTreeMap<osm::WayID, streets_reader::osm_reader::Way>,
}

#[wasm_bindgen]
impl JsStreetNetwork {
    // TODO clip_pts_geojson should be Option. Empty means None.
    #[wasm_bindgen(constructor)]
    pub fn new(
        osm_input: &[u8],
        clip_pts_geojson: &str,
        input: JsValue,
    ) -> Result<JsStreetNetwork, JsValue> {
        SETUP_LOGGER.call_once(|| console_log::init_with_level(log::Level::Info).unwrap());
        // Panics shouldn't happen, but if they do, console.log them.
        console_error_panic_hook::set_once();

        let input: ImportOptions = serde_wasm_bindgen::from_value(input)?;

        let clip_pts = if clip_pts_geojson.is_empty() {
            None
        } else {
            let mut list = LonLat::parse_geojson_polygons(clip_pts_geojson.to_string())
                .map_err(|err| JsValue::from_str(&err.to_string()))?;
            if list.len() != 1 {
                return Err(JsValue::from_str(&format!(
                    "{clip_pts_geojson} doesn't contain exactly one polygon"
                )));
            }
            Some(list.pop().unwrap().0)
        };

        let mut cfg = MapConfig::default();
        cfg.inferred_sidewalks = input.inferred_sidewalks;

        let mut timer = Timer::throwaway();
        let (mut street_network, doc) =
            streets_reader::osm_to_street_network(osm_input, clip_pts, cfg, &mut timer)
                .map_err(|err| JsValue::from_str(&err.to_string()))?;
        let mut transformations = Transformation::standard_for_clipped_areas();
        if input.dual_carriageway_experiment {
            // Collapsing short roads tries to touch "bridges," making debugging harder
            transformations.retain(|t| !matches!(t, Transformation::CollapseShortRoads));
            transformations.push(Transformation::MergeDualCarriageways);
        }
        if input.sidepath_zipping_experiment {
            transformations.push(Transformation::ZipSidepaths);
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
            ways: doc.ways,
        })
    }
    #[wasm_bindgen(js_name = toGeojsonPlain)]
    pub fn to_geojson_plain(&self) -> String {
        self.inner.to_geojson(&Filter::All).unwrap()
    }

    #[wasm_bindgen(js_name = toLanePolygonsGeojson)]
    pub fn to_lane_polygons_geojson(&self) -> String {
        self.inner.to_lane_polygons_geojson(&Filter::All).unwrap()
    }

    #[wasm_bindgen(js_name = toLaneMarkingsGeojson)]
    pub fn to_lane_markings_geojson(&self) -> String {
        self.inner.to_lane_markings_geojson(&Filter::All).unwrap()
    }

    #[wasm_bindgen(js_name = toIntersectionMarkingsGeojson)]
    pub fn to_intersection_markings_geojson(&self) -> String {
        self.inner
            .to_intersection_markings_geojson(&Filter::All)
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
        self.inner
            .debug_steps
            .iter()
            .map(|x| JsValue::from(JsDebugStreets { inner: x.clone() }))
            .collect()
    }

    #[wasm_bindgen(js_name = debugClockwiseOrderingGeojson)]
    pub fn debug_clockwise_ordering_geojson(&self) -> String {
        self.inner
            .debug_clockwise_ordering_geojson(&Filter::All)
            .unwrap()
    }

    // TODO Can we take Filter as input here?
    #[wasm_bindgen(js_name = debugClockwiseOrderingForIntersectionGeojson)]
    pub fn debug_clockwise_ordering_for_intersection_geojson(&self, intersection: usize) -> String {
        let mut intersections = BTreeSet::new();
        intersections.insert(IntersectionID(intersection));
        self.inner
            .debug_clockwise_ordering_geojson(&Filter::Filtered(BTreeSet::new(), intersections))
            .unwrap()
    }

    #[wasm_bindgen(js_name = debugMovementsFromLaneGeojson)]
    pub fn debug_movements_from_lane_geojson(&self, road: usize, index: usize) -> String {
        self.inner
            .debug_movements_from_lane_geojson(LaneID {
                road: RoadID(road),
                index,
            })
            .unwrap()
    }

    #[wasm_bindgen(js_name = debugRoadsConnectedToIntersectionGeojson)]
    pub fn debug_roads_connected_to_intersection_geojson(&self, i: usize) -> String {
        let mut polygons = Vec::new();
        for r in &self.inner.intersections[&IntersectionID(i)].roads {
            let road = &self.inner.roads[r];
            polygons.push(
                road.center_line
                    .make_polygons(road.total_width())
                    .to_geojson(Some(&self.inner.gps_bounds)),
            );
        }
        serde_json::to_string_pretty(&geom::geometries_to_geojson(polygons)).unwrap()
    }

    // TODO I think https://github.com/cloudflare/serde-wasm-bindgen would let us just return a
    // HashMap
    #[wasm_bindgen(js_name = getOsmTagsForWay)]
    pub fn get_osm_tags_for_way(&self, id: i64) -> String {
        serde_json::to_string_pretty(&self.ways[&osm::WayID(id)].tags).unwrap()
    }

    /// Returns the entire StreetNetwork as JSON. The API doesn't have guarantees about backwards
    /// compatibility.
    #[wasm_bindgen(js_name = toJson)]
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.inner).unwrap()
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
            .find(|r| r.from_osm_way(id))
            .map(|r| r.total_width())
            .unwrap();

        let polyline = PolyLine::unchecked_new(self.ways[&id].pts.clone());

        // Show a wide buffer around the way
        let mut polygon = polyline.make_polygons(1.5 * width);

        // "Cut out" one or more chevrons from the polygon to indicate the direction
        let num_chevrons = std::cmp::max(
            1,
            (polyline.length() / Distance::meters(50.0)).floor() as i64,
        );
        let chevrons = (1..=num_chevrons)
            .map(|i| {
                let (top_pt, angle) = polyline
                    // chevrons are uniformly spread along the polyline
                    .dist_along((i as f64 / (num_chevrons as f64 + 1.0)) * polyline.length())
                    .unwrap();
                PolyLine::must_new(vec![
                    top_pt.project_away(width / 2.0, angle.rotate_degs(135.0)),
                    top_pt,
                    top_pt.project_away(width / 2.0, angle.rotate_degs(-135.0)),
                ])
                .make_polygons(width * 0.2)
            })
            .collect::<Vec<Polygon>>();

        chevrons
            .iter()
            .for_each(|c| polygon = polygon.difference(c).unwrap()[0].clone());

        serde_json::to_string_pretty(&polygon.to_geojson(Some(&self.inner.gps_bounds))).unwrap()
    }

    /// Returns the XML string representing a way. Any OSM tags changed via
    /// `overwrite_osm_tags_for_way` are reflected.
    #[wasm_bindgen(js_name = wayToXml)]
    pub fn way_to_xml(&self, id: i64) -> String {
        let way = &self.ways[&osm::WayID(id)];
        let mut out = format!(r#"<way id="{id}""#);
        // TODO Add this to osm-reader
        /*if let Some(version) = way.version {
            out.push_str(&format!(r#" version="{version}""#));
        }*/
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

// Mutations
#[wasm_bindgen]
impl JsStreetNetwork {
    /// Modifies all affected roads
    #[wasm_bindgen(js_name = overwriteOsmTagsForWay)]
    pub fn overwrite_osm_tags_for_way(&mut self, id: i64, tags: String) {
        let id = osm::WayID(id);
        let tags: Tags = serde_json::from_slice(tags.as_bytes()).unwrap();

        let mut intersections = BTreeSet::new();
        for road in self.inner.roads.values_mut() {
            if road.from_osm_way(id) {
                // Repeat some of the work in Road::new

                // TODO This could panic, for example if the user removes the highway tag
                road.lane_specs_ltr = osm2streets::get_lane_specs_ltr(&tags, &self.inner.config);
                intersections.extend(road.endpoints());

                // Silently fail
                if let Ok(p) = Placement::parse(&tags) {
                    road.reference_line_placement = p;
                }

                road.update_center_line(self.inner.config.driving_side);
            }
        }
        for i in intersections {
            self.inner.update_i(i);
        }

        self.ways.get_mut(&id).unwrap().tags = tags;
    }

    #[wasm_bindgen(js_name = collapseShortRoad)]
    pub fn collapse_short_road(&mut self, road: usize) {
        // TODO Handle errors how?
        self.inner.collapse_short_road(RoadID(road)).unwrap()
    }

    #[wasm_bindgen(js_name = collapseIntersection)]
    pub fn collapse_intersection(&mut self, intersection: usize) {
        let i = IntersectionID(intersection);
        if self.inner.intersections[&i].roads.len() == 2 {
            self.inner.collapse_intersection(i);
        }
    }

    #[wasm_bindgen(js_name = zipSidepath)]
    pub fn zip_sidepath(&mut self, road: usize) {
        if let Some(sidepath) = Sidepath::new(&self.inner, RoadID(road)) {
            sidepath.zip(&mut self.inner);
        }
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
