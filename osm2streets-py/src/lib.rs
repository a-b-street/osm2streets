use std::collections::{BTreeMap, BTreeSet};
use std::sync::Once;

use abstutil::{Tags, Timer};
use chrono::NaiveDateTime;
use geom::{Distance, LonLat, PolyLine, Polygon};
use osm2streets::{
    osm, DebugStreets, DrivingSide, Filter, IntersectionID, LaneID, MapConfig, Placement, RoadID,
    RoadSideID, SideOfRoad, Sidepath, StreetNetwork, Transformation,
};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json; // Added serde_json import

static SETUP_LOGGER: Once = Once::new();

#[derive(Serialize, Deserialize)]
pub struct ImportOptions {
    debug_each_step: bool,
    dual_carriageway_experiment: bool,
    sidepath_zipping_experiment: bool,
    inferred_sidewalks: bool,
    inferred_kerbs: bool,
    date_time: Option<NaiveDateTime>,
    override_driving_side: String,
}

#[pyclass]
pub struct PyStreetNetwork {
    inner: StreetNetwork,
    ways: BTreeMap<osm::WayID, streets_reader::osm_reader::Way>,
}

#[pymethods]
impl PyStreetNetwork {
    /// Creates a new instance of `PyStreetNetwork`.
    ///
    /// - `osm_input`: Byte array representing OSM data input.
    /// - `clip_pts_geojson`: Optional GeoJSON string representing a polygon to clip the input data.
    /// - `input`: JSON string that sets configuration options for the import, including `debug_each_step`,
    ///   `dual_carriageway_experiment`, `sidepath_zipping_experiment`, `inferred_sidewalks`, `inferred_kerbs`,
    ///   `date_time`, and `override_driving_side`.
    #[new]
    pub fn new(
        py: Python, // Added `py: Python` here to get the Python context
        osm_input: &[u8],
        clip_pts_geojson: &str,
        input: PyObject,
    ) -> PyResult<Self> {
        SETUP_LOGGER.call_once(|| env_logger::init());

        let input: ImportOptions =
            serde_json::from_str(input.extract::<&str>(py)?).map_err(|e| {
                pyo3::exceptions::PyValueError::new_err(format!("Failed to parse input: {}", e))
            })?;

        // Parse clip points if provided
        let clip_pts = if clip_pts_geojson.is_empty() {
            None
        } else {
            let mut list = LonLat::parse_geojson_polygons(clip_pts_geojson.to_string())
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
            if list.len() != 1 {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "clip_pts_geojson must contain exactly one polygon",
                ));
            }
            Some(list.pop().unwrap().0)
        };

        let mut cfg = MapConfig::default();
        cfg.inferred_sidewalks = input.inferred_sidewalks;
        cfg.inferred_kerbs = input.inferred_kerbs;
        cfg.date_time = input.date_time;
        cfg.override_driving_side = match input.override_driving_side.as_str() {
            "" => None,
            "Left" => Some(DrivingSide::Left),
            "Right" => Some(DrivingSide::Right),
            x => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Unknown driving side: {x}"
                )))
            }
        };

        let mut timer = Timer::throwaway();
        let (mut street_network, doc) =
            streets_reader::osm_to_street_network(osm_input, clip_pts, cfg, &mut timer)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;

        let mut transformations = Transformation::standard_for_clipped_areas();
        if input.dual_carriageway_experiment {
            transformations.retain(|t| !matches!(t, Transformation::CollapseShortRoads));
            transformations.push(Transformation::MergeDualCarriageways);
        }
        if input.sidepath_zipping_experiment {
            transformations.push(Transformation::ZipSidepaths);
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

    /// Converts the entire `StreetNetwork` to a GeoJSON format.
    ///
    /// Returns a GeoJSON string representing all elements in the street network.
    pub fn to_geojson_plain(&self) -> PyResult<String> {
        self.inner
            .to_geojson(&Filter::All)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    /// Converts lane polygons in the `StreetNetwork` to a GeoJSON format.
    ///
    /// Returns a GeoJSON string representing the polygons of each lane.
    pub fn to_lane_polygons_geojson(&self) -> PyResult<String> {
        self.inner
            .to_lane_polygons_geojson(&Filter::All)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    /// Converts lane markings in the `StreetNetwork` to a GeoJSON format.
    ///
    /// Returns a GeoJSON string representing the lane markings, such as dashed or solid lines.
    pub fn to_lane_markings_geojson(&self) -> PyResult<String> {
        self.inner
            .to_lane_markings_geojson(&Filter::All)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    /// Converts intersection markings in the `StreetNetwork` to a GeoJSON format.
    ///
    /// Returns a GeoJSON string representing the markings at intersections.
    pub fn to_intersection_markings_geojson(&self) -> PyResult<String> {
        self.inner
            .to_intersection_markings_geojson(&Filter::All)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    /// Retrieves debugging steps for each modification applied to the `StreetNetwork`.
    ///
    /// Returns a vector of `PyDebugStreets` objects, which represent intermediate states or steps
    /// in the network transformation process.
    pub fn get_debug_steps(&self) -> PyResult<Vec<PyObject>> {
        Python::with_gil(|py| {
            Ok(self
                .inner
                .debug_steps
                .iter()
                .map(|x| PyDebugStreets { inner: x.clone() }.into_py(py))
                .collect())
        })
    }

    /// Converts the clockwise ordering debug information to a GeoJSON format.
    ///
    /// Returns a GeoJSON string showing the ordering of intersections in a clockwise manner, which
    /// can be helpful for debugging road connections.
    pub fn debug_clockwise_ordering_geojson(&self) -> PyResult<String> {
        self.inner
            .debug_clockwise_ordering_geojson(&Filter::All)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    /// Converts clockwise ordering information for a specific intersection to GeoJSON format.
    ///
    /// - `intersection`: ID of the intersection to be debugged.
    ///
    /// Returns a GeoJSON string with clockwise ordering for the given intersection.
    pub fn debug_clockwise_ordering_for_intersection_geojson(
        &self,
        intersection: usize,
    ) -> PyResult<String> {
        let mut intersections = BTreeSet::new();
        intersections.insert(IntersectionID(intersection));
        self.inner
            .debug_clockwise_ordering_geojson(&Filter::Filtered(BTreeSet::new(), intersections))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    /// Converts movement information from a specific lane to GeoJSON format.
    ///
    /// - `road`: ID of the road containing the lane.
    /// - `index`: Index of the lane within the specified road.
    ///
    /// Returns a GeoJSON string with information about allowed movements from this lane.
    pub fn debug_movements_from_lane_geojson(&self, road: usize, index: usize) -> PyResult<String> {
        self.inner
            .debug_movements_from_lane_geojson(LaneID {
                road: RoadID(road),
                index,
            })
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    /// Retrieves information about roads connected to a given intersection as a GeoJSON format.
    ///
    /// - `i`: ID of the intersection.
    ///
    /// Returns a GeoJSON string representing the roads connected to the specified intersection.
    pub fn debug_roads_connected_to_intersection_geojson(&self, i: usize) -> PyResult<String> {
        let mut polygons = Vec::new();
        for r in &self.inner.intersections[&IntersectionID(i)].roads {
            let road = &self.inner.roads[r];
            polygons.push(
                road.center_line
                    .make_polygons(road.total_width())
                    .to_geojson(Some(&self.inner.gps_bounds)),
            );
        }
        serde_json::to_string_pretty(&geom::geometries_to_geojson(polygons))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    /// Retrieves OSM tags for a given way (road or path).
    ///
    /// - `id`: The OSM ID of the way.
    ///
    /// Returns a JSON string with the OSM tags for the way, or an error if the way does not exist.
    pub fn get_osm_tags_for_way(&self, id: i64) -> PyResult<String> {
        if let Some(ref way) = self.ways.get(&osm::WayID(id)) {
            Ok(serde_json::to_string_pretty(&way.tags).unwrap())
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "unknown way {}",
                id
            )))
        }
    }

    /// Converts the entire `StreetNetwork` to a JSON format.
    ///
    /// Returns a JSON string representing the full `StreetNetwork` data structure.
    pub fn to_json(&self) -> PyResult<String> {
        serde_json::to_string_pretty(&self.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    /// Retrieves the geometry of a way (road or path) as a buffered polygon in GeoJSON format.
    ///
    /// - `id`: The OSM ID of the way.
    ///
    /// Returns a GeoJSON string of the way's geometry, including additional buffers and chevrons
    /// for indicating directionality.
    pub fn get_geometry_for_way(&self, id: i64) -> PyResult<String> {
        let id = osm::WayID(id);
        let width = self
            .inner
            .roads
            .values()
            .find(|r| r.from_osm_way(id))
            .map(|r| r.total_width())
            .unwrap();
        let polyline = PolyLine::unchecked_new(self.ways[&id].pts.clone());
        let mut polygon = polyline.make_polygons(1.5 * width);
        let num_chevrons = std::cmp::max(
            1,
            (polyline.length() / Distance::meters(50.0)).floor() as i64,
        );
        let chevrons = (1..=num_chevrons)
            .map(|i| {
                let (top_pt, angle) = polyline
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
        serde_json::to_string_pretty(&polygon.to_geojson(Some(&self.inner.gps_bounds)))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    /// Converts a way to an XML representation reflecting OSM tags.
    ///
    /// - `id`: The OSM ID of the way.
    ///
    /// Returns an XML string for the way, or an error if the way does not exist.
    pub fn way_to_xml(&self, id: i64) -> PyResult<String> {
        let Some(ref way) = self.ways.get(&osm::WayID(id)) else {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "unknown way {}",
                id
            )));
        };
        let mut out = format!(r#"<way id="{}""#, id);
        if let Some(version) = way.version {
            out.push_str(&format!(r#" version="{}""#, version));
        }
        out.push_str(">\n");
        for node in &way.nodes {
            out.push_str(&format!(r#"  <nd ref="{}"/>\n"#, node.0));
        }
        for (k, v) in way.tags.inner() {
            out.push_str(&format!(r#"  <tag k="{}" v="{}"/>\n"#, k, v));
        }
        out.push_str("</way>");
        Ok(out)
    }

    /// Finds and returns a block for a specified road side as a polygon.
    ///
    /// - `road`: ID of the road.
    /// - `left`: Boolean indicating if the left side of the road should be used.
    /// - `sidewalks`: Boolean indicating if sidewalks should be included in the block.
    ///
    /// Returns a polygon in GeoJSON format representing the block, or an error if not found.    
    pub fn find_block(&self, road: usize, left: bool, sidewalks: bool) -> PyResult<String> {
        self.inner
            .find_block(
                RoadSideID {
                    road: RoadID(road),
                    side: if left {
                        SideOfRoad::Left
                    } else {
                        SideOfRoad::Right
                    },
                },
                sidewalks,
            )
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?
            .render_polygon(&self.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    /// Finds and returns all blocks in the network as polygons.
    ///
    /// - `sidewalks`: Boolean indicating if sidewalks should be included in the blocks.
    ///
    /// Returns a GeoJSON string representing all blocks.    
    pub fn find_all_blocks(&self, sidewalks: bool) -> PyResult<String> {
        self.inner
            .find_all_blocks(sidewalks)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    // Moved all mutations methods into a single block to handle python implementation where everything needs to be in the same struct

    /// Overwrites OSM tags for a specified way, updating all affected roads in the `StreetNetwork`.
    ///
    /// - `id`: The OSM ID of the way.
    /// - `tags`: JSON string representing the new tags for the way.
    ///
    /// Updates the roads and intersections connected to this way based on the new tags.
    pub fn overwrite_osm_tags_for_way(&mut self, id: i64, tags: &str) -> PyResult<()> {
        let id = osm::WayID(id);
        let tags: Tags = serde_json::from_str(tags).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse tags: {}", e))
        })?;

        let mut intersections = BTreeSet::new();
        for road in self.inner.roads.values_mut() {
            if road.from_osm_way(id) {
                road.lane_specs_ltr = osm2streets::get_lane_specs_ltr(&tags, &self.inner.config);
                intersections.extend(road.endpoints());

                if let Ok(p) = Placement::parse(&tags) {
                    road.reference_line_placement = p;
                }

                road.update_center_line(self.inner.config.driving_side);
            }
        }
        for i in intersections {
            self.inner.update_i(i);
        }

        if let Some(way) = self.ways.get_mut(&id) {
            way.tags = tags;
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Unknown way ID {}",
                id
            )));
        }
        Ok(())
    }

    /// Collapses a short road by merging it into its neighboring road segments.
    ///
    /// - `road`: ID of the road to be collapsed.
    ///
    /// Collapsing short roads can help clean up unnecessary segments in the network.
    pub fn collapse_short_road(&mut self, road: usize) -> PyResult<()> {
        self.inner
            .collapse_short_road(RoadID(road))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
        Ok(())
    }

    /// Collapses a given intersection if it connects only two roads, simplifying the intersection.
    ///
    /// - `intersection`: ID of the intersection to be collapsed.
    ///
    /// This method can reduce complexity in sparse networks.
    pub fn collapse_intersection(&mut self, intersection: usize) -> PyResult<()> {
        let i = IntersectionID(intersection);
        if self.inner.intersections.get(&i).map(|int| int.roads.len()) == Some(2) {
            self.inner.collapse_intersection(i);
        }
        Ok(())
    }

    /// Zips a sidepath (e.g., a bike lane or sidewalk) alongside a main road.
    ///
    /// - `road`: ID of the road with the sidepath to be zipped.
    ///
    /// This method combines sidepaths with their adjacent roads.
    pub fn zip_sidepath(&mut self, road: usize) -> PyResult<()> {
        if let Some(sidepath) = Sidepath::new(&self.inner, RoadID(road)) {
            sidepath.zip(&mut self.inner);
        }
        Ok(())
    }
}

#[pyclass]
pub struct PyDebugStreets {
    inner: DebugStreets,
}

#[pymethods]
impl PyDebugStreets {
    pub fn get_label(&self) -> String {
        self.inner.label.clone()
    }
    //TODO: add get_network
    pub fn to_debug_geojson(&self) -> Option<String> {
        self.inner.to_debug_geojson()
    }
}

// Module setup
#[pymodule]
fn osm2streets_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyStreetNetwork>()?;
    m.add_class::<PyDebugStreets>()?;
    Ok(())
}
