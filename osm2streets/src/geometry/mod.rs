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
mod degenerate;
mod on_off_ramp;
mod terminus;

use std::collections::BTreeMap;

use geom::{Distance, PolyLine, Polygon, Pt2D};

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

    pub fn center_line_pointed_at(&self, i: IntersectionID) -> PolyLine {
        if self.dst_i == i {
            self.center_line.clone()
        } else {
            assert_eq!(self.src_i, i);
            self.center_line.reversed()
        }
    }

    // TODO This is a hack. Probably we want to get rid of InputRoad.
    pub fn to_road(&self) -> crate::Road {
        crate::Road {
            id: self.id,
            src_i: self.src_i,
            dst_i: self.dst_i,
            center_line: self.center_line.clone(),
            lane_specs_ltr: vec![crate::LaneSpec {
                lt: crate::LaneType::Driving,
                dir: crate::Direction::Fwd,
                width: self.total_width,
                turn_restrictions: Vec::new(),
            }],
            // Mostly dummy values, except for what selfEdge::calculate needs
            osm_ids: Vec::new(),
            highway_type: String::new(),
            name: None,
            internal_junction_road: false,
            layer: 0,
            reference_line: PolyLine::dummy(),
            reference_line_placement: crate::lanes::Placement::Transition,
            turn_restrictions: Vec::new(),
            complicated_turn_restrictions: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct Results {
    pub intersection_id: IntersectionID,
    pub intersection_polygon: Polygon,
    pub trimmed_center_pts: BTreeMap<RoadID, PolyLine>,
    /// Extra polygons with labels to debug the algorithm
    pub debug: Vec<(String, Polygon)>,
}

// TODO Dedupe with Piece!
#[derive(Clone)]
pub(crate) struct RoadLine {
    id: RoadID,
    // Both are oriented to be incoming to the intersection (ending at it).
    // TODO Maybe express as the "right" and "left"
    fwd_pl: PolyLine,
    back_pl: PolyLine,
}

// TODO Remove in favor of Ring::deduping_new
fn close_off_polygon(mut pts: Vec<Pt2D>) -> Vec<Pt2D> {
    if pts.last().unwrap().approx_eq(pts[0], Distance::meters(0.1)) {
        pts.pop();
    }
    pts.push(pts[0]);
    pts
}
