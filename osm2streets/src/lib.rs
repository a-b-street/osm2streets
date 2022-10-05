#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use abstutil::{deserialize_btreemap, serialize_btreemap, Tags};
use geom::{Angle, Distance, GPSBounds, PolyLine, Polygon, Pt2D};

pub use self::geometry::{intersection_polygon, InputRoad};
pub use self::lanes::{
    get_lane_specs_ltr, BufferType, Direction, LaneSpec, LaneType, NORMAL_LANE_THICKNESS,
    SIDEWALK_THICKNESS,
};
pub use self::transform::Transformation;
pub use self::types::{
    ControlType, DrivingSide, IntersectionComplexity, MapConfig, NamePerLanguage,
};

mod edit;
mod geometry;
pub mod initial;
mod lanes;
pub mod osm;
mod pathfinding;
mod render;
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

/// A way to refer to roads across many maps and over time. Also trivial to relate with OSM to find
/// upstream problems.
//
// - Using LonLat is more indirect, and f64's need to be trimmed and compared carefully with epsilon
//   checks.
// - TODO Look at some stable ID standard like linear referencing
// (https://github.com/opentraffic/architecture/issues/1).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OriginalRoad {
    pub osm_way_id: osm::WayID,
    pub i1: osm::NodeID,
    pub i2: osm::NodeID,
}

impl fmt::Display for OriginalRoad {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "OriginalRoad({} from {} to {}",
            self.osm_way_id, self.i1, self.i2
        )
    }
}
impl fmt::Debug for OriginalRoad {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl OriginalRoad {
    pub fn new(way: i64, (i1, i2): (i64, i64)) -> OriginalRoad {
        OriginalRoad {
            osm_way_id: osm::WayID(way),
            i1: osm::NodeID(i1),
            i2: osm::NodeID(i2),
        }
    }

    /// Prints the OriginalRoad in a way that can be copied to Rust code.
    pub fn as_string_code(&self) -> String {
        format!(
            "OriginalRoad::new({}, ({}, {}))",
            self.osm_way_id.0, self.i1.0, self.i2.0
        )
    }

    pub fn has_common_endpoint(&self, other: OriginalRoad) -> bool {
        if self.i1 == other.i1 || self.i1 == other.i2 {
            return true;
        }
        if self.i2 == other.i1 || self.i2 == other.i2 {
            return true;
        }
        false
    }

    // TODO Doesn't handle two roads between the same pair of intersections
    pub fn common_endpt(&self, other: OriginalRoad) -> osm::NodeID {
        #![allow(clippy::suspicious_operation_groupings)]
        if self.i1 == other.i1 || self.i1 == other.i2 {
            return self.i1;
        }
        if self.i2 == other.i1 || self.i2 == other.i2 {
            return self.i2;
        }
        panic!("{:?} and {:?} have no common_endpt", self, other);
    }

    pub fn other_side(&self, i: osm::NodeID) -> osm::NodeID {
        if self.i1 == i {
            self.i2
        } else if self.i2 == i {
            self.i1
        } else {
            panic!("{} doesn't have {} on either side", self, i);
        }
    }
}

impl StreetNetwork {
    pub fn blank() -> Self {
        Self {
            roads: BTreeMap::new(),
            intersections: BTreeMap::new(),
            // Some nonsense thing
            boundary_polygon: Polygon::rectangle(1.0, 1.0),
            gps_bounds: GPSBounds::new(),
            config: MapConfig::default_for_side(DrivingSide::Right),

            debug_steps: RefCell::new(Vec::new()),
        }
    }

    pub fn insert_road(&mut self, id: OriginalRoad, road: Road) {
        self.roads.insert(id, road);
        for i in [id.i1, id.i2] {
            self.intersections.get_mut(&i).unwrap().roads.push(id);
            self.sort_roads(i);
        }
    }

