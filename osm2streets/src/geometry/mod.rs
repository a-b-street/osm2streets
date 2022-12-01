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

use std::collections::BTreeMap;

use geom::Polygon;

use crate::{IntersectionID, Road, RoadID};
pub use algorithm::intersection_polygon;

// Why doesn't intersection_polygon() directly operate on a StreetNetwork? Within this repo, it
// probably could. But this code is also used to recalculate one or two intersections at a time in
// A/B Street's edit mode, which currently works off of a different representation than a
// StreetNetwork. Any future refactors should keep that in mind.

#[derive(Clone)]
pub struct Results {
    pub intersection_id: IntersectionID,
    pub intersection_polygon: Polygon,
    /// Echo back all Roads passed in, with `trimmed_center_line` modified
    pub roads: BTreeMap<RoadID, Road>,
    /// Extra polygons with labels to debug the algorithm
    pub debug: Vec<(String, Polygon)>,
}
