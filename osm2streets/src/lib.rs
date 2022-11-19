#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

use std::cell::RefCell;
use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use abstutil::{deserialize_btreemap, serialize_btreemap};
use geom::{GPSBounds, PolyLine, Polygon, Pt2D};

pub use self::geometry::{intersection_polygon, InputRoad};
pub use self::ids::OriginalRoad;
pub use self::intersection::Intersection;
pub use self::lanes::{
    get_lane_specs_ltr, BufferType, Direction, LaneSpec, LaneType, NORMAL_LANE_THICKNESS,
    SIDEWALK_THICKNESS,
};
pub use self::road::Road;
pub use self::transform::Transformation;
pub use self::types::{
    ConflictType, ControlType, DrivingSide, IntersectionComplexity, MapConfig, Movement,
    NamePerLanguage,
};

mod edit;
mod geometry;
mod ids;
mod intersection;
mod lanes;
pub mod osm;
mod pathfinding;
mod render;
mod road;
mod transform;
mod types;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StreetNetwork {
    #[serde(
        serialize_with = "serialize_btreemap",
        deserialize_with = "deserialize_btreemap"
    )]
    pub roads: BTreeMap<OriginalRoad, Road>,
    #[serde(
        serialize_with = "serialize_btreemap",
        deserialize_with = "deserialize_btreemap"
    )]
    pub intersections: BTreeMap<osm::NodeID, Intersection>,

    pub boundary_polygon: Polygon,
    pub gps_bounds: GPSBounds,
    pub config: MapConfig,

    #[serde(skip_serializing, skip_deserializing)]
    pub debug_steps: RefCell<Vec<DebugStreets>>,
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

            debug_steps: RefCell::new(Vec::new()),
        }
    }

    pub fn insert_road(&mut self, road: Road) {
        let endpts = vec![road.src_i, road.dst_i];
        let id = road.id;
        self.roads.insert(road.id, road);
        for i in endpts {
            self.intersections.get_mut(&i).unwrap().roads.push(id);
            self.sort_roads(i);
            // Recalculate movements and complexity.
            self.recalculate_movements(i);
        }
    }

    pub fn remove_road(&mut self, id: OriginalRoad) -> Road {
        let endpts = {
            let r = &self.roads[&id];
            vec![r.src_i, r.dst_i]
        };
        for i in endpts {
            self.intersections
                .get_mut(&i)
                .unwrap()
                .roads
                .retain(|r| *r != id);
            // Since the roads are already sorted, removing doesn't break the sort.
            self.recalculate_movements(i);
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

    pub fn remove_intersection(&mut self, id: osm::NodeID) {
        let i = self.intersections.remove(&id).unwrap();
        if !i.roads.is_empty() {
            panic!("Can't remove_intersection({id}), it has roads still connected");
        }
    }

    /// Returns roads oriented in clockwise order around the intersection
    pub fn roads_per_intersection(&self, i: osm::NodeID) -> Vec<&Road> {
        self.intersections[&i]
            .roads
            .iter()
            .map(|r| &self.roads[r])
            .collect()
    }

    /// This calculates a road's `trimmed_center_line` early, before
    /// `Transformation::GenerateIntersectionGeometry` has run. Use sparingly.
    pub(crate) fn estimate_trimmed_geometry(&self, road_id: OriginalRoad) -> Result<PolyLine> {
        // First trim at one of the endpoints
        let trimmed_center_pts = {
            let mut input_roads = Vec::new();
            for road in self.roads_per_intersection(road_id.i1) {
                // trimmed_center_line hasn't been initialized yet, so override this
                let mut input = road.to_input_road();
                input.center_pts = road.untrimmed_road_geometry().0;
                input_roads.push(input);
            }
            let mut results = intersection_polygon(
                road_id.i1,
                input_roads,
                // TODO Not sure if we should use this or not
                &BTreeMap::new(),
            )?;
            results.trimmed_center_pts.remove(&road_id).unwrap().0
        };

        // Now the second
        {
            let mut input_roads = Vec::new();
            for road in self.roads_per_intersection(road_id.i2) {
                let mut input = road.to_input_road();
                if road.id == road_id {
                    input.center_pts = trimmed_center_pts.clone();
                } else {
                    input.center_pts = road.untrimmed_road_geometry().0;
                }
                input_roads.push(input);
            }
            let mut results = intersection_polygon(
                road_id.i2,
                input_roads,
                // TODO Not sure if we should use this or not
                &BTreeMap::new(),
            )?;
            Ok(results.trimmed_center_pts.remove(&road_id).unwrap().0)
        }
    }

    pub(crate) fn start_debug_step<I: Into<String>>(&self, label: I) {
        let copy = DebugStreets {
            label: label.into(),
            streets: StreetNetwork {
                roads: self.roads.clone(),
                intersections: self.intersections.clone(),
                boundary_polygon: self.boundary_polygon.clone(),
                gps_bounds: self.gps_bounds.clone(),
                config: self.config.clone(),
                debug_steps: RefCell::new(Vec::new()),
            },
            points: Vec::new(),
            polylines: Vec::new(),
        };
        self.debug_steps.borrow_mut().push(copy);
    }

    /// Only start a new debug step if there's at least one already (indicating that debugging is
    /// enabled).
    pub(crate) fn maybe_start_debug_step<I: Into<String>>(&self, label: I) {
        if self.debug_steps.borrow().is_empty() {
            return;
        }
        self.start_debug_step(label);
    }

    pub(crate) fn debug_intersection<I: Into<String>>(&self, i: osm::NodeID, label: I) {
        if let Some(step) = self.debug_steps.borrow_mut().last_mut() {
            step.points
                .push((self.intersections[&i].point, label.into()));
        }
    }

    pub(crate) fn debug_road<I: Into<String>>(&self, r: OriginalRoad, label: I) {
        if let Some(step) = self.debug_steps.borrow_mut().last_mut() {
            step.polylines
                .push((self.roads[&r].untrimmed_road_geometry().0, label.into()));
        }
    }

    // Restore the invariant that an intersection's roads are ordered clockwise
    //
    // TODO This doesn't handle trim_roads_for_merging
    fn sort_roads(&mut self, i: osm::NodeID) {
        let intersection = self.intersections.get_mut(&i).unwrap();

        // (ID, polyline pointing to the intersection, sorting point that's filled out later)
        let mut road_centers = Vec::new();
        let mut endpoints_for_center = Vec::new();
        for r in &intersection.roads {
            let road = &self.roads[r];
            // road.center_pts is unadjusted; it doesn't handle unequal widths yet. But that
            // shouldn't matter for sorting.
            let center_pl = if r.i1 == i {
                road.untrimmed_center_line.reversed()
            } else if r.i2 == i {
                road.untrimmed_center_line.clone()
            } else {
                panic!("Incident road {r} doesn't have an endpoint at {i}");
            };
            endpoints_for_center.push(center_pl.last_pt());

            road_centers.push((*r, center_pl, Pt2D::zero()));
        }
        // In most cases, this will just be the same point repeated a few times, so Pt2D::center is a
        // no-op. But when we have pretrimmed roads, this is much closer to the real "center" of the
        // polygon we're attempting to create.
        let intersection_center = Pt2D::center(&endpoints_for_center);

        // Sort the road polylines in clockwise order around the center. This is subtle --
        // https://a-b-street.github.io/docs/tech/map/geometry/index.html#sorting-revisited. When we
        // get this wrong, the resulting polygon looks like a "bowtie," because the order of the
        // intersection polygon's points follows this clockwise ordering of roads.
        //
        // We could use the point on each road center line farthest from the intersection center. But
        // when some of the roads bend around, this produces incorrect ordering. Try walking along that
        // center line a distance equal to the _shortest_ road.
        let shortest_center = road_centers
            .iter()
            .map(|(_, pl, _)| pl.length())
            .min()
            .unwrap();
        for (_, pl, sorting_pt) in &mut road_centers {
            *sorting_pt = pl.must_dist_along(pl.length() - shortest_center).0;
        }
        road_centers.sort_by_key(|(_, _, sorting_pt)| {
            sorting_pt
                .angle_to(intersection_center)
                .normalized_degrees() as i64
        });

        intersection.roads = road_centers.into_iter().map(|(r, _, _)| r).collect();
    }

    /// Recalculate movements, complexity, and conflict_level of an intersection.
    fn recalculate_movements(&mut self, i: osm::NodeID) {
        let (complexity, conflict_level, movements) =
            crate::transform::classify_intersections::guess_complexity(self, &i);
        let int = self.intersections.get_mut(&i).unwrap();
        int.movements = movements;
        int.conflict_level = conflict_level;
        // The fact that an intersection represents a road leaving the map bounds is stored in the
        // complexity field but guess_complexity ignores that. Make sure we don't overwrite it.
        if int.complexity != IntersectionComplexity::MapEdge {
            int.complexity = complexity;
        }
    }
}

/// Classifies pedestrian and cyclist crossings. Note lots of detail is missing.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CrossingType {
    /// Part of some traffic signal
    Signalized,
    /// Not part of a traffic signal
    Unsignalized,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RestrictionType {
    BanTurns,
    OnlyAllowTurns,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TurnRestriction(pub OriginalRoad, pub RestrictionType, pub OriginalRoad);

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
