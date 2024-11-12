use std::collections::{BTreeMap, BTreeSet};
use std::sync::Once;

use abstutil::{Tags, Timer};
use chrono::NaiveDateTime;
use geom::{Distance, LonLat, PolyLine, Polygon};
use serde::{Deserialize, Serialize};
use serde_json; // Added serde_json import
use pyo3::prelude::*;
use osm2streets::{
    osm, DebugStreets, DrivingSide, Filter, IntersectionID, LaneID, MapConfig, Placement, RoadID,
    RoadSideID, SideOfRoad, Sidepath, StreetNetwork, Transformation,
};

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

#[pymethods] // Changed from #[pyfunction] to #[pymethods]
impl PyStreetNetwork {
    #[new]
    pub fn new(
        py: Python, // Added `py: Python` here to get the Python context
        osm_input: &[u8],
        clip_pts_geojson: &str,
        input: PyObject,
    ) -> PyResult<Self> {
        SETUP_LOGGER.call_once(|| env_logger::init());

        let input: ImportOptions = serde_json::from_str(input.extract::<&str>(py)?)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Failed to parse input: {}", e)))?;

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
            x => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Unknown driving side: {x}"))),
        };

        let mut timer = Timer::throwaway();
        let (mut street_network, doc) = streets_reader::osm_to_street_network(osm_input, clip_pts, cfg, &mut timer)
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

    pub fn to_geojson_plain(&self) -> PyResult<String> {
        self.inner.to_geojson(&Filter::All).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    pub fn to_lane_polygons_geojson(&self) -> PyResult<String> {
        self.inner.to_lane_polygons_geojson(&Filter::All).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    pub fn to_lane_markings_geojson(&self) -> PyResult<String> {
        self.inner.to_lane_markings_geojson(&Filter::All).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    pub fn to_intersection_markings_geojson(&self) -> PyResult<String> {
        self.inner.to_intersection_markings_geojson(&Filter::All).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
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