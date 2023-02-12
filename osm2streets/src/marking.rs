//! Semantic definitions of the road markings used to control traffic.
//!
//! These types describe the *meaning* of the markings, in a locale-independent way, not necessarily
//! how they look. See [`crate::paint`] for generating locale-dependant renderings of these markings.
//!
//! This initial version of these types is based on, and takes its nomenclature from, the Australian
//! Manual of Uniform Traffic Control Devices (MUTCD). As such, the distinctions between different
//! kinds of markings that are represented as variants are tailored to support the marking scheme
//! used in Australia, not necessarily in other parts of the world. The intention is to expand these
//! definitions so that they can distinguish all the distinct situations that OSM represents around
//! the world that have distinct markings in one of the supported locales.

// We use geom and stay in map space. Output is done in latlon.
use geom::{Angle, Line, PolyLine, Polygon, Pt2D};

use crate::lanes::TrafficClass;
use crate::LaneType;

/// A marking painted on the road surface to direct traffic.
pub enum RoadMarking {
    /// Markings along a lane.
    Longitudinal(PolyLine, Longitudinal),
    /// Markings across one or more lanes.
    Transverse(Line, Transverse),
    /// Symbolic markings, iconic or textual, oriented at a given angle.
    Symbol(Pt2D, Angle, Symbol),
    /// Area markings.
    // TODO: Add an optional center line for orienting certain features, such as medians and "splayed approaches".
    Area(Polygon, Area),
}

pub struct Longitudinal {
    pub kind: LongitudinalLine,
    /// The two lanes, ltr. At least one will be a traffic lane, and the other might be a buffer.
    pub lanes: [LaneType; 2],
}

pub enum LongitudinalLine {
    /// A line separating opposing directions of traffic.
    Dividing {
        overtake_left: bool,
        overtake_right: bool,
    },
    /// A line separating lanes of traffic travelling in the same direction.
    Lane { merge_left: bool, merge_right: bool },
    /// A line at the edge of a lane that is also the edge of the road.
    Edge,
    /// A line at the edge of a lane that is intended to be crossed by other traffic crossing,
    /// entering or exiting the lane.
    Continuity,
    /// A line guiding traffic turning through an intersection.
    Turn,
}

#[derive(Clone, Copy)]
pub enum Transverse {
    StopLine,
    YieldLine,
}

pub enum Symbol {
    /// A marking indicating a mode of traffic that is allowed.
    TrafficMode(TrafficClass),
    /// A marking indicating which turns may be performed.
    TurnArrow(TurnDirections),
}

/// A set of turn directions that are allowed.
// TODO: move to lanes/mod.rs
pub struct TurnDirections {
    through: bool,
    left: bool,
    right: bool,
    slight_left: bool,
    slight_right: bool,
    reverse: bool,
}
impl TurnDirections {
    pub fn through() -> Self {
        TurnDirections {
            through: true,
            left: false,
            right: false,
            slight_left: false,
            slight_right: false,
            reverse: false,
        }
    }
}

pub enum Area {
    /// Generic no traffic areas.
    OutOfBounds,
    // KeepClear,
}

impl RoadMarking {
    pub fn longitudinal(geometry: PolyLine, kind: LongitudinalLine, lanes: [LaneType; 2]) -> Self {
        RoadMarking::Longitudinal(geometry, Longitudinal { kind, lanes })
    }

    pub fn transverse(geometry: Line, kind: Transverse) -> Self {
        RoadMarking::Transverse(geometry, kind)
    }

    pub fn turn_arrow(geometry: Pt2D, angle: Angle, turns: TurnDirections) -> Self {
        RoadMarking::Symbol(geometry, angle, Symbol::TurnArrow(turns))
    }

    pub fn area(geometry: Polygon) -> Self {
        RoadMarking::Area(geometry, Area::OutOfBounds)
    }
}

impl LongitudinalLine {
    pub fn dividing(overtake_left: bool, overtake_right: bool) -> Self {
        LongitudinalLine::Dividing {
            overtake_left,
            overtake_right,
        }
    }
    pub fn lane(merge_left: bool, merge_right: bool) -> Self {
        LongitudinalLine::Lane {
            merge_left,
            merge_right,
        }
    }
    pub fn edge() -> Self {
        LongitudinalLine::Edge
    }
    pub fn continuity() -> Self {
        LongitudinalLine::Continuity
    }
}
