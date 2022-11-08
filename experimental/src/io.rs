use anyhow::Result;
use enum_map::{enum_map, EnumMap};
use itertools::Itertools;
use std::collections::HashMap;

use abstutil::Timer;
use osm2streets::{osm, LaneSpec, LaneType, OriginalRoad, Road, StreetNetwork, Transformation};

use crate::network::RoadNetwork;
use crate::road_functions::IntersectionType;
use crate::road_functions::{ControlType, Intersection, RoadWay};
use crate::road_parts::{Carriage, Designation, RoadEdge, RoadPart};
use crate::units::preamble::*;
use crate::units::{Direction, DrivingSide, Meters, Side, TrafficDirections};

/// ```
/// use abstutil::Timer;
/// use petgraph::dot::{Config, Dot};
/// use experimental::io::load_road_network;
/// let mut timer = Timer::new("test osm2streets");
/// let mut net = load_road_network(String::from("tests/src/aurora_sausage_link/input.osm"), &mut timer).unwrap();
/// println!("{}", net.to_dot());
pub fn load_road_network(osm_path: String, timer: &mut Timer) -> Result<RoadNetwork> {
    let clip_pts = None;

    let mut street_network = streets_reader::osm_to_street_network(
        &std::fs::read_to_string(osm_path).unwrap(),
        clip_pts,
        streets_reader::Options::default(),
        timer,
    )?;

    street_network.apply_transformations(Transformation::standard_for_clipped_areas(), timer);

    Ok(street_network.into())
}

impl From<StreetNetwork> for RoadNetwork {
    fn from(streets: StreetNetwork) -> Self {
        let mut net = RoadNetwork::new();
        let intersections: HashMap<&osm::NodeID, _> = streets
            .intersections
            .iter()
            .map(|(node_id, int)| (node_id, net.add_intersection(Intersection::from(int))))
            .collect();
        let _road_ways: HashMap<&OriginalRoad, _> = streets
            .roads
            .iter()
            .map(|(rid, road)| {
                let mut ways = RoadWay::pair_from(road, streets.config.driving_side);
                (
                    rid,
                    (
                        ways[Forward].take().map(|f| {
                            net.add_closing_roadway(
                                f.clone(),
                                intersections[&rid.i1],
                                intersections[&rid.i2],
                            )
                        }),
                        ways[Backward].take().map(|b| {
                            net.add_closing_roadway(
                                b,
                                intersections[&rid.i2],
                                intersections[&rid.i1],
                            )
                        }),
                    ),
                )
            })
            .collect();
        net
    }
}

// ## Conversions

impl RoadWay {
    pub fn pair_from(
        r: &Road,
        driving_side: osm2streets::DrivingSide,
    ) -> EnumMap<Direction, Option<RoadWay>> {
        let ds = DrivingSide::from(driving_side);
        let mut lanes = r.lane_specs_ltr.iter();
        // lanes are ltr, so take the left lanes until we see one in the direction of the traffic
        // on the right. Then the right hand lanes will be remaining.
        let dir_on_right = match ds.get_direction(Right) {
            Forward => osm2streets::Direction::Fwd,
            Backward => osm2streets::Direction::Back,
        };
        let left_lanes = lanes
            .take_while_ref(|&l| match l.lt {
                LaneType::Driving | LaneType::Bus => l.dir != dir_on_right,
                _ => true,
            })
            .map(|l| l.into())
            .collect::<Vec<_>>(); // Any middle buffer would end up at the end here...
        let right_lanes = lanes.map(|l| l.into()).collect::<Vec<_>>();
        let half_roads: EnumMap<Side, Vec<RoadPart>> = enum_map! {
            Left => left_lanes.clone(),
            Right => right_lanes.clone(),
        };
        // TODO set no overtaking for the divider if needed.

        let has_half = enum_map! {
            Left => half_roads[Left].len() > 0,
            Right => half_roads[Right].len() > 0,
        };
        let fs = ds.get_side(Forward);
        let bs = ds.get_side(Backward);
        enum_map! {
            Forward => if has_half[fs] {
                Some(RoadWay {
                    inner: if has_half[bs] { RoadEdge::Join } else { RoadEdge::Sudden },
                    elements: half_roads[fs].clone(),
                    outer: RoadEdge::Sudden,
                })
            } else { None},
            Backward => if has_half[bs] {
                Some(RoadWay {
                    inner: if has_half[fs] { RoadEdge::Join } else { RoadEdge::Sudden },
                    elements: half_roads[bs].clone(),
                    outer: RoadEdge::Sudden,
                })
            } else { None },
        }
    }
}

impl From<&osm2streets::Intersection> for Intersection {
    fn from(int: &osm2streets::Intersection) -> Self {
        Self {
            // int.intersection_type has some useful info, bit is often misleading.
            t: match int.control {
                osm2streets::ControlType::Border => IntersectionType::MapEdge,
                osm2streets::ControlType::TrafficSignal
                | osm2streets::ControlType::Construction => IntersectionType::RoadIntersection,
                _ => IntersectionType::Unknown,
            },
            control: match int.control {
                // IntersectionType::StopSign => ControlType::Signed, // wrong when it should be uncontrolled
                osm2streets::ControlType::TrafficSignal => ControlType::Lights,
                _ => ControlType::Uncontrolled,
            },
        }

        // TODO do all needed transformations until it is correct (no unknowns, normal form).
    }
}

impl From<&LaneSpec> for RoadPart {
    fn from(l: &LaneSpec) -> Self {
        RoadPart {
            // All our lanes are forward on their RoadWay, unless they are doing something fishy.
            // Lane type is used to represent the both-ways aspect of middle turn lanes.
            designation: match l.lt {
                LaneType::Sidewalk => Designation::Travel {
                    carriage: Carriage::Foot,
                    direction: TrafficDirections::Forward,
                },
                LaneType::Biking => Designation::Travel {
                    carriage: Carriage::Bike,
                    direction: TrafficDirections::Forward,
                },
                LaneType::Driving => Designation::Travel {
                    carriage: Carriage::Cars,
                    direction: TrafficDirections::Forward,
                },
                LaneType::SharedLeftTurn => Designation::Travel {
                    carriage: Carriage::Cars,
                    direction: TrafficDirections::BothWays,
                },
                LaneType::Bus => Designation::Travel {
                    carriage: Carriage::Bus,
                    direction: TrafficDirections::Forward,
                },
                LaneType::LightRail => Designation::Travel {
                    carriage: Carriage::Train,
                    direction: TrafficDirections::Forward,
                },

                LaneType::Buffer(_) => Designation::NoTravel,
                LaneType::Shoulder => Designation::NoTravel, // ?

                LaneType::Parking => Designation::Parking {
                    carriage: Carriage::Cars,
                },
                LaneType::Construction => Designation::NoTravel,
                LaneType::Footway => Designation::Travel {
                    carriage: Carriage::Foot,
                    direction: TrafficDirections::BothWays,
                },
                LaneType::SharedUse => Designation::Travel {
                    // TODO Both foot and bike
                    carriage: Carriage::Foot,
                    direction: TrafficDirections::BothWays,
                },
            },
            can_enter_from_inside: true,
            can_enter_from_outside: false,
            width: Meters::from(l.width.inner_meters()),
        }
    }
}

impl From<osm2streets::DrivingSide> for DrivingSide {
    fn from(s: osm2streets::DrivingSide) -> Self {
        match s {
            osm2streets::DrivingSide::Right => RHT,
            osm2streets::DrivingSide::Left => LHT,
        }
    }
}
