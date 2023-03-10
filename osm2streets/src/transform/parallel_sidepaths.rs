use geom::{Distance, PolyLine};

use crate::{BufferType, Direction, IntersectionID, LaneSpec, LaneType, RoadID, StreetNetwork};

/// Find sidepath segments that exist as separate objects, parallel to a main road. Zip (or "snap")
/// them into the main road, inserting a buffer lane to represent the physical division.
pub fn zip_sidepaths(streets: &mut StreetNetwork) {
    for sidepath in find_sidepaths(streets) {
        streets.maybe_start_debug_step(format!("snap sidepath {}", sidepath.debug_idx));
        sidepath.debug(streets);
        snap(streets, sidepath);
    }
}

// We're only pattern matching on one type of separate sidepath right now. This represents a single
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
struct Sidepath {
    sidepath: RoadID,
    sidepath_center: PolyLine,
    // Just to distinguish different sidepaths when debugging
    debug_idx: usize,
    main_road_src_i: IntersectionID,
    main_road_dst_i: IntersectionID,
    main_roads: Vec<RoadID>,
}

impl Sidepath {
    fn debug(&self, streets: &mut StreetNetwork) {
        streets.debug_road(self.sidepath, format!("sidepath {}", self.debug_idx));
        streets.debug_intersection(self.main_road_src_i, format!("src_i of {}", self.debug_idx));
        streets.debug_intersection(self.main_road_dst_i, format!("dst_i of {}", self.debug_idx));
        for x in &self.main_roads {
            streets.debug_road(*x, format!("main road along {}", self.debug_idx));
        }
    }
}

fn find_sidepaths(streets: &StreetNetwork) -> Vec<Sidepath> {
    const SHORT_ROAD_THRESHOLD: Distance = Distance::const_meters(10.0);

    let mut sidepaths = Vec::new();
    for sidepath_road in streets.roads.values() {
        if sidepath_road.is_cycleway() {
            // Look at other roads connected to both endpoints. One of them should be "very short."
            let mut main_road_endpoints = Vec::new();
            for i in sidepath_road.endpoints() {
                let mut candidates = Vec::new();
                for road in streets.roads_per_intersection(i) {
                    if road.untrimmed_length() < SHORT_ROAD_THRESHOLD {
                        candidates.push(road.id);
                    }
                }
                if candidates.len() == 1 {
                    main_road_endpoints.push(streets.roads[&candidates[0]].other_side(i));
                }
            }

            if main_road_endpoints.len() == 2 {
                // Often the main road parallel to this sidepath segment is just one road, but it
                // might be more.
                let main_road_src_i = main_road_endpoints[0];
                let main_road_dst_i = main_road_endpoints[1];
                // Find all main road segments "parallel to" this sidepath, by pathfinding between
                // the main road intersections. We don't care about the order, but simple_path
                // does. In case it's one-way for driving, try both.
                if let Some(path) = streets
                    .simple_path(main_road_src_i, main_road_dst_i, &[LaneType::Driving])
                    .or_else(|| {
                        streets.simple_path(main_road_dst_i, main_road_src_i, &[LaneType::Driving])
                    })
                {
                    sidepaths.push(Sidepath {
                        sidepath: sidepath_road.id,
                        sidepath_center: sidepath_road.center_line.clone(),
                        debug_idx: sidepaths.len(),
                        main_road_src_i,
                        main_road_dst_i,
                        main_roads: path.into_iter().map(|(r, _)| r).collect(),
                    });
                }
            }
        }
    }
    sidepaths
}

fn snap(streets: &mut StreetNetwork, input: Sidepath) {
    // This analysis shouldn't modify other sidepaths when it works on one
    assert!(streets.roads.contains_key(&input.sidepath));

    // Remove the sidepath, but remember the lanes it contained
    let mut sidepath_lanes = streets.remove_road(input.sidepath).lane_specs_ltr;

    // TODO Preserve osm_ids

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
        // TODO Use https://wiki.openstreetmap.org/wiki/Proposed_features/separation if available
        lt: LaneType::Buffer(BufferType::Planters),
        dir: Direction::Fwd,
        width: LaneSpec::typical_lane_width(LaneType::Buffer(BufferType::Planters)),
        allowed_turns: Default::default(),
    };

    // For every main road segment corresponding to the sidepath, we need to insert these
    // sidepath_lanes somewhere.
    //
    // - Fixing the direction of the lanes
    // - Appending them on the left or right side (and "inside" the inferred sidewalk on the road)
    // - Inserting the buffer
    for r in input.main_roads {
        let main_road = &streets.roads[&r];
        // Which side is closer to the sidepath?
        let (left, right) = main_road
            .get_untrimmed_sides(streets.config.driving_side)
            .unwrap();
        // TODO georust has a way to check distance of linestrings. But for now, just check the
        // middles
        let snap_to_left = input.sidepath_center.middle().dist_to(left.middle())
            < input.sidepath_center.middle().dist_to(right.middle());

        // Does the sidepath point the same direction as this main road? We can use the left or
        // right side, doesn't matter.
        // TODO Check this logic very carefully; angles always lead to bugs. 90 is a very generous
        // definition of parallel. But we have a binary decision to make, so maybe we should even
        // use 180.
        let oriented_same_way = input
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
    }

    // After this transformation, we should run CollapseDegenerateIntersections to handle the
    // intersection where the side road originally crossed the sidepath, and TrimDeadendCycleways
    // to clean up any small cycle connection roads.
}

// Insert all of `insert` at `idx` in `target`
fn splice_in<T>(target: &mut Vec<T>, idx: usize, insert: Vec<T>) {
    let tail = target.split_off(idx);
    target.extend(insert);
    target.extend(tail);
}
