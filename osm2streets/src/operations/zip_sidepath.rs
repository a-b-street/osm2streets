use std::collections::BTreeSet;

use geom::{Distance, PolyLine};

use crate::{BufferType, Direction, IntersectionID, LaneSpec, LaneType, RoadID, StreetNetwork};

// We're only pattern matching on one type of parallel sidepath right now. This represents a single
// Road that's parallel to one or more main_roads.
//
// X--X
// S  M
// S  M
// S  M
// S  X
// S  M
// S  M
// X--X
//
// S is the sidepath segment. X are intersections. M are main roads -- note there are two matching
// up to this one sidepath. The '-'s are short connector roads between the two.
pub struct Sidepath {
    sidepath: RoadID,
    sidepath_center: PolyLine,
    main_road_src_i: IntersectionID,
    main_road_dst_i: IntersectionID,
    main_roads: Vec<RoadID>,
    connector_src_i: Option<RoadID>,
    connector_dst_i: Option<RoadID>,
}

impl Sidepath {
    pub fn new(streets: &StreetNetwork, start: RoadID) -> Option<Self> {
        const SHORT_ROAD_THRESHOLD: Distance = Distance::const_meters(10.0);

        let sidepath_road = &streets.roads[&start];

        // Look at other roads connected to both endpoints. One of them should be "very short."
        let mut main_road_endpoints = Vec::new();
        for i in sidepath_road.endpoints() {
            let mut connector_candidates = Vec::new();
            for road in streets.roads_per_intersection(i) {
                if road.id != sidepath_road.id && road.center_line.length() < SHORT_ROAD_THRESHOLD {
                    connector_candidates.push(road.id);
                }
            }
            if connector_candidates.len() == 1 {
                let connector = connector_candidates[0];
                main_road_endpoints
                    .push((streets.roads[&connector].other_side(i), Some(connector)));
            } else if connector_candidates.is_empty() {
                // Maybe this intersection has been merged already. Use it directly.
                main_road_endpoints.push((i, None));
            }
        }

        if main_road_endpoints.len() != 2 {
            return None;
        }

        // Often the main road parallel to this sidepath segment is just one road, but it might
        // be more.
        let (main_road_src_i, connector_src_i) = main_road_endpoints[0];
        let (main_road_dst_i, connector_dst_i) = main_road_endpoints[1];
        // It may be none at all, when the main road intersection gets merged
        if main_road_src_i == main_road_dst_i {
            return None;
        }

        // Find all main road segments "parallel to" this sidepath, by pathfinding between the
        // main road intersections. We don't care about the order, but simple_path does. In
        // case it's one-way for driving, try both.
        if let Some(path) = streets
            .simple_path(main_road_src_i, main_road_dst_i, &[LaneType::Driving])
            .or_else(|| streets.simple_path(main_road_dst_i, main_road_src_i, &[LaneType::Driving]))
        {
            return Some(Self {
                sidepath: sidepath_road.id,
                sidepath_center: sidepath_road.center_line.clone(),
                main_road_src_i,
                main_road_dst_i,
                main_roads: path.into_iter().map(|(r, _)| r).collect(),
                connector_src_i,
                connector_dst_i,
            });
        }

        None
    }

    pub fn debug(&self, streets: &mut StreetNetwork, label: String) {
        streets.debug_road(self.sidepath, format!("sidepath {label}"));
        streets.debug_intersection(self.main_road_src_i, format!("src_i of {label}"));
        streets.debug_intersection(self.main_road_dst_i, format!("dst_i of {label}"));
        for x in &self.main_roads {
            streets.debug_road(*x, format!("main road along {label}"));
        }
        if let Some(r) = self.connector_src_i {
            streets.debug_road(r, format!("src_i connector of {label}"));
        }
        if let Some(r) = self.connector_dst_i {
            streets.debug_road(r, format!("dst_i connector of {label}"));
        }
    }

