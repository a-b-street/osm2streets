use std::collections::{HashMap, HashSet};

use abstio::MapName;
use abstutil::Timer;
use anyhow::{bail, Result};
use convert_osm::reader;
use enum_map::{enum_map, EnumMap};
use geom::Distance;
use raw_map::{DrivingSide, RawIntersection, RawMap, RawRoad};
use serde::Deserialize;

use crate::network::RoadNetwork;
use crate::road_functions::IntersectionType;
use crate::road_functions::{ControlType, Intersection, RoadWay};
use crate::road_parts::RoadEdge;
use crate::units::preamble::{Backward, Forward, Left, Right};
use crate::units::Direction;

// use crate::osm_geom::{get_multipolygon_members, glue_multipolygon, multipoly_geometry};

/// ```
/// use abstutil::Timer;
/// use petgraph::dot::{Config, Dot};
/// use streets::io::load_road_network;
/// let mut timer = Timer::new("test osm2streets");
/// let mut net = load_road_network(String::from("tests/src/aurora_sausage_link/input.osm"), &mut timer);
/// println!("{}", Dot::new(&map.graph));
pub fn load_road_network(path: String, timer: &mut Timer) -> RoadNetwork {
    let driving_side = DrivingSide::Left; // TODO get driving side from country containing lat lon?
    let clip = None;

    let mut raw_map = convert_osm::convert(
        path,
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

impl From<&RawMap> for RoadNetwork {
    fn from(map: &RawMap) -> Self {
        let mut net = RoadNetwork::new();
        /// Intersection ids from NodeIds
        let is = HashMap::from_iter(map.intersections.iter().map(|(node_id, raw_int)| {
            (node_id, net.add_intersection(Intersection::from(raw_int)))
        }));
        /// RoadWay ids from OriginalRoads
        let rs = HashMap::from_iter(map.roads.iter().map(|(rid, raw_road)| {
            let (fw, bw) = RoadWay::pair_from(raw_road);
            (
                rid,
                (
                    fw.map(|f| net.add_closing_roadway(f, is[rid.i1], is[rid.i2])),
                    bw.map(|b| net.add_closing_roadway(b, is[rid.i2], is[rid.i1])),
                ),
            )
        }));
        net
    }
}

// ## Conversions

impl RoadWay {
    pub fn pair_from(r: &RawRoad) -> EmumMap<Direction, Option<RoadWay>> {
        //
        let mut lanes = r.lane_specs_ltr.iter();
        let ds: DrivingSide;
        // lanes are ltr, so take the left lanes until we see one in the direction of the traffic
        // on the right. Then the right hand lanes will be remaining.
        let half_roads = enum_map! {
            Left => lanes
                .peeking_take_while(|l| l.dir == ds.direction_on(Right))
                .map() // to forward lane or buffer
                .collect(), // Any middle buffer would end up at the end here...
            Right => lanes
                .map() // to backward lane or buffer
                .collect(),
        };

        has_half = half_roads.map(|ls| ls.len() > 0);
        let fs = ds.side_of(Forward);
        let bs = ds.side_of(Backward);
        enum_map! {
            Forward => if has_half[fs] {
                Some(RoadWay {
                    inner: if has_half[bs] { RoadEdge::Join } else { RoadEdge::Sudden },
                    elements: half_roads[fs],
                    outer: RoadEdge::Sudden,
                })
            },
            Backward => if has_half[bs] {
                Some(RoadWay {
                    inner: if has_half[fs] { RoadEdge::Join } else { RoadEdge::Sudden },
                    elements: half_roads[bs],
                    outer: RoadEdge::Sudden,
                })
            },
        }
    }
}

impl From<&RawIntersection> for Intersection {
    fn from(raw_int: &RawIntersection) -> Self {
        Intersection {
            // raw_int.intersection_type has some useful info, bit is often misleading.
            t: IntersectionType::Incomplete,
            control: match raw_int.intersection_type {
                // IntersectionType::StopSign => ControlType::Signed, // wrong when it should be uncontrolled
                raw_map::IntersectionType::TrafficSignal => ControlType::Lights,
                _ => ControlType::Uncontrolled,
            },
        }
    }
}
