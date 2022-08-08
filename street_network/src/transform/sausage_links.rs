use std::collections::BTreeSet;

use crate::{
    osm, BufferType, Direction, DrivingSide, LaneSpec, LaneType, OriginalRoad, StreetNetwork,
};

/// Find dual carriageways that split very briefly and then re-join, with no intermediate roads.
/// Collapse them into one road with a barrier in the middle.
pub fn collapse_sausage_links(streets: &mut StreetNetwork) {
    for (id1, id2) in find_sausage_links(streets) {
        // TODO Temporarily demonstrate debugging by labelling some things. Remove after checking
        // in a transformation that actually needs this.
        streets.debug_road(id1.clone(), "one side of sausage link");
        streets.debug_intersection(id1.i2, "one endpoint of sausage link");

        fix(streets, id1, id2);
    }
}

fn find_sausage_links(streets: &StreetNetwork) -> BTreeSet<(OriginalRoad, OriginalRoad)> {
    let mut pairs: BTreeSet<(OriginalRoad, OriginalRoad)> = BTreeSet::new();

    for (id1, road1) in &streets.roads {
        // TODO People often forget to fix the lanes when splitting a dual carriageway, but don't
        // attempt to detect/repair that yet.
        if road1.oneway_for_driving().is_none() {
            continue;
        }
        // Find roads that lead between the two endpoints
        let mut common_roads: BTreeSet<OriginalRoad> =
            into_set(streets.roads_per_intersection(id1.i1))
                .intersection(&into_set(streets.roads_per_intersection(id1.i2)))
                .cloned()
                .collect();
        // Normally it's just this one road
        assert!(common_roads.remove(id1));
        // If there's many roads between these intersections, something weird is happening; ignore
        // it
        if common_roads.len() != 1 {
            continue;
        }

        let id2 = common_roads.into_iter().next().unwrap();
        // Ignore if we've already found this match
        if pairs.contains(&(id2, *id1)) {
            continue;
        }

        let road2 = &streets.roads[&id2];
        if road2.oneway_for_driving().is_none()
            || road1.osm_tags.get(osm::NAME) != road2.osm_tags.get(osm::NAME)
        {
            continue;
        }

        // The two roads must point in a loop. Since they're both one-way, we can just
        // check the endpoints.
        // See the 'service_road_loop' test for why this is needed.
        if !(id1.i2 == id2.i1 && id2.i2 == id1.i1) {
            continue;
        }

        // Both intersections must connect to something else. If one of them is degenerate, we
        // would collapse a loop down. See the 'oneway_loop' test for an example.
        if streets.roads_per_intersection(id1.i1).len() < 3
            || streets.roads_per_intersection(id1.i2).len() < 3
        {
            continue;
        }

        pairs.insert((*id1, id2));
    }

    pairs
}

fn fix(streets: &mut StreetNetwork, id1: OriginalRoad, id2: OriginalRoad) {
    // We're never modifying intersections, so even if sausage links are clustered together, both
    // roads should always continue to exist as we fix things.
    assert!(streets.roads.contains_key(&id1));
    assert!(streets.roads.contains_key(&id2));

    // Arbitrarily remove the 2nd
    let mut road2 = streets.roads.remove(&id2).unwrap();
    // And modify the 1st
    let road1 = streets.roads.get_mut(&id1).unwrap();

    // Geometry
    //
    // Just make a straight line between the intersections. In OSM, usually the two pieces
    // bend away from the median in some unrealistic way.
    //
    // Alternate idea: Try to average the two PolyLines somehow
    road1.osm_center_points = vec![
        road1.osm_center_points[0],
        *road1.osm_center_points.last().unwrap(),
    ];

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
                dir: Direction::Fwd,
                width: LaneSpec::typical_lane_width(LaneType::Buffer(BufferType::Curb)),
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
            dir: Direction::Fwd,
            width: LaneSpec::typical_lane_width(LaneType::Buffer(BufferType::Curb)),
        });

        for mut lane in road2.lane_specs_ltr {
            lane.dir = lane.dir.opposite();
            road1.lane_specs_ltr.push(lane);
        }
    }

    // Tags
    // TODO We shouldn't need to modify road1's tags; lanes_ltr are the source of truth. But...
    // other pieces of code still treat tags as an "original" source of truth. In A/B Street,
    // reverting the road to its original state in the lane editor, for example, will get confused
    // here and only see the original road1.

    // IDs
    // TODO The IDs in StreetNetwork are based on original OSM IDs, but they diverge as we make
    // transformations like this. We could consider some combination of assigning new IDs all the
    // time and associating one RawRoad with multiple OSM IDs.
}

fn into_set<T: Ord>(list: Vec<T>) -> BTreeSet<T> {
    list.into_iter().collect()
}
