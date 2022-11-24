use std::collections::BTreeMap;

use geom::{Distance, Polygon, Pt2D};
use serde::{Deserialize, Serialize};

use crate::{osm, DrivingSide, IntersectionID, RoadID, StreetNetwork};
use TrafficConflict::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Intersection {
    pub id: IntersectionID,
    /// The OSM nodes making up this intersection. Multiple intersections may share the same OSM
    /// nodes (when an out-of-bounds intersection connected to multiple roads is clipped). One
    /// intersection may have multiple OSM nodes (when the intersection is consolidated).
    pub osm_ids: Vec<osm::NodeID>,

    /// Represents the original place where OSM center-lines meet. This may be meaningless beyond
    /// StreetNetwork; roads and intersections get merged and deleted.
    pub point: Pt2D,
    /// This will be a placeholder until `Transformation::GenerateIntersectionGeometry` runs.
    pub polygon: Polygon,
    pub kind: IntersectionKind,
    pub control: IntersectionControl,
    pub elevation: Distance,

    /// All roads connected to this intersection. They may be incoming or outgoing relative to this
    /// intersection. They're ordered clockwise aroundd the intersection.
    pub roads: Vec<RoadID>,
    pub movements: Vec<Movement>,

    // true if src_i matches this intersection (or the deleted/consolidated one, whatever)
    // TODO Store start/end trim distance on _every_ road
    pub trim_roads_for_merging: BTreeMap<(RoadID, bool), Pt2D>,
}

/// How two lanes of travel conflict with each other.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TrafficConflict {
    Uncontested,
    Diverge,
    Merge,
    Cross,
}

/// What kind of feature an `Intersection` actually represents. Any connection between roads in the
/// network graph is represented by an `Intersection`, but many of them are not traffic
/// "intersections" in the common sense.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum IntersectionKind {
    /// A `Road` ends because the road crosses the map boundary.
    MapEdge,

    /// A single `Road` ends because the actual roadway ends; "the end of the line".
    ///
    /// E.g. turning circles, road end signs, train terminus thingos, ...
    Terminus,

    /// Multiple `Road`s connect but no flow of traffic interacts with any other.
    ///
    /// Usually one `Road` ends and another begins because the number of lanes has changed or some
    /// other attribute of the roadway has changed. More than two `Road`s could be involved,
    /// e.g. when a single carriageway (a bidirectional `Road`) splits into a dual carriageway
    /// (two oneway `Road`s).
    Connection,

    /// One flow of traffic forks into multiple, or multiple merge into one, but all traffic is
    /// expected to keep flowing.
    ///
    /// E.g. highway on-ramps and off-ramps.
    Fork,

    /// At least three `Road`s meet at an actual "intersection" where at least one flow of traffic
    /// gives way to, or conflicts with, another.
    Intersection,
}

/// The kind of traffic control present at an intersection.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum IntersectionControl {
    Uncontrolled,
    Signed,
    Signalled,
    Construction,
}

/// The path that some group of adjacent lanes of traffic can take through an intersection.
pub type Movement = (RoadID, RoadID);

impl Intersection {
    pub fn is_map_edge(&self) -> bool {
        self.kind == IntersectionKind::MapEdge
    }
}

impl StreetNetwork {
    pub fn next_intersection_id(&mut self) -> IntersectionID {
        let id = IntersectionID(self.intersection_id_counter);
        self.intersection_id_counter += 1;
        id
    }

    /// This creates a new intersection based on one or more real OSM nodes, assigning an ID and
    /// returning it.
    pub fn insert_intersection(
        &mut self,
        osm_ids: Vec<osm::NodeID>,
        point: Pt2D,
        t: IntersectionKind,
        control: IntersectionControl,
    ) -> IntersectionID {
        let id = self.next_intersection_id();
        self.intersections.insert(
            id,
            Intersection {
                id,
                osm_ids,
                point,
                polygon: Polygon::dummy(),
                kind: t,
                control,
                // Filled out later
                roads: Vec::new(),
                movements: Vec::new(),
                elevation: Distance::ZERO,
                trim_roads_for_merging: BTreeMap::new(),
            },
        );
        id
    }