    pub fn remove_road(&mut self, id: &OriginalRoad) -> Road {
        // Since the roads are already sorted, removing doesn't hurt anything
        self.intersections
            .get_mut(&id.i1)
            .unwrap()
            .roads
            .retain(|r| r != id);
        self.intersections
            .get_mut(&id.i2)
            .unwrap()
            .roads
            .retain(|r| r != id);
        self.roads.remove(id).unwrap()
    }

    pub fn retain_roads<F: Fn(&OriginalRoad, &Road) -> bool>(&mut self, f: F) {
        let mut remove = Vec::new();
        for (id, r) in &self.roads {
            if !f(id, r) {
                remove.push(*id);
            }
        }
        for id in remove {
            self.remove_road(&id);
        }
    }

    // This always returns roads oriented in clockwise order around the intersection
    // TODO Consider not cloning. Many callers will have to change
    pub fn roads_per_intersection(&self, i: osm::NodeID) -> Vec<OriginalRoad> {
        self.intersections[&i].roads.clone()
    }

    /// (Intersection polygon, polygons for roads, list of labeled polygons to debug)
    #[allow(clippy::type_complexity)]
    pub fn preview_intersection(
        &self,
        id: osm::NodeID,
    ) -> Result<(Polygon, Vec<Polygon>, Vec<(String, Polygon)>)> {
        let mut input_roads = Vec::new();
        for r in self.roads_per_intersection(id) {
            input_roads.push(initial::Road::new(self, r).to_input_road());
        }
        let results = intersection_polygon(
            id,
            input_roads,
            // This'll be empty unless we've called merge_short_road
            &self.intersections[&id].trim_roads_for_merging,
        )?;
        Ok((
            results.intersection_polygon,
            results
                .trimmed_center_pts
                .into_values()
                .map(|(pl, half_width)| pl.make_polygons(2.0 * half_width))
                .collect(),
            results.debug,
        ))
    }

