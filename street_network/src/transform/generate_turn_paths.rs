use crate::osm::NodeID;
use crate::Direction::{Back, Fwd};
use crate::DrivingSide::Left;
use crate::TurnType::*;
use crate::{
    osm, DrivingSide, IntersectionComplexity, LaneID, LaneSpec, LengthEnd, OriginalRoad, Road,
    StreetNetwork, TurnPath, TurnPathID, TurnType,
};
use geom::PolyLine;
use std::iter::zip;

/// Fills in some (TODO: all) valid TurnPaths through some (TODO: all) intersections.
pub fn generate_turn_paths(streets: &mut StreetNetwork) {
    let mut changes: Vec<(osm::NodeID, Vec<TurnPath>)> = Vec::new();
    for (id, _inter) in &streets.intersections {
        let roads = streets.roads_per_intersection(*id);
        // I am not using IntersectionComplexity here, because I think the turns will be needed to
        // determine the complexity, not the other way around.
        if roads.len() == 2 {
            let r1 = roads.get(0).unwrap();
            let r2 = roads.get(1).unwrap();
            changes.push((
                *id,
                connections_to_turn_paths(
                    default_lane_connections_through_connection(
                        streets.config.driving_side,
                        streets.roads.get(r1).unwrap(),
                        r1.i1 == *id,
                        streets.roads.get(r2).unwrap(),
                        r2.i2 == *id,
                    ),
                    r1,
                    r2,
                ),
            ));
            changes.push((
                *id,
                connections_to_turn_paths(
                    default_lane_connections_through_connection(
                        streets.config.driving_side,
                        streets.roads.get(r2).unwrap(),
                        r2.i2 == *id,
                        streets.roads.get(r1).unwrap(),
                        r1.i1 == *id,
                    ),
                    r2,
                    r1,
                ),
            ));
        }

        // TODO the rest of the intersections...
    }

    for (id, mut paths) in changes.into_iter() {
        streets
            .intersections
            .get_mut(&id)
            .unwrap()
            .turn_paths
            .append(&mut paths)
    }
}

/// Calculates the assumed connectivity between lanes at a Connection.
/// See https://wiki.openstreetmap.org/wiki/Relation:connectivity
fn default_lane_connections_through_connection(
    side: DrivingSide,
    src: &Road,
    reverse_src: bool,
    dst: &Road,
    reverse_dst: bool,
) -> Vec<(usize, usize, TurnType)> {
    // Get the forward facing lanes, ordered inside out, for both roads.
    let mut src_lanes: Vec<(usize, &LaneSpec)> = src
        .lane_specs_ltr
        .iter()
        .enumerate()
        .filter(|(_, lane)| lane.dir == if reverse_src { Fwd } else { Back })
        .collect();
    let mut dst_lanes: Vec<(usize, &LaneSpec)> = dst
        .lane_specs_ltr
        .iter()
        .enumerate()
        .filter(|(_, lane)| lane.dir == if reverse_dst { Fwd } else { Back })
        .collect();
    if src_lanes.len() == 0 || dst_lanes.len() == 0 {
        return Vec::new();
    }
    if side == Left {
        src_lanes.reverse();
        dst_lanes.reverse();
    }

    // TODO use placement tags if present, because that tells you exactly which lanes join up.

    // Naively pair lanes up, starting at the inside of the road.
    let mut result: Vec<(usize, usize, TurnType)> = zip(&src_lanes, &dst_lanes)
        .map(|(s, d)| (s.0, d.0, Straight))
        .collect();
    let len_s = &src_lanes.len();
    let len_d = &dst_lanes.len();
    if len_s > len_d {
        // Merge additional outside source lanes into the outside destination lane.
        let dst = dst_lanes.last().unwrap();
        for i in 0..len_s - len_d {
            result.push((
                src_lanes.get(len_d + i).unwrap().0,
                dst.0,
                if side == Left {
                    SlightRight
                } else {
                    SlightLeft
                },
            ))
        }
    }
    if len_d > len_s {
        // Let the outside source lane change into the added destination lanes.
        let src = src_lanes.last().unwrap();
        for i in 0..len_d - len_s {
            result.push((
                src.0,
                dst_lanes.get(len_s + i).unwrap().0,
                if side == Left {
                    SlightLeft
                } else {
                    SlightRight
                },
            ))
        }
    }
    result
}

fn connections_to_turn_paths(
    connections: Vec<(usize, usize, TurnType)>,
    src_id: &OriginalRoad,
    dst_id: &OriginalRoad,
) -> Vec<TurnPath> {
    // Map the connections to TurnPaths
    connections
        .into_iter()
        .map(|(src_idx, dst_idx, turn_type)| TurnPath {
            id: TurnPathID {
                src: LaneID {
                    road_id: *src_id,
                    offset: src_idx,
                },
                dst: LaneID {
                    road_id: *dst_id,
                    offset: dst_idx,
                },
            },
            turn_type,
        })
        .collect()
}
