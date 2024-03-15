#[macro_use]
extern crate anyhow;

mod algorithm;
mod edit;
pub mod osm;
mod placement;
#[cfg(test)]
mod tests;
mod turns;

use enumset::{EnumSet, EnumSetType};
use muv_osm::lanes::Lane;
use std::fmt;

use serde::{Deserialize, Serialize};

use geom::Distance;

pub use algorithm::get_lane_specs_ltr;

pub const NORMAL_LANE_THICKNESS: Distance = Distance::const_meters(3.0);
const SERVICE_ROAD_LANE_THICKNESS: Distance = Distance::const_meters(2.0);
pub const SIDEWALK_THICKNESS: Distance = Distance::const_meters(1.5);
const SHOULDER_THICKNESS: Distance = Distance::const_meters(0.5);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LaneType {
    Driving,
    Parking,
    Sidewalk,
    // Walkable like a Sidewalk, but very narrow. Used to model pedestrians walking on roads
    // without sidewalks.
    Shoulder,
    Biking,
    Bus,
    SharedLeftTurn,
    Construction,
    LightRail,
    Buffer(BufferType),
    /// Some kind of pedestrian-only path unassociated with a road
    Footway,
    /// Some kind of shared pedestrian+bicycle space. May be associated with a road or not. Unclear
    /// which mode has effective priority.
    SharedUse,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BufferType {
    /// Just paint!
    Stripes,
    /// Flex posts, wands, cones, other "weak" forms of protection. Can weave through them.
    FlexPosts,
    /// Sturdier planters, with gaps.
    Planters,
    /// Solid barrier, no gaps.
    JerseyBarrier,
    /// A raised curb
    Curb,
    /// Non-road surface, between the road and footpath or within a median.
    Verge,
}

impl LaneType {
    pub fn is_for_moving_vehicles(self) -> bool {
        match self {
            LaneType::Driving => true,
            LaneType::Biking => true,
            LaneType::Bus => true,
            LaneType::Parking => false,
            LaneType::Sidewalk => false,
            LaneType::Shoulder => false,
            LaneType::SharedLeftTurn => false,
            LaneType::Construction => false,
            LaneType::LightRail => true,
            LaneType::Buffer(_) => false,
            LaneType::Footway => false,
            LaneType::SharedUse => true,
        }
    }

    pub fn supports_any_movement(self) -> bool {
        match self {
            LaneType::Driving => true,
            LaneType::Biking => true,
            LaneType::Bus => true,
            LaneType::Parking => false,
            LaneType::Sidewalk => true,
            LaneType::Shoulder => true,
            LaneType::SharedLeftTurn => false,
            LaneType::Construction => false,
            LaneType::LightRail => true,
            LaneType::Buffer(_) => false,
            LaneType::Footway => true,
            LaneType::SharedUse => true,
        }
    }

    /// Determines if the lane is a travel lane that is represented in OSM `*:lanes` tags.
    /// Note that the `lanes` tag counts car driving lanes, excluding bike lanes, whereas the
    /// `:lanes` suffix specifies that each lane, including bike lanes, should have a value between
    /// `|`s. This function identifies the latter kind.
    pub fn is_tagged_by_lanes_suffix(&self) -> bool {
        match self {
            LaneType::Driving => true,
            LaneType::Biking => true, // FIXME depends on lane vs track
            LaneType::Bus => true,
            LaneType::Parking => false,
            LaneType::Sidewalk => false,
            LaneType::Shoulder => false,
            LaneType::SharedLeftTurn => true,
            LaneType::Construction => true,
            LaneType::LightRail => false,
            LaneType::Buffer(_) => false,
            LaneType::Footway => false,
            LaneType::SharedUse => false,
        }
    }

    /// Determines if the lane is part of the roadway, the contiguous sealed surface that OSM
    /// mappers consider the "road".
    pub fn is_roadway(&self) -> bool {
        match self {
            LaneType::Driving => true,
            LaneType::Biking => true, // FIXME depends on lane vs track
            LaneType::Bus => true,
            LaneType::Parking => true, // FIXME depends on on-street vs street-side
            LaneType::Sidewalk => false,
            LaneType::Shoulder => true,
            LaneType::SharedLeftTurn => true,
            LaneType::Construction => true,
            LaneType::LightRail => true, // FIXME only for trams
            LaneType::Buffer(BufferType::Curb) => false,
            LaneType::Buffer(BufferType::Verge) => false,
            LaneType::Buffer(_) => true,
            LaneType::Footway => false,
            LaneType::SharedUse => false,
        }
    }

    pub fn is_walkable(self) -> bool {
        matches!(
            self,
            LaneType::Sidewalk | LaneType::Shoulder | LaneType::Footway | LaneType::SharedUse
        )
    }

    /// The most significant class of traffic that travels in this lane.
    // I don't know about parking lanes yet...
    pub fn traffic_class(&self) -> Option<TrafficClass> {
        use LaneType::*;
        match self {
            Footway | Sidewalk => Some(TrafficClass::Pedestrian),
            SharedUse | Biking => Some(TrafficClass::Bicycle),
            Bus | SharedLeftTurn | Driving => Some(TrafficClass::Motor),
            LightRail => Some(TrafficClass::Rail),
            Buffer(_) | Shoulder | Construction | Parking => None,
        }
    }

    pub fn describe(self) -> &'static str {
        match self {
            LaneType::Driving => "a general-purpose driving lane",
            LaneType::Biking => "a bike lane",
            LaneType::Bus => "a bus-only lane",
            LaneType::Parking => "an on-street parking lane",
            LaneType::Sidewalk => "a sidewalk",
            LaneType::Shoulder => "a shoulder",
            LaneType::SharedLeftTurn => "a shared left-turn lane",
            LaneType::Construction => "a lane that's closed for construction",
            LaneType::LightRail => "a light rail track",
            LaneType::Buffer(BufferType::Stripes) => "striped pavement",
            LaneType::Buffer(BufferType::FlexPosts) => "flex post barriers",
            LaneType::Buffer(BufferType::Planters) => "planter barriers",
            LaneType::Buffer(BufferType::JerseyBarrier) => "a Jersey barrier",
            LaneType::Buffer(BufferType::Curb) => "a raised curb",
            LaneType::Buffer(BufferType::Verge) => "a grassy verge",
            LaneType::Footway => "a footway",
            LaneType::SharedUse => "a shared-use walking/cycling path",
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            LaneType::Driving => "driving lane",
            LaneType::Biking => "bike lane",
            LaneType::Bus => "bus lane",
            LaneType::Parking => "parking lane",
            LaneType::Sidewalk => "sidewalk",
            LaneType::Shoulder => "shoulder",
            LaneType::SharedLeftTurn => "left-turn lane",
            LaneType::Construction => "construction",
            LaneType::LightRail => "light rail track",
            LaneType::Buffer(BufferType::Stripes) => "stripes",
            LaneType::Buffer(BufferType::FlexPosts) => "flex posts",
            LaneType::Buffer(BufferType::Planters) => "planters",
            LaneType::Buffer(BufferType::JerseyBarrier) => "Jersey barrier",
            LaneType::Buffer(BufferType::Curb) => "curb",
            LaneType::Buffer(BufferType::Verge) => "verge",
            LaneType::Footway => "footway",
            LaneType::SharedUse => "shared-use path",
        }
    }

    pub fn from_short_name(x: &str) -> Option<LaneType> {
        match x {
            "driving lane" => Some(LaneType::Driving),
            "bike lane" => Some(LaneType::Biking),
            "bus lane" => Some(LaneType::Bus),
            "parking lane" => Some(LaneType::Parking),
            "sidewalk" => Some(LaneType::Sidewalk),
            "shoulder" => Some(LaneType::Shoulder),
            "left-turn lane" => Some(LaneType::SharedLeftTurn),
            "construction" => Some(LaneType::Construction),
            "light rail track" => Some(LaneType::LightRail),
            "stripes" => Some(LaneType::Buffer(BufferType::Stripes)),
            "flex posts" => Some(LaneType::Buffer(BufferType::FlexPosts)),
            "planters" => Some(LaneType::Buffer(BufferType::Planters)),
            "Jersey barrier" => Some(LaneType::Buffer(BufferType::JerseyBarrier)),
            "curb" => Some(LaneType::Buffer(BufferType::Curb)),
            "verge" => Some(LaneType::Buffer(BufferType::Verge)),
            "footway" => Some(LaneType::Footway),
            "shared-use path" => Some(LaneType::SharedUse),
            _ => None,
        }
    }

    /// Represents the lane type as a single character, for use in tests.
    pub fn to_char(self) -> char {
        match self {
            LaneType::Driving => 'd',
            LaneType::Biking => 'b',
            LaneType::Bus => 'B',
            LaneType::Parking => 'p',
            LaneType::Sidewalk => 's',
            LaneType::Shoulder => 'S',
            LaneType::SharedLeftTurn => 'C',
            LaneType::Construction => 'x',
            LaneType::LightRail => 'l',
            LaneType::Buffer(_) => '|',
            LaneType::Footway => 'f',
            LaneType::SharedUse => 'F',
        }
    }

    /// The inverse of `to_char`. Always picks one buffer type. Panics on invalid input.
    pub fn from_char(x: char) -> LaneType {
        match x {
            'd' => LaneType::Driving,
            'b' => LaneType::Biking,
            'B' => LaneType::Bus,
            'p' => LaneType::Parking,
            's' => LaneType::Sidewalk,
            'S' => LaneType::Shoulder,
            'C' => LaneType::SharedLeftTurn,
            'x' => LaneType::Construction,
            'l' => LaneType::LightRail,
            '|' => LaneType::Buffer(BufferType::FlexPosts),
            'f' => LaneType::Footway,
            'F' => LaneType::SharedUse,
            _ => panic!("from_char({}) undefined", x),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LaneSpec {
    pub lt: LaneType,
    pub dir: Direction,
    pub width: Distance,
    /// Turn restrictions for this lane. An empty set represents that no restrictions are indicated
    /// (though local rules might still dictate restrictions).
    /// Turns for specific vehicle types (`turn:bus:lanes` and such) are not yet captured.
    pub allowed_turns: EnumSet<TurnDirection>,

    pub lane: Option<Lane>,
}

impl LaneSpec {
    /// For a given lane type, returns some likely widths. This may depend on the OSM highway type
    /// of the road. The first value returned will be used as a default.
    pub fn typical_lane_widths(lt: LaneType, highway_type: &str) -> Vec<(Distance, &'static str)> {
        // These're cobbled together from various sources
        match lt {
            // https://en.wikipedia.org/wiki/Lane#Lane_width
            LaneType::Driving => {
                let mut choices = vec![
                    (NORMAL_LANE_THICKNESS, "typical"),
                    (SERVICE_ROAD_LANE_THICKNESS, "alley"),
                    (Distance::feet(8.0), "narrow"),
                    (Distance::feet(12.0), "highway"),
                ];
                if highway_type == "service" {
                    choices.swap(1, 0);
                }
                choices
            }
            // https://www.gov.uk/government/publications/cycle-infrastructure-design-ltn-120 table
            // 5-2
            LaneType::Biking => vec![
                (Distance::meters(1.5), "absolute minimum"),
                (Distance::meters(2.0), "standard"),
            ],
            // https://nacto.org/publication/urban-street-design-guide/street-design-elements/transit-streets/dedicated-curbside-offset-bus-lanes/
            LaneType::Bus => vec![
                (Distance::feet(10.0), "minimum"),
                (Distance::feet(12.0), "normal"),
            ],
            // https://nacto.org/publication/urban-street-design-guide/street-design-elements/lane-width/
            LaneType::Parking => {
                let mut choices = vec![
                    (NORMAL_LANE_THICKNESS, "full lane"),
                    (SERVICE_ROAD_LANE_THICKNESS, "alley"),
                    (Distance::feet(7.0), "narrow"),
                    (Distance::feet(15.0), "loading zone"),
                ];
                if highway_type == "service" {
                    choices.swap(1, 0);
                }
                choices
            }
            // Just a guess
            LaneType::SharedLeftTurn => vec![(NORMAL_LANE_THICKNESS, "default")],
            // These're often converted from existing lanes, so just retain that width
            LaneType::Construction => vec![(NORMAL_LANE_THICKNESS, "default")],
            // No idea, just using this for now...
            LaneType::LightRail => vec![(NORMAL_LANE_THICKNESS, "default")],
            // http://www.seattle.gov/rowmanual/manual/4_11.asp
            LaneType::Sidewalk => vec![
                (SIDEWALK_THICKNESS, "default"),
                (Distance::feet(6.0), "wide"),
            ],
            LaneType::Shoulder => vec![(SHOULDER_THICKNESS, "default")],
            // Pretty wild guesses
            LaneType::Buffer(BufferType::Stripes) => vec![(Distance::meters(1.5), "default")],
            LaneType::Buffer(BufferType::FlexPosts) => {
                vec![(Distance::meters(0.5), "default")]
            }
            LaneType::Buffer(BufferType::Planters) => {
                vec![(Distance::meters(2.0), "default")]
            }
            LaneType::Buffer(BufferType::JerseyBarrier) => {
                vec![(Distance::meters(1.5), "default")]
            }
            LaneType::Buffer(BufferType::Curb) => vec![(Distance::meters(0.1), "default")],
            LaneType::Buffer(BufferType::Verge) => vec![(Distance::meters(2.0), "default")],
            LaneType::Footway => vec![(Distance::meters(2.0), "default")],
            LaneType::SharedUse => vec![(Distance::meters(3.0), "default")],
        }
    }

    /// Pick a reasonable default for a lane width, without any context on locale or tags.
    pub fn typical_lane_width(lt: LaneType) -> Distance {
        Self::typical_lane_widths(lt, "road")[0].0
    }

    /// None if bidirectional. If it's one-way, which direction is that relative to the road?
    /// (Usually forwards)
    pub fn oneway_for_driving(lanes: &[LaneSpec]) -> Option<Direction> {
        let mut fwd = false;
        let mut back = false;
        for x in lanes {
            if x.lt == LaneType::Driving {
                if x.dir == Direction::Forward {
                    fwd = true;
                } else {
                    back = true;
                }
            }
        }
        if fwd && back {
            // Bidirectional
            None
        } else if fwd {
            Some(Direction::Forward)
        } else if back {
            Some(Direction::Backward)
        } else {
            // Not driveable at all
            None
        }
    }
}

/// A broad categorisation of traffic by the kind of infrastructure it requires.
///
/// Look elsewhere for the "mode" of traffic, distinguishing busses, taxis, etc.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum TrafficClass {
    /// Pedestrians, wheelchair users, etc.
    Pedestrian,
    /// Bicycles or similar small ridden vehicles.
    Bicycle,
    /// Licenced motor vehicles, including motorbikes, cars, busses and trucks.
    Motor,
    /// Trains and trams that run on rails.
    Rail,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Direction {
    Forward,
    Backward,
}

impl Direction {
    pub fn opposite(self) -> Direction {
        match self {
            Direction::Forward => Direction::Backward,
            Direction::Backward => Direction::Forward,
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Direction::Forward => write!(f, "forwards"),
            Direction::Backward => write!(f, "backwards"),
        }
    }
}

/// A turn direction as defined by <https://wiki.openstreetmap.org/wiki/Key:turn>.
#[derive(Debug, EnumSetType)]
pub enum TurnDirection {
    Through,
    Left,
    Right,
    /// A turn to the left of less than 90 degrees. Not to be confused with a merge or a highway exit.
    SlightLeft,
    /// A turn to the right of less than 90 degrees. Not to be confused with a merge or a highway exit.
    SlightRight,
    SharpLeft,
    SharpRight,
    /// A merge one lane to the left, or a highway exit on the left.
    MergeLeft,
    /// A merge one lane to the right, or a highway exit on the right.
    MergeRight,
    /// A full 180 degree turn, aka a U-turn.
    Reverse,
}

/// Refers to a lane by its left-to-right position among all lanes in that direction. Backwardward
/// lanes are counted left-to-right from the backwards direction.
///
/// e.g. The left-most forward lane is `LtrLaneNum::Forward(1)` and the backward lane furthest to
/// the road-right is `LtrLaneNum::Backwardward(1)`, because of the backward perspective.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum LtrLaneNum {
    Forward(usize),
    Backwardward(usize),
}

impl LtrLaneNum {
    pub fn direction(&self) -> Direction {
        match self {
            Self::Forward(_) => Direction::Forward,
            Self::Backwardward(_) => Direction::Backward,
        }
    }

    pub fn number(&self) -> usize {
        match self {
            Self::Forward(num) | Self::Backwardward(num) => *num,
        }
    }

    /// Converts to the same numbered lane in the opposite direction.
    pub fn reverse(&self) -> Self {
        use LtrLaneNum::*;
        match self {
            Forward(n) => Backwardward(*n),
            Backwardward(n) => Forward(*n),
        }
    }
}

/// Identifies a position within the width of a roadway. Lanes are identified by their left-to-right
/// position, as per the OSM convention.
///
/// Most commonly seen as a value of the placement tag, e.g.
/// `placement=right_of:1` means that the OSM way is drawn along the right edge of lane 1.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum RoadPosition {
    /// The center of the carriageway width, ignoring lanes. The default placement of OSM ways.
    Center,
    /// The center of the full width of a `Road`, including verges and footpaths.
    FullWidthCenter,
    /// The center of the separation between both directions of traffic, i.e. the dividing line,
    /// median, or shared turning lane. For a oneway road, this is the "inside" edge of the road,
    /// i.e. the right side of LHT and the left side of RHT.
    Separation,
    /// On the left edge of the named lane (from the direction of the named lane).
    LeftOf(LtrLaneNum),
    /// In the middle of the named lane.
    MiddleOf(LtrLaneNum),
    /// On the right edge of the named lane (from the direction of the named lane).
    RightOf(LtrLaneNum),
}

impl RoadPosition {
    /// Converts to the same placement interpreted from the other direction. That is, only the
    /// wrapped LtrLaneNum is reversed.
    pub fn reverse(self) -> Self {
        use RoadPosition::*;
        match self {
            Center | FullWidthCenter | Separation => self,
            LeftOf(n) => LeftOf(n.reverse()),
            MiddleOf(n) => MiddleOf(n.reverse()),
            RightOf(n) => RightOf(n.reverse()),
        }
    }
}

/// Describes the placement of a line (such as the OSM Way) along a road.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Placement {
    /// Along the specified position down the entire length.
    Consistent(RoadPosition),
    /// Varying linearly from a specified position at the start, to a different one at the end.
    Varying(RoadPosition, RoadPosition),
    /// Varying linearly from some unspecified position at the start, to a different one at the end.
    Transition,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum DrivingSide {
    Right,
    Left,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MapConfig {
    /// If true, driving happens on the right side of the road (USA). If false, on the left
    /// (Australia).
    ///
    /// Note this is calculated by osm2streets! The value passed in is ignored; don't do any work
    /// to set it.
    pub driving_side: DrivingSide,
    /// The [two-letter ISO country code](https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2) where
    /// this network exists. Note osm2streets doesn't support areas that cross country boundaries.
    ///
    /// Note this is calculated by osm2streets! The value passed in is ignored; don't do any work
    /// to set it.
    pub country_code: String,
    pub bikes_can_use_bus_lanes: bool,
    /// If true, roads without explicitly tagged sidewalks may be assigned sidewalks or shoulders.
    /// If false, no inference will occur and separate sidewalks and crossings will be included.
    pub inferred_sidewalks: bool,
    /// Street parking is divided into spots of this length. 8 meters is a reasonable default, but
    /// people in some regions might be more accustomed to squeezing into smaller spaces. This
    /// value can be smaller than the hardcoded maximum car length; cars may render on top of each
    /// other, but otherwise the simulation doesn't care.
    pub street_parking_spot_length: Distance,
    /// If true, turns on red which do not conflict crossing traffic ('right on red') are allowed
    pub turn_on_red: bool,
    /// OSM railway=rail will be included as light rail if so. Cosmetic only.
    pub include_railroads: bool,
    pub inferred_kerbs: bool,
}

impl MapConfig {
    pub fn default() -> Self {
        Self {
            // Just a dummy value that'll be set later
            driving_side: DrivingSide::Right,
            country_code: String::new(),
            bikes_can_use_bus_lanes: true,
            inferred_sidewalks: false,
            street_parking_spot_length: Distance::meters(8.0),
            turn_on_red: true,
            include_railroads: true,
            inferred_kerbs: true,
        }
    }
}
