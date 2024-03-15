use std::collections::BTreeSet;

use geom::PolyLine;

use osm2lanes::RoadPosition;

use crate::{
    BufferType, Direction, DrivingSide, LaneSpec, LaneType, Placement, RoadID, StreetNetwork,
};

/// Find dual carriageways that split very briefly and then re-join, with no intermediate roads.
/// Collapse them into one road with a barrier in the middle.
pub fn collapse_sausage_links(streets: &mut StreetNetwork) {
    for (id1, id2) in find_sausage_links(streets) {
        fix(streets, id1, id2);
    }
}

fn find_sausage_links(streets: &StreetNetwork) -> BTreeSet<(RoadID, RoadID)> {
    let mut pairs: BTreeSet<(RoadID, RoadID)> = BTreeSet::new();

    for road1 in streets.roads.values() {
        // TODO People often forget to fix the lanes when splitting a dual carriageway, but don't
        // attempt to detect/repair that yet.
        if road1.oneway_for_driving().is_none() {
            continue;
        }
        // Find roads that lead between the two endpoints
        let mut common_roads: BTreeSet<RoadID> =
            into_set(streets.intersections[&road1.src_i].roads.clone())
                .intersection(&into_set(streets.intersections[&road1.dst_i].roads.clone()))
                .cloned()
                .collect();
        // Normally it's just this one road
        assert!(common_roads.remove(&road1.id));
        // If there's many roads between these intersections, something weird is happening; ignore
        // it
        if common_roads.len() != 1 {
            continue;
        }

        let id2 = common_roads.into_iter().next().unwrap();
        // Ignore if we've already found this match
        if pairs.contains(&(id2, road1.id)) {
            continue;
        }

        let road2 = &streets.roads[&id2];
        if road2.oneway_for_driving().is_none() || road1.name != road2.name {
            continue;
        }

        // The two roads must point in a loop. Since they're both one-way, we can just
        // check the endpoints.
        // See the 'service_road_loop' test for why this is needed.
        if !(road1.dst_i == road2.src_i && road2.dst_i == road1.src_i) {
            continue;
        }

        // Both intersections must connect to something else. If one of them is degenerate, we
        // would collapse a loop down. See the 'oneway_loop' test for an example.
        if streets.roads_per_intersection(road1.src_i).len() < 3
            || streets.roads_per_intersection(road1.dst_i).len() < 3
        {
            continue;
        }

        // Don't collapse roundabouts. Since we don't preserve the junction=roundabout tag, another
        // heuristic is if the two roads came from the same original OSM way.
        // https://www.openstreetmap.org/way/235499756 is an example.
        if !road1.osm_ids.is_empty() && road1.osm_ids == road2.osm_ids {
            continue;
        }

        pairs.insert((road1.id, id2));
    }

    pairs
}

fn fix(streets: &mut StreetNetwork, id1: RoadID, id2: RoadID) {
    // We're never modifying intersections, so even if sausage links are clustered together, both
    // roads should always continue to exist as we fix things.
    assert!(streets.roads.contains_key(&id1));
    assert!(streets.roads.contains_key(&id2));

    // Arbitrarily remove the 2nd
    let mut road2 = streets.remove_road(id2);
    // And modify the 1st
    let road1 = streets.roads.get_mut(&id1).unwrap();

    road1.osm_ids.extend(road2.osm_ids);

    // Geometry
    //
    // Just make a straight line between the intersections. In OSM, usually the two pieces
    // bend away from the median in some unrealistic way.
    //
    // Alternate idea: Try to average the two PolyLines somehow
    road1.reference_line = PolyLine::must_new(vec![
        road1.reference_line.first_pt(),
        road1.reference_line.last_pt(),
    ]);
    road1.reference_line_placement = Placement::Consistent(RoadPosition::Center);

    // Lanes
    //
    // We need to append road2's lanes onto road1's.
    // - Fixing the direction of the lanes
    // - Handling mistagged or mis-inferred sidewalks
    // - Appending them on the left or the right?
    //
    // And this dual carriageway briefly appeared in the first place because of some kind of
    // barrier dividing the road -- maybe a pedestrian crossing island or a piece of concrete. For
    // now, always assume it's a curb.
    if streets.config.driving_side == DrivingSide::Right {
        // Assume there's not a sidewalk in the middle of the road
        if road1.lane_specs_ltr[0].lt == LaneType::Sidewalk {
            road1.lane_specs_ltr.remove(0);
        }
        if road2.lane_specs_ltr[0].lt == LaneType::Sidewalk {
            road2.lane_specs_ltr.remove(0);
        }

        // Insert a buffer to represent the split
        road1.lane_specs_ltr.insert(
            0,
            LaneSpec {
                lt: LaneType::Buffer(BufferType::Curb),
                dir: Direction::Forward,
                width: LaneSpec::typical_lane_width(LaneType::Buffer(BufferType::Curb)),
                allowed_turns: Default::default(),
                lane: None,
            },
        );

        for mut lane in road2.lane_specs_ltr {
            lane.dir = lane.dir.opposite();
            road1.lane_specs_ltr.insert(0, lane);
        }
    } else {
        if road1.lane_specs_ltr.last().unwrap().lt == LaneType::Sidewalk {
            road1.lane_specs_ltr.pop().unwrap();
        }
        road2.lane_specs_ltr.reverse();
        if road2.lane_specs_ltr[0].lt == LaneType::Sidewalk {
            road2.lane_specs_ltr.remove(0);
        }

        road1.lane_specs_ltr.push(LaneSpec {
            lt: LaneType::Buffer(BufferType::Curb),
            dir: Direction::Forward,
            width: LaneSpec::typical_lane_width(LaneType::Buffer(BufferType::Curb)),
            allowed_turns: Default::default(),
            lane: None,
        });

        for mut lane in road2.lane_specs_ltr {
            lane.dir = lane.dir.opposite();
            road1.lane_specs_ltr.push(lane);
        }
    }

    // Because we have modified the lanes of road1 we need to update the derived data.
    road1.update_center_line(streets.config.driving_side);
    let intersections = road1.endpoints();
    for i in intersections {
        streets.update_i(i);
    }
}

fn into_set<T: Ord>(list: Vec<T>) -> BTreeSet<T> {
    list.into_iter().collect()
}
