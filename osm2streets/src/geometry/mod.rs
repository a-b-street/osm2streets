//! OSM describes roads as center-lines that intersect. Turn these into road and intersection
//! polygons roughly by
//!
//! 1) treating the road as a PolyLine with a width, so that it has a left and right edge
//! 2) finding the places where the edges of different roads intersect
//! 3) "Trimming back" the center lines to avoid the overlap
//! 4) Producing a polygon for the intersection itsef
//!
//! I wrote a novella about this: <https://a-b-street.github.io/docs/tech/map/geometry/index.html>

mod degenerate;
mod general_case;
mod on_off_ramp;
mod pretrimmed;
mod terminus;

use std::collections::BTreeMap;

use anyhow::Result;
use geom::{Distance, PolyLine, Polygon, Pt2D, Ring};

use crate::road::RoadEdge;
use crate::{IntersectionID, IntersectionKind, RoadID, StopLine};

// For anyone considering removing this indirection in the future: it's used to recalculate one or
// two intersections at a time in A/B Street's edit mode. Within just this repo, it does seem
// redundant.
#[derive(Clone)]
pub struct InputRoad {
    pub id: RoadID,
    pub src_i: IntersectionID,
    pub dst_i: IntersectionID,
    /// The true center of the road, including sidewalks. This must be untrimmed on both ends when passed in.
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
                dir: crate::Direction::Forward,
                width: self.total_width,
                allowed_turns: Default::default(),
                lane: None,
            }],
            // Mostly dummy values, except for what selfEdge::calculate needs
            osm_ids: Vec::new(),
            highway_type: String::new(),
            name: None,
            internal_junction_road: false,
            layer: 0,
            speed_limit: None,
            reference_line: PolyLine::dummy(),
            reference_line_placement: osm2lanes::Placement::Transition,
            trim_start: Distance::ZERO,
            trim_end: Distance::ZERO,
            turn_restrictions: Vec::new(),
            complicated_turn_restrictions: Vec::new(),
            stop_line_start: StopLine::dummy(),
            stop_line_end: StopLine::dummy(),
        }
    }
}

#[derive(Clone)]
pub struct Results {
    pub intersection_id: IntersectionID,
    pub intersection_polygon: Polygon,
    /// The only transformation to `center_line` passed in must be to trim it (reducing the length)
    /// or to lengthen the first/last line. `trim_starts` and `trim_ends` are calculated from this,
    /// and the caller deliberately can't see `trimmed_center_pts`.
    trimmed_center_pts: BTreeMap<RoadID, PolyLine>,
    pub trim_starts: BTreeMap<RoadID, Distance>,
    pub trim_ends: BTreeMap<RoadID, Distance>,
    /// Extra points with labels to debug the algorithm
    pub debug: Vec<(Pt2D, String)>,
}

