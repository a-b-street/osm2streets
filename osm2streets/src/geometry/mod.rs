//! OSM describes roads as center-lines that intersect. Turn these into road and intersection
//! polygons roughly by
//!
//! 1) treating the road as a PolyLine with a width, so that it has a left and right edge
//! 2) finding the places where the edges of different roads intersect
//! 3) "Trimming back" the center lines to avoid the overlap
//! 4) Producing a polygon for the intersection itsef
//!
//! I wrote a novella about this: <https://a-b-street.github.io/docs/tech/map/geometry/index.html>

mod algorithm;

use std::collections::{BTreeSet, BTreeMap};

use geom::{Distance, PolyLine, Polygon};

use crate::{IntersectionID, RoadID};
pub use algorithm::intersection_polygon;

// For anyone considering removing this indirection in the future: it's used to recalculate one or
// two intersections at a time in A/B Street's edit mode. Within just this repo, it does seem
// redundant.
#[derive(Clone)]
pub struct InputRoad {
    pub id: RoadID,
    pub src_i: IntersectionID,
    pub dst_i: IntersectionID,
    /// The true center of the road, including sidewalks. The input is untrimmed when called on the
    /// first endpoint, then trimmed on that first side when called on the second endpoint.
    pub center_line: PolyLine,
    pub total_width: Distance,
    pub highway_type: String,
}

impl InputRoad {
    pub fn half_width(&self) -> Distance {
        self.total_width / 2.0
    }
}

#[derive(Clone)]
pub struct Results {
    pub intersection_id: IntersectionID,
    pub intersection_polygon: Polygon,
    pub trimmed_center_pts: BTreeMap<RoadID, PolyLine>,
    /// Did we extend, not shorten, a road?
    pub extended_roads: BTreeSet<RoadID>,
    /// Extra polygons with labels to debug the algorithm
    pub debug: Vec<(String, Polygon)>,
}
