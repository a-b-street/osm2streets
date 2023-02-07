// We use geom and stay in map space. Output is done in latlon.
use geom::{Angle, Line, PolyLine, Polygon, Pt2D};

use crate::lanes::TrafficMode;
use crate::marking::Area::OutOfBounds;
use crate::LaneType;

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
    pub lanes: [LaneType; 2],
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

// TODO move to lanes/mod.rs
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

impl Marking {
    pub fn longitudinal(geometry: PolyLine, kind: LaneEdgeKind, lanes: [LaneType; 2]) -> Self {
        Marking::Longitudinal(geometry, Longitudinal { kind, lanes })
    }

    pub fn stop_line(geometry: Line) -> Self {
        Marking::Transverse(geometry, Transverse::StopLine)
    }
    pub fn yield_line(geometry: Line) -> Self {
        Marking::Transverse(geometry, Transverse::YieldLine)
    }

    pub fn turn_arrow(geometry: Pt2D, angle: Angle, turns: TurnDirections) -> Self {
        Marking::Symbol(geometry, angle, Symbol::TurnArrow(turns))
    }

    pub fn area(geometry: Polygon) -> Self {
        Marking::Area(geometry, OutOfBounds)
    }
}

impl LaneEdgeKind {
    pub fn oncoming(overtake_left: bool, overtake_right: bool) -> Self {
        LaneEdgeKind::OncomingSeparation {
            overtake_left,
            overtake_right,
        }
    }
    pub fn separation(merge_left: bool, merge_right: bool) -> Self {
        LaneEdgeKind::LaneSeparation {
            merge_left,
            merge_right,
        }
    }
    pub fn edge() -> Self {
        LaneEdgeKind::RoadEdge
    }
    pub fn continuity() -> Self {
        LaneEdgeKind::Continuity
    }
}
