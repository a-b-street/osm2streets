#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use abstutil::{deserialize_btreemap, serialize_btreemap};
use geom::{GPSBounds, PolyLine, Polygon, Pt2D};

pub use self::geometry::{intersection_polygon, InputRoad};
pub(crate) use self::ids::RoadWithEndpoints;
pub use self::ids::{CommonEndpoint, IntersectionID, LaneID, RoadID};
pub use self::intersection::{
    Intersection, IntersectionControl, IntersectionKind, Turn, TrafficConflict,
};
pub use self::lanes::{
    get_lane_specs_ltr, BufferType, Direction, LaneSpec, LaneType, Placement,
    NORMAL_LANE_THICKNESS, SIDEWALK_THICKNESS,
};
pub use self::road::{Road, StopLine, TrafficInterruption};
pub use self::transform::Transformation;
pub use self::types::{DrivingSide, MapConfig, NamePerLanguage};

mod edit;
mod geometry;
mod ids;
mod intersection;
mod lanes;
mod marking;
mod operations;
pub mod osm;
mod output;
mod paint;
mod pathfinding;
mod render;
mod road;
mod transform;
mod types;
mod validate;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StreetNetwork {
    #[serde(
        serialize_with = "serialize_btreemap",
        deserialize_with = "deserialize_btreemap"
    )]
    pub roads: BTreeMap<RoadID, Road>,
    #[serde(
        serialize_with = "serialize_btreemap",
        deserialize_with = "deserialize_btreemap"
    )]
    pub intersections: BTreeMap<IntersectionID, Intersection>,

    pub boundary_polygon: Polygon,
    pub gps_bounds: GPSBounds,
    pub config: MapConfig,

    #[serde(skip_serializing, skip_deserializing)]
    pub debug_steps: Vec<DebugStreets>,

    intersection_id_counter: usize,
    road_id_counter: usize,
}

#[derive(Clone, Debug)]
pub struct DebugStreets {
    pub label: String,
    /// A full copy of an intermediate `StreetNetwork` that can be rendered. It doesn't recursively
    /// contain any `debug_steps`.
    pub streets: StreetNetwork,
    /// Extra labelled points to debug
    pub points: Vec<(Pt2D, String)>,
    /// Extra labelled polylines to debug
    pub polylines: Vec<(PolyLine, String)>,
}

impl StreetNetwork {
    pub fn blank() -> Self {
        Self {
            roads: BTreeMap::new(),
            intersections: BTreeMap::new(),
            // Some nonsense thing
            boundary_polygon: Polygon::rectangle(1.0, 1.0),
            gps_bounds: GPSBounds::new(),
            config: MapConfig::default(),

            debug_steps: Vec::new(),

            intersection_id_counter: 0,
            road_id_counter: 0,
        }
    }

    pub fn insert_road(&mut self, road: Road) {
        let endpts = road.endpoints();
        let id = road.id;
        self.roads.insert(road.id, road);
        for i in endpts {
            self.intersections.get_mut(&i).unwrap().roads.push(id);
            self.sort_roads(i);
            self.update_i(i);
        }
    }

    pub fn remove_road(&mut self, id: RoadID) -> Road {
        for i in self.roads[&id].endpoints() {
            self.intersections
                .get_mut(&i)
                .unwrap()
                .roads
                .retain(|r| *r != id);
            // Since the roads are already sorted, removing doesn't break the sort.
            self.update_i(i);
        }
        self.roads.remove(&id).unwrap()
    }

    pub fn retain_roads<F: Fn(&Road) -> bool>(&mut self, f: F) {
        let mut remove = Vec::new();
        for r in self.roads.values() {
            if !f(r) {
                remove.push(r.id);
            }
        }
        for id in remove {
            self.remove_road(id);
        }
    }

    pub fn remove_intersection(&mut self, id: IntersectionID) {
        let i = self.intersections.remove(&id).unwrap();
        if !i.roads.is_empty() {
            panic!("Can't remove_intersection({id}), it has roads still connected");
        }
    }

    /// Returns roads oriented in clockwise order around the intersection
    pub fn roads_per_intersection(&self, i: IntersectionID) -> Vec<&Road> {
        self.intersections[&i]
            .roads
            .iter()
            .map(|r| &self.roads[r])
            .collect()
    }

    pub(crate) fn start_debug_step<I: Into<String>>(&mut self, label: I) {
        let copy = DebugStreets {
            label: label.into(),
            streets: StreetNetwork {
                roads: self.roads.clone(),
                intersections: self.intersections.clone(),
                boundary_polygon: self.boundary_polygon.clone(),
                gps_bounds: self.gps_bounds.clone(),
                config: self.config.clone(),
                debug_steps: Vec::new(),
                intersection_id_counter: self.intersection_id_counter,
                road_id_counter: self.road_id_counter,
            },
            points: Vec::new(),
            polylines: Vec::new(),
        };
        self.debug_steps.push(copy);
    }

    /// Only start a new debug step if there's at least one already (indicating that debugging is
    /// enabled).
    pub(crate) fn maybe_start_debug_step<I: Into<String>>(&mut self, label: I) {
        if self.debug_steps.is_empty() {
            return;
        }
        self.start_debug_step(label);
    }

    pub(crate) fn debug_intersection<I: Into<String>>(&mut self, i: IntersectionID, label: I) {
        if let Some(step) = self.debug_steps.last_mut() {
            step.points
                .push((self.intersections[&i].polygon.center(), label.into()));
        }
    }

    pub(crate) fn debug_road<I: Into<String>>(&mut self, r: RoadID, label: I) {
        if let Some(step) = self.debug_steps.last_mut() {
            step.polylines
                .push((self.roads[&r].center_line.clone(), label.into()));
        }
    }

    pub(crate) fn debug_point<I: Into<String>>(&mut self, pt: Pt2D, label: I) {
        if let Some(step) = self.debug_steps.last_mut() {
            step.points.push((pt, label.into()));
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RestrictionType {
    BanTurns,
    OnlyAllowTurns,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TurnRestriction(pub RoadID, pub RestrictionType, pub RoadID);

impl RestrictionType {
    pub fn new(restriction: &str) -> Option<RestrictionType> {
        // TODO There's a huge space of things not represented yet: time conditions, bus-only, no
        // right turn on red...

        // There are so many possibilities:
        // https://taginfo.openstreetmap.org/keys/restriction#values
        // Just attempt to bucket into allow / deny.
        if restriction.contains("no_") || restriction == "psv" {
            Some(RestrictionType::BanTurns)
        } else if restriction.contains("only_") {
            Some(RestrictionType::OnlyAllowTurns)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    // Check at compile-time if StreetNetwork can be shared across a thread. If a RefCell or
    // something sneaks in anywhere, this'll fail.
    #[test]
    fn test_sync() {
        must_be_sync(super::StreetNetwork::blank());
    }

    fn must_be_sync<T: Sync>(_x: T) {}
}