    pub fn zip(self, streets: &mut StreetNetwork) {
        // The caller may find a bunch of these and zip, which sometimes could delete one of the
        // located pieces
        if !streets.roads.contains_key(&self.sidepath) {
            return;
        }

        // Remove the sidepath, but remember the lanes it contained
        let mut sidepath_lanes = streets.remove_road(self.sidepath).lane_specs_ltr;

        // TODO Preserve osm_ids

        // TODO Re-evaluate this!
        // The sidepath likely had shoulder lanes assigned to it by get_lane_specs_ltr, because we have
        // many partially competing strategies for representing shared walking/cycling roads. Remove
        // those.
        if sidepath_lanes[0].lt == LaneType::Shoulder {
            sidepath_lanes.remove(0);
        }
        if sidepath_lanes.last().as_ref().unwrap().lt == LaneType::Shoulder {
            sidepath_lanes.pop();
        }

        // The sidepath was tagged as a separate way due to some kind of physical separation. We'll
        // represent that with a buffer lane.
        let buffer = LaneSpec {
            // TODO Use https://wiki.openstreetmap.org/wiki/Proposed_features/separation if
            // available
            lt: LaneType::Buffer(BufferType::Planters),
            dir: Direction::Forward,
            width: LaneSpec::typical_lane_width(LaneType::Buffer(BufferType::Planters)),
            allowed_turns: Default::default(),
            lane: None,
        };

        // For every main road segment corresponding to the sidepath, we need to insert these
        // sidepath_lanes somewhere.
        //
        // - Fixing the direction of the lanes
        // - Appending them on the left or right side (and "inside" the inferred sidewalk on the road)
        // - Inserting the buffer
        let mut intersections = BTreeSet::new();
        for r in self.main_roads {
            let main_road = &streets.roads[&r];
            // Which side is closer to the sidepath?
            let (left, right) = main_road
                .get_untrimmed_sides(streets.config.driving_side)
                .unwrap();
            // TODO georust has a way to check distance of linestrings. But for now, just check the
            // middles
            let snap_to_left = self.sidepath_center.middle().dist_to(left.middle())
                < self.sidepath_center.middle().dist_to(right.middle());

            // Does the sidepath point the same direction as this main road? We can use the left or
            // right side, doesn't matter.
            // TODO Check this logic very carefully; angles always lead to bugs. 90 is a very generous
            // definition of parallel. But we have a binary decision to make, so maybe we should even
            // use 180.
            let oriented_same_way = self
                .sidepath_center
                .overall_angle()
                .approx_eq(left.overall_angle(), 90.0);

            // Where should we insert the sidepath lanes? If the main road already has a sidewalk,
            // let's assume it should stay at the outermost part of the road. (That isn't always true,
            // but it's an assumption we'll take for now.)
            let insert_idx = if snap_to_left {
                if main_road.lane_specs_ltr[0].lt.is_walkable() {
                    1
                } else {
                    0
                }
            } else {
                if main_road
                    .lane_specs_ltr
                    .last()
                    .as_ref()
                    .unwrap()
                    .lt
                    .is_walkable()
                {
                    main_road.lane_specs_ltr.len() - 1
                } else {
                    main_road.lane_specs_ltr.len()
                }
            };

            streets.debug_road(r, format!("snap_to_left = {snap_to_left}, oriented_same_way = {oriented_same_way}, insert_idx = {insert_idx}"));

            // This logic thankfully doesn't depend on driving side at all!
            let mut insert_lanes = Vec::new();
            for mut lane in sidepath_lanes.clone() {
                if !oriented_same_way {
                    lane.dir = lane.dir.opposite();
                }
                insert_lanes.push(lane);
            }
            // TODO Do we ever need to reverse the order of the lanes?
            let mut buffer_lane = buffer.clone();
            if snap_to_left {
                // TODO I'm not sure what direction the buffer lane should face. This is a very strong
                // argument for Direction::Both.
                buffer_lane.dir = insert_lanes.last().as_ref().unwrap().dir;
                insert_lanes.push(buffer_lane);
            } else {
                buffer_lane.dir = insert_lanes[0].dir;
                insert_lanes.insert(0, buffer_lane);
            }

            let main_road = streets.roads.get_mut(&r).unwrap();
            splice_in(&mut main_road.lane_specs_ltr, insert_idx, insert_lanes);

            intersections.extend(main_road.endpoints());
        }

        // Recalculate geometry along all of the main roads we just thickened
        for i in intersections {
            streets.update_i(i);
        }

        // After this transformation, we should run CollapseDegenerateIntersections to handle the
        // intersection where the side road originally crossed the sidepath, and TrimDeadendCycleways
        // to clean up any small cycle connection roads.
        //
        // ALTERNATIVELY, remove the connector segments immediately.
        if let Some(r) = self.connector_src_i {
            if streets.roads.contains_key(&r) {
                // Ignore errors
                let _ = streets.collapse_short_road(r);
            }
        }
        if let Some(r) = self.connector_dst_i {
            if streets.roads.contains_key(&r) {
                let _ = streets.collapse_short_road(r);
            }
        }
    }
}

// Insert all of `insert` at `idx` in `target`
fn splice_in<T>(target: &mut Vec<T>, idx: usize, insert: Vec<T>) {
    let tail = target.split_off(idx);
    target.extend(insert);
    target.extend(tail);
}