    // Restore the invariant that an intersection's roads are ordered clockwise
    //
    // TODO This doesn't handle trim_roads_for_merging
    pub fn sort_roads(&mut self, i: IntersectionID) {
        let intersection = self.intersections.get_mut(&i).unwrap();
        if intersection.roads.len() < 2 {
            return; // Already sorted.
        }

        // (ID, polyline pointing to the intersection, sorting point that's filled out later)
        let mut road_centers = Vec::new();
        let mut endpoints_for_center = Vec::new();
        for r in &intersection.roads {
            let road = &self.roads[r];
            // road.center_pts is unadjusted; it doesn't handle unequal widths yet. But that
            // shouldn't matter for sorting.
            let center_pl = if road.src_i == i {
                road.untrimmed_center_line.reversed()
            } else if road.dst_i == i {
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

    /// Updates the derived properties of an intersection.
    ///
    /// The kind and movements of a `MapEdge` are handled independently, so this method skips them.
    pub fn update_movements(&mut self, i: IntersectionID) {
        if self.intersections[&i].kind == IntersectionKind::MapEdge {
            return;
        }

        let (movements, kind) = self.calculate_movements_and_kind(i);
        let intersection = self.intersections.get_mut(&i).unwrap();
        intersection.movements = movements;
        intersection.kind = kind;
    }

    pub fn calculate_movements_and_kind(
        &self,
        i: IntersectionID,
    ) -> (Vec<Movement>, IntersectionKind) {
        let roads: Vec<_> = self
            .roads_per_intersection(i)
            .into_iter()
            .filter(|road| road.is_driveable())
            .collect();

        // A terminus is characterised by a single connected road.
        if roads.len() == 1 {
            return (Vec::new(), IntersectionKind::Terminus);
        }

        // Calculate all the possible movements, (except U-turns, for now).
        let mut connections = Vec::new();
        // Consider all pairs of roads, from s to d.
        // Identify them using their index in the list - which
        // is sorted in clockwise order - so that we can compare their position later.
        for s in 0..roads.len() {
            for d in 0..roads.len() {
                if s == d {
                    continue; // Ignore U-turns.
                }

                // Calculate if it is possible to emerge from s into the intersection.
                let src_road = roads[s];
                if !src_road.can_drive_out_of_end(i) {
                    continue;
                }

                // Calculate if it is possible to leave the intersection into d.
                let dst_road = roads[d];
                if !dst_road.can_drive_into_end(i) {
                    continue;
                }

                // TODO detect U-Turns that should be assumed forbidden.
                // if src and dst are oneway and
                // adjacent on the intersection and
                // ordered with the "insides" touching and
                // the angle between them is small enough.

                // Check for any turn restrictions.
                if src_road.allowed_to_turn_to(dst_road.id) {
                    connections.push((s, d));
                }
            }
        }

        // Calculate the highest level of conflict between movements.
        let mut worst_conflict = Uncontested;
        // Compare every unordered pair of connections. Use the order of the roads around the
        // intersection to detect if they diverge, merge, or cross.
        let mut each_con = connections.iter();
        while let Some(con_a) = each_con.next() {
            for con_b in each_con.clone() {
                worst_conflict = std::cmp::max(
                    worst_conflict,
                    calc_conflict(con_a, con_b, self.config.driving_side),
                );

                // Stop looking if we've already found the worst.
                if worst_conflict == Cross {
                    break;
                }
            }
        }

        (
            connections
                .iter()
                .map(|(s, d)| (roads[*s].id, roads[*d].id))
                .collect(),
            match worst_conflict {
                Uncontested => IntersectionKind::Connection,
                // TODO check for give way signs or count lanes to detect Intersections:
                Diverge | Merge => IntersectionKind::Fork,
                Cross => IntersectionKind::Intersection,
            },
        )
    }
}

/// Calculate how two turns through an intersection conflict. Turns are identified by the clockwise
/// index of their (src, dst) roads.
fn calc_conflict(a: &(usize, usize), b: &(usize, usize), side: DrivingSide) -> TrafficConflict {
    // If the traffic starts and ends at the same place in the same direction...
    if a.0 == b.0 && a.1 == b.1 {
        return Uncontested;
    }
    if a.0 == b.0 {
        return Diverge;
    }
    if a.1 == b.1 {
        return Merge;
    }

    // The intersection has a boundary that we have labelled 0 to n-1 in clockwise order (from an
    // arbitrary point), like a string laying in a circle. If we represent `a` as an arc from one
    // point on the string to another, then there is a section of the string between the two points,
    // connecting them and two ends of string "on the outside". A second arc, `b`, crosses `a` if
    // and only if `b` has one end between the points and one end outside.
    //     ______
    //    /  |   \
    //   |   |a   n
    //   |   |    0
    //    \__|___/

    // What if the traffic meets going in opposite directions?
    // It depends on where the traffic came from, and which side we're driving on.

    // Below: If a movement going in the other direction, `b`, joins the indicated LHT movement `a`
    // (at either end), it will join the road on the dotted side. Whether the other end of `b` is
    // between the endpoints of `a` or not corresponds to the crossing of the road.
    // Therefore, if `a` is drawn pointing upwards from low .0 to high .1,
    // then LHT would be crossed by movements joining from the "inside".
    //     ______          ______
    //    /  ^:  \        /  :|  \
    //   |  a|:   n      |   :|   n
    //   |   |:   0      |   :|a  0
    //    \__|:__/        \__:V__/

    // This equation (hopefully) works. Once it does, just trust it:
    // TODO unit test these three equations.
    let is_driving_side_between = (side == DrivingSide::Left) ^ (a.0 < a.1); // `==` or `^`?

    if a.0 == b.1 {
        return if is_driving_side_between ^ is_between(b.0, a) {
            Cross
        } else {
            Uncontested
        };
    }
    if a.1 == b.0 {
        return if is_driving_side_between ^ is_between(b.1, a) {
            Cross
        } else {
            Uncontested
        };
    }

    if is_between(a.0, b) ^ is_between(a.1, b) {
        return Cross;
    }
    return Uncontested;
}

fn is_between(num: usize, range: &(usize, usize)) -> bool {
    let bot = std::cmp::min(range.0, range.1);
    let top = std::cmp::max(range.0, range.1);
    return bot < num && num < top;
}