    /// Generate the trimmed `PolyLine` for a single Road by calculating both intersections
    pub fn trimmed_road_geometry(&self, road_id: OriginalRoad) -> Result<PolyLine> {
        // First trim at one of the endpoints
        let trimmed_center_pts = {
            let mut input_roads = Vec::new();
            for r in self.roads_per_intersection(road_id.i1) {
                input_roads.push(initial::Road::new(self, r).to_input_road());
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
            for r in self.roads_per_intersection(road_id.i2) {
                let mut road = initial::Road::new(self, r).to_input_road();
                if r == road_id {
                    road.center_pts = trimmed_center_pts.clone();
                }
                input_roads.push(road);
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
                PolyLine::must_new(road.osm_center_points.clone()).reversed()
            } else if r.i2 == i {
                PolyLine::must_new(road.osm_center_points.clone())
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
}

// Mutations and supporting queries
impl StreetNetwork {
    pub fn can_delete_intersection(&self, i: osm::NodeID) -> bool {
        self.roads_per_intersection(i).is_empty()
    }

    pub fn delete_intersection(&mut self, id: osm::NodeID) {
        if !self.can_delete_intersection(id) {
            panic!(
                "Can't delete_intersection {}, must have roads connected",
                id
            );
        }
        self.intersections.remove(&id).unwrap();
    }

    pub fn move_intersection(&mut self, id: osm::NodeID, point: Pt2D) -> Option<Vec<OriginalRoad>> {
        self.intersections.get_mut(&id).unwrap().point = point;

        // Update all the roads.
        let mut fixed = Vec::new();
        for r in self.roads_per_intersection(id) {
            fixed.push(r);
            let road = self.roads.get_mut(&r).unwrap();
            if r.i1 == id {
                road.osm_center_points[0] = point;
            } else {
                assert_eq!(r.i2, id);
                *road.osm_center_points.last_mut().unwrap() = point;
            }
        }

        Some(fixed)
    }

    pub fn closest_intersection(&self, pt: Pt2D) -> osm::NodeID {
        self.intersections
            .iter()
            .min_by_key(|(_, i)| i.point.dist_to(pt))
            .map(|(id, _)| *id)
            .unwrap()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Road {
    /// This is effectively a PolyLine, except there's a case where we need to plumb forward
    /// cul-de-sac roads for roundabout handling. No transformation of these points whatsoever has
    /// happened.
    pub osm_center_points: Vec<Pt2D>,
    pub osm_tags: Tags,
    pub turn_restrictions: Vec<(RestrictionType, OriginalRoad)>,
    /// (via, to). For turn restrictions where 'via' is an entire road. Only BanTurns.
    pub complicated_turn_restrictions: Vec<(OriginalRoad, OriginalRoad)>,
    pub percent_incline: f64,
    /// Is there a tagged crosswalk near each end of the road?
    pub crosswalk_forward: bool,
    pub crosswalk_backward: bool,
    // TODO Preserving these two across transformations (especially merging dual carriageways!)
    // could be really hard. It might be better to split the road into two pieces to match the more
    // often used OSM style.
    /// Barrier nodes along this road's original center line.
    pub barrier_nodes: Vec<Pt2D>,
    /// Crossing nodes along this road's original center line. Attributes about the crossing are
    /// lost.
    pub crossing_nodes: Vec<Pt2D>,

    /// Derived from osm_tags. Not automatically updated.
    pub lane_specs_ltr: Vec<LaneSpec>,
}

impl Road {
    pub fn new(osm_center_points: Vec<Pt2D>, osm_tags: Tags, config: &MapConfig) -> Result<Self> {
        // Just flush out errors immediately.
        // TODO Store the PolyLine, not a Vec<Pt2D>
        let _ = PolyLine::new(osm_center_points.clone())?;

        let lane_specs_ltr = get_lane_specs_ltr(&osm_tags, config);

        Ok(Self {
            osm_center_points,
            osm_tags,
            turn_restrictions: Vec::new(),
            complicated_turn_restrictions: Vec::new(),
            percent_incline: 0.0,
            // Start assuming there's a crosswalk everywhere, and maybe filter it down
            // later
            crosswalk_forward: true,
            crosswalk_backward: true,
            barrier_nodes: Vec::new(),
            crossing_nodes: Vec::new(),

            lane_specs_ltr,
        })
    }

    // TODO For the moment, treating all rail things as light rail
    pub fn is_light_rail(&self) -> bool {
        self.osm_tags.is_any("railway", vec!["light_rail", "rail"])
    }

    pub fn is_footway(&self) -> bool {
        self.osm_tags.is_any(
            osm::HIGHWAY,
            vec![
                // TODO cycleway in here is weird, reconsider. is_footway is only used in one
                // disabled transformation right now.
                "cycleway",
                "footway",
                "path",
                "pedestrian",
                "steps",
                "track",
            ],
        )
    }

    pub fn is_service(&self) -> bool {
        self.osm_tags.is(osm::HIGHWAY, "service")
    }

    pub fn is_cycleway(&self) -> bool {
        // Don't repeat the logic looking at the tags, just see what lanes we'll create
        let mut bike = false;
        for spec in &self.lane_specs_ltr {
            if spec.lt == LaneType::Biking {
                bike = true;
            } else if spec.lt != LaneType::Shoulder {
                return false;
            }
        }
        bike
    }

    pub fn is_driveable(&self) -> bool {
        self.lane_specs_ltr
            .iter()
            .any(|spec| spec.lt == LaneType::Driving)
    }

    pub fn oneway_for_driving(&self) -> Option<Direction> {
        LaneSpec::oneway_for_driving(&self.lane_specs_ltr)
    }

    /// Points from first to last point. Undefined for loops.
    pub fn angle(&self) -> Angle {
        self.osm_center_points[0].angle_to(*self.osm_center_points.last().unwrap())
    }

    pub fn length(&self) -> Distance {
        PolyLine::unchecked_new(self.osm_center_points.clone()).length()
    }

    pub fn get_zorder(&self) -> isize {
        if let Some(layer) = self.osm_tags.get("layer") {
            match layer.parse::<f64>() {
                // Just drop .5 for now
                Ok(l) => l as isize,
                Err(_) => {
                    warn!("Weird layer={} on {}", layer, self.osm_url());
                    0
                }
            }
        } else {
            0
        }
    }

    /// Returns the corrected (but untrimmed) center and total width for a road
    pub fn untrimmed_road_geometry(&self) -> (PolyLine, Distance) {
        let mut total_width = Distance::ZERO;
        let mut sidewalk_right = None;
        let mut sidewalk_left = None;
        for l in &self.lane_specs_ltr {
            total_width += l.width;
            if l.lt.is_walkable() {
                if l.dir == Direction::Back {
                    sidewalk_left = Some(l.width);
                } else {
                    sidewalk_right = Some(l.width);
                }
            }
        }

        // If there's a sidewalk on only one side, adjust the true center of the road.
        // TODO I don't remember the rationale for doing this in the first place. What if there's a
        // shoulder and a sidewalk of different widths? We don't do anything then
        let mut true_center = match PolyLine::new(self.osm_center_points.clone()) {
            Ok(pl) => pl,
            Err(err) => panic!(
                "untrimmed_road_geometry of {} failed: {}",
                self.osm_url(),
                err
            ),
        };
        match (sidewalk_right, sidewalk_left) {
            (Some(w), None) => {
                true_center = true_center.must_shift_right(w / 2.0);
            }
            (None, Some(w)) => {
                true_center = true_center.must_shift_right(w / 2.0);
            }
            _ => {}
        }

        (true_center, total_width)
    }

    pub fn osm_url(&self) -> String {
        // Since we don't store an OriginalRoad (since we may need to update it during
        // transformations), this may be convenient
        format!(
            "http://openstreetmap.org/way/{}",
            self.osm_tags.get(osm::OSM_WAY_ID).unwrap()
        )
    }

    pub fn total_width(&self) -> Distance {
        self.lane_specs_ltr.iter().map(|l| l.width).sum()
    }

    /// Returns one PolyLine representing the center of each lane in this road. Pass in
    /// `road_center` generated from `InitialMap` (trimmed from the intersection) or from
    /// `osm_center_points`. The result also faces the same direction as the road.
    pub fn get_lane_center_lines(&self, road_center: &PolyLine) -> Vec<PolyLine> {
        let total_width = self.total_width();

        let mut width_so_far = Distance::ZERO;
        let mut output = Vec::new();
        for lane in &self.lane_specs_ltr {
            width_so_far += lane.width / 2.0;
            output.push(
                road_center
                    .shift_from_center(total_width, width_so_far)
                    .unwrap_or_else(|_| road_center.clone()),
            );
            width_so_far += lane.width / 2.0;
        }
        output
    }

    /// Returns the untrimmed left and right side of the road, oriented in the same direction of
    /// the road
    pub fn get_untrimmed_sides(&self) -> Result<(PolyLine, PolyLine)> {
        let (center, total_width) = self.untrimmed_road_geometry();
        let left = center.shift_from_center(total_width, -total_width / 2.0)?;
        let right = center.shift_from_center(total_width, total_width / 2.0)?;
        Ok((left, right))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Intersection {
    /// Represents the original place where OSM center-lines meet. This may be meaningless beyond
    /// StreetNetwork; roads and intersections get merged and deleted.
    pub point: Pt2D,
    pub complexity: IntersectionComplexity,
    pub control: ControlType,
    pub elevation: Distance,

    /// All roads connected to this intersection. They may be incoming or outgoing relative to this
    /// intersection. They're ordered clockwise aroundd the intersection.
    pub roads: Vec<OriginalRoad>,

    // true if src_i matches this intersection (or the deleted/consolidated one, whatever)
    pub trim_roads_for_merging: BTreeMap<(osm::WayID, bool), Pt2D>,
}

impl Intersection {
    pub fn new(point: Pt2D, complexity: IntersectionComplexity, control: ControlType) -> Self {
        Self {
            point,
            complexity,
            control,
            // Filled out later
            roads: Vec::new(),
            elevation: Distance::ZERO,
            trim_roads_for_merging: BTreeMap::new(),
        }
    }

    fn is_border(&self) -> bool {
        self.control == ControlType::Border
    }
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