/// Trims back all roads connected to the intersection, and generates a polygon for the
/// intersection. The trimmed roads should meet this polygon at a right angle. The input is assumed
/// to be untrimmed (based on the original reference geometry), and the roads must be ordered clockwise.
pub fn intersection_polygon(
    intersection_id: IntersectionID,
    intersection_kind: IntersectionKind,
    input_roads: Vec<InputRoad>,
    trim_roads_for_merging: &BTreeMap<(RoadID, bool), Pt2D>,
) -> Result<Results> {
    // TODO Possibly take this as input in the first place
    let mut roads: BTreeMap<RoadID, InputRoad> = BTreeMap::new();
    let mut sorted_roads: Vec<RoadID> = Vec::new();
    for r in input_roads {
        sorted_roads.push(r.id);
        roads.insert(r.id, r);
    }

    let results = Results {
        intersection_id,
        intersection_polygon: Polygon::dummy(),
        debug: Vec::new(),
        trimmed_center_pts: BTreeMap::new(),
        trim_starts: BTreeMap::new(),
        trim_ends: BTreeMap::new(),
    };

    // TODO Hack! Transformation::CollapseDegenerateIntersections triggers this, because we try to
    // update_geometry in the middle. We need to track changes and defer the recalculation.
    if roads.is_empty() {
        error!("Hack! intersection_polygon({intersection_id}) called with no roads");
        return Ok(results);
    }

    let mut untrimmed_roads = roads.clone();

    let mut results = if roads.len() == 1 {
        terminus::terminus(
            results,
            roads.into_values().next().unwrap(),
            intersection_kind,
        )
    } else if roads.len() == 2 {
        let mut iter = roads.into_values();
        degenerate::degenerate(results, iter.next().unwrap(), iter.next().unwrap())
    } else if !trim_roads_for_merging.is_empty() {
        pretrimmed::pretrimmed_geometry(results, roads, sorted_roads, trim_roads_for_merging)
    } else if let Some(result) =
        on_off_ramp::on_off_ramp(results.clone(), roads.clone(), &sorted_roads)
    {
        Ok(result)
    } else {
        general_case::trim_to_corners(results, roads, sorted_roads)
    }?;

    // We've filled out trimmed_center_pts, now calculate trim_starts and trim_ends
    for (r, pl) in &results.trimmed_center_pts {
        // Normally this'll be positive, indicating trim. If it's negative, the algorithm extended
        // the first or last line
        let road = untrimmed_roads.remove(r).unwrap();
        let trim = road.center_line.length() - pl.length();
        if road.src_i == intersection_id {
            results.trim_starts.insert(*r, trim);
        } else {
            results.trim_ends.insert(*r, trim);
        }
    }

    Ok(results)
}

/// After trimming roads back, form the final polygon using the endpoints of each road edge and
/// also the corners where those edges originally met.
fn polygon_from_corners(
    roads: &BTreeMap<RoadID, InputRoad>,
    sorted_road_ids: &Vec<RoadID>,
    orig_centers: &BTreeMap<RoadID, PolyLine>,
    i: IntersectionID,
) -> Result<Polygon> {
    let mut sorted_roads = Vec::new();
    for id in sorted_road_ids {
        sorted_roads.push(roads[id].to_road());
    }
    let mut edges = RoadEdge::calculate(sorted_roads.iter().collect(), i);
    edges.push(edges[0].clone());

    // Form the intersection polygon by using the endpoints of each road edge.
    let mut endpts = Vec::new();
    for pair in edges.windows(2) {
        let one = &pair[0];
        let two = &pair[1];

        endpts.push(one.pl.last_pt());

        if one.road != two.road {
            // But also, we want to use the original points where untrimmed road edges collided.
            // We didn't retain those in the main loop above. So instead, let's use the trimmed
            // edges. If the other side of a road produced a larger trim, this side won't collide.
            // So extend the side until it has the same length as the original untrimmed line. Note
            // all the reversing is to find the hit closest to the intersection.
            if let Some((corner, _)) = one
                .pl
                .extend_to_length(orig_centers[&one.road].length())
                .reversed()
                .intersection(
                    &two.pl
                        .extend_to_length(orig_centers[&two.road].length())
                        .reversed(),
                )
            {
                // When both roads lead between the same pair of endpoints, it's possible the
                // extended lines don't collide on the intersection side, but they do on the other
                // side. This can happen when the current side is pre-trimmed, for example. To deal
                // with this, see if the collision point is on the original polyline on the
                // incorrect half.
                //
                // For simplicity, we use one.pl, not extended. Extending only matters on the
                // correct intersection end, anyway.
                if let Some((dist, _)) = one.pl.dist_along_of_point(corner) {
                    if dist < one.pl.length() / 2.0 {
                        continue;
                    }
                }

                endpts.push(corner);
            }
        }
    }
    endpts.push(endpts[0]);
    Ok(Ring::deduping_new(endpts)?.into_polygon())
}
