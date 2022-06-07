use crate::units::preamble::*;

use crate::network::RoadNetwork;
use crate::road_functions::IntersectionType;
use crate::road_functions::{ControlType, Intersection, RoadWay};
use crate::road_parts::{Carriage, Designation, Lane, RoadEdge, E};
use crate::units::{Direction, DrivingSide, Meters, Side, TrafficDirections};

use abstio::MapName;
use abstutil::Timer;
use geom::Distance;
use raw_map::{LaneSpec, LaneType, RawIntersection, RawMap, RawRoad};

use enum_map::{enum_map, EnumMap};
use itertools::Itertools;
use std::collections::HashMap;

/// ```
/// use abstutil::Timer;
/// use petgraph::dot::{Config, Dot};
/// use streets::io::load_road_network;
/// let mut timer = Timer::new("test osm2streets");
/// let mut net = load_road_network(String::from("tests/src/aurora_sausage_link/input.osm"), &mut timer);
/// println!("{}", net.to_dot());
pub fn load_road_network(osm_path: String, timer: &mut Timer) -> RoadNetwork {
    let driving_side = raw_map::DrivingSide::Right; // TODO
    let clip = None;

    let mut raw_map = convert_osm::convert(
        osm_path.clone(),
        MapName::new("zz", "osm2streets_test", &abstutil::basename(&osm_path)),
        clip,
        convert_osm::Options {
            map_config: map_model::MapConfig {
                driving_side,
                bikes_can_use_bus_lanes: true,
                inferred_sidewalks: true,
                street_parking_spot_length: Distance::meters(8.0),
                turn_on_red: true,
            },
            onstreet_parking: convert_osm::OnstreetParking::JustOSM,
            public_offstreet_parking: convert_osm::PublicOffstreetParking::None,
            private_offstreet_parking: convert_osm::PrivateOffstreetParking::FixedPerBldg(1),
            include_railroads: true,
            extra_buildings: None,
            skip_local_roads: false,
            filter_crosswalks: false,
            gtfs_url: None,
            elevation: false,
        },
        timer,
    );

    raw_map.run_all_simplifications(false, false, timer);

    raw_map.into()
}

impl From<RawMap> for RoadNetwork {
    fn from(map: RawMap) -> Self {
        let mut net = RoadNetwork::new();
        // Intersection ids from NodeIds
        let is = HashMap::<_, _>::from_iter(map.intersections.iter().map(|(node_id, raw_int)| {
            (node_id, net.add_intersection(Intersection::from(raw_int)))
        }));
        // RoadWay ids from OriginalRoads
        let _rs = HashMap::<_, _>::from_iter(map.roads.iter().map(|(rid, raw_road)| {
            let mut ways = RoadWay::pair_from(raw_road);
            (
                rid,
                (
                    ways[Forward]
                        .take()
                        .map(|f| net.add_closing_roadway(f.clone(), is[&rid.i1], is[&rid.i2])),
                    ways[Backward]
                        .take()
                        .map(|b| net.add_closing_roadway(b, is[&rid.i2], is[&rid.i1])),
                ),
            )
        }));
        net
    }
}

// ## Conversions

impl RoadWay {
    pub fn pair_from(r: &RawRoad) -> EnumMap<Direction, Option<RoadWay>> {
        let ds: DrivingSide = RHT; // TODO
        let mut lanes = r.lane_specs_ltr.iter();
        // lanes are ltr, so take the left lanes until we see one in the direction of the traffic
        // on the right. Then the right hand lanes will be remaining.
        let dir_on_right = match ds.get_direction(Right) {
            Forward => raw_map::Direction::Fwd,
            Backward => raw_map::Direction::Back,
        };
        let left_lanes = lanes
            .take_while_ref(|&l| l.dir == dir_on_right) // TODO car lanes only, better define
            .map(|l| E::Lane(l.into()))
            .collect::<Vec<_>>(); // Any middle buffer would end up at the end here...
        let right_lanes = lanes.map(|l| E::Lane(l.into())).collect::<Vec<_>>();
        let half_roads: EnumMap<Side, Vec<E>> = enum_map! {
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

impl From<&RawIntersection> for Intersection {
    fn from(raw_int: &RawIntersection) -> Self {
        Intersection {
            // raw_int.intersection_type has some useful info, bit is often misleading.
            t: match raw_int.intersection_type {
                raw_map::IntersectionType::Border => IntersectionType::MapEdge,
                raw_map::IntersectionType::TrafficSignal
                | raw_map::IntersectionType::Construction => IntersectionType::RoadIntersection,
                _ => IntersectionType::Unknown,
            },
            control: match raw_int.intersection_type {
                // IntersectionType::StopSign => ControlType::Signed, // wrong when it should be uncontrolled
                raw_map::IntersectionType::TrafficSignal => ControlType::Lights,
                _ => ControlType::Uncontrolled,
            },
        }

        // TODO do all needed transformations until it is correct (no unknowns, normal form).
    }
}

impl From<&LaneSpec> for Lane {
    fn from(l: &LaneSpec) -> Self {
        Lane {
            dir: if let LaneType::SharedLeftTurn = l.lt {
                // Lane type is used to represent the both-ways aspect of middle turn lanes, I guess.
                TrafficDirections::BothWays
            } else {
                // All our lanes are forward on their RoadWay, unless they are doing something fishy.
                TrafficDirections::Forward
            },

            designation: match l.lt {
                LaneType::Sidewalk => Designation::Travel(Carriage::Foot),
                LaneType::Biking => Designation::Travel(Carriage::Bike),
                LaneType::Driving | LaneType::SharedLeftTurn => Designation::Travel(Carriage::Cars),
                LaneType::Bus => Designation::Travel(Carriage::Bus),
                LaneType::LightRail => Designation::Travel(Carriage::Train),

                LaneType::Buffer(_) => Designation::NoTravel,
                LaneType::Shoulder => Designation::NoTravel, // ?

                LaneType::Parking => Designation::Parking(Carriage::Cars),
                LaneType::Construction => Designation::Amenity,
            },
            can_enter_from_inside: true,
            can_enter_from_outside: false,
            width: Meters::from(l.width.inner_meters()),
        }
    }
}
