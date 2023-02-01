pub enum Marking {
    Longitudinal(geom::PolyLine, LongitudinalMarking),
    Transverse(geom::Line, TransverseMarking),
    Symbol(geom::Pt2D, geom::Angle,  SymbolMarking),
    // Area(AreaMarking),
}

pub enum LongitudinalMarking {
    Separation {
        overtake_left: bool,
        overtake_right: bool,
    },
    LaneDivider {
        overtake: bool,
    },
    RoadEdge,
    /// Longitudinal marking that is interrupted by an intersection or merging traffic.
    Continuity,
}

pub enum TransverseMarking {
    StopLine,
    YieldLine,
}

pub enum SymbolMarking {
    TrafficMode(TrafficMode),
    TurnArrow(TurnDirections),
}

pub struct TurnDirections {
    through: bool,
    left: bool,
    right: bool,
    slight_left: bool,
    slight_right: bool,
    u_turn: bool,
}

pub enum TrafficMode {
    Car,
    Bike,
    Pedestrian,
    Bus,
    Taxi,
}
