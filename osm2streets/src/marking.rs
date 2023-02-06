use crate::LaneType;

// We use geom and stay in map space. Output is done in latlon.
use geom::{Angle, Line, PolyLine, Polygon, Pt2D};

pub enum Marking {
    /// Lines along a lane.
    Longitudinal(PolyLine, Longitudinal),
    /// Lines across a lane.
    Transverse(Line, Transverse),
    /// Iconic or textual symbols displayed at some angle.
    Symbol(Pt2D, Angle, Symbol),
    /// Designated areas, like buffers, painted medians, keep clear, etc.
    Area(Polygon, Area),
}

pub struct Longitudinal {
    pub kind: LaneEdgeKind,
    /// The two lanes, ltr.
    pub lanes: (LaneType, LaneType),
}

pub enum LaneEdgeKind {
    OncomingSeparation {
        overtake_left: bool,
        overtake_right: bool,
    },
    LaneSeparation {
        merge_left: bool,
        merge_right: bool,
    },
    RoadEdge,
    /// Longitudinal marking that is interrupted by other traffic.
    Continuity,
}

pub enum Transverse {
    StopLine,
    YieldLine,
}

pub enum Symbol {
    TrafficMode(TrafficMode),
    TurnArrow(TurnDirections),
}

pub struct TurnDirections {
    through: bool,
    left: bool,
    right: bool,
    slight_left: bool,
    slight_right: bool,
    reverse: bool,
}

pub enum TrafficMode {
    Car,
    Bike,
    Pedestrian,
    Bus,
    Taxi,
}

pub enum Area {
    /// Generic no traffic areas.
    OutOfBounds,
    // KeepClear,
}
