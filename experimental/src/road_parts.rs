#![allow(dead_code)]

use crate::units::{Meters, TrafficDirections};

use Carriage::*;
use Designation::*;

/// Some hunk of something hurtling or dawdling down some lane, or being stored somewhere.
/// From train carriages, to hand drawn carts, to the sack of bones pulling it.
#[derive(Clone, Debug, PartialEq)]
pub enum Carriage {
    /// People on foot, aka "Pedestrians".
    Foot,
    /// People on bikes. (Scooters too?)
    Bike,
    /// Licenced vehicles in general, but *roads are for cars*, so lets call them cars.
    /// (Besides, that was already more times than I want to have to type or say "vehicles".)
    Cars,
    /// Heavy vehicles that qualify for those little pictures of trucks on road.
    Taxi,
    Bus,
    Truck,
    /// Things on tracks, like trams and light rail. Heavy rail too, I guess.
    Train,
}

#[derive(Clone, Debug, PartialEq)]
// TODO, defer to osm2streets::osm::RoadRank and others.
pub enum RoadRanks {
    Freeway,
    Highway,
    Local,
    Rural,
    Service,
}

/// A usage designation for an area, such as a lane.
#[derive(Clone, Debug, PartialEq)]
pub enum Designation {
    /// A part of the road designated for travel.
    Travel {
        carriage: Carriage,
        direction: TrafficDirections,
    },
    /// A part of the road designated for parking / "standing".
    Parking { carriage: Carriage },
    /// A part of the road that is explicitly not (normally) for carriages.
    /// E.g. a painted buffer, median or verge.
    NoTravel,
}

/// What is the nature of the edge of this area of road?
#[derive(Clone, Debug, PartialEq)]
pub enum RoadEdge {
    /// Not actually the edge of the road, but a continuation into more road surface.
    Join,
    /// The road just ends and transitions into another groundcover.
    Sudden,
    /// A short rise up from the road surface. Constructed, usually out of concrete in a certain shape.
    Kerb,
    /// A cliff or the edge of a bridge.
    Drop,
    /// Walls etc. that interrupt the road surface.
    Barrier,
}

/// A single lane on the carriageway, with designation, width, etc.
#[derive(Clone, Debug, PartialEq)]
pub struct RoadPart {
    pub designation: Designation,
    pub width: Meters,
    pub can_enter_from_inside: bool,
    pub can_enter_from_outside: bool,
}

impl RoadPart {
    pub fn path() -> Self {
        Self {
            designation: Travel {
                carriage: Foot,
                direction: TrafficDirections::BothWays,
            },
            width: 1.5,
            can_enter_from_inside: true,
            can_enter_from_outside: true,
        }
    }
    pub fn track() -> Self {
        Self {
            designation: Travel {
                carriage: Cars,
                direction: TrafficDirections::BothWays,
            },
            width: 4.0,
            can_enter_from_inside: true,
            can_enter_from_outside: true,
        }
    }
    pub fn bike_lane() -> Self {
        Self {
            designation: Travel {
                carriage: Bike,
                direction: TrafficDirections::BothWays,
            },
            width: 1.0,
            can_enter_from_inside: true,
            can_enter_from_outside: true,
        }
    }
    pub fn service_road() -> Self {
        Self {
            designation: Travel {
                carriage: Cars,
                direction: TrafficDirections::BothWays, // negotiated lane like foot traffic
            },
            width: 4.0,
            can_enter_from_inside: true,
            can_enter_from_outside: true,
        }
    }
    pub fn lane() -> Self {
        Self {
            designation: Travel {
                carriage: Cars,
                direction: TrafficDirections::Forward,
            },
            width: 3.5,
            can_enter_from_inside: true, // start by assume overtaking is allowed.
            can_enter_from_outside: true, // Not usually any reason to disallow entry from the outside.
        }
    }
    pub fn bus_lane() -> Self {
        Self {
            designation: Travel {
                carriage: Bus,
                direction: TrafficDirections::Forward,
            },
            ..Self::lane()
        }
    }
    pub fn truck_lane() -> Self {
        Self {
            designation: Travel {
                carriage: Truck,
                direction: TrafficDirections::Forward,
            },
            ..Self::lane()
        }
    }

    pub fn median() -> Self {
        Self {
            designation: NoTravel,
            width: 1.0,
            can_enter_from_inside: false,
            can_enter_from_outside: false,
        }
    }
    pub fn verge() -> Self {
        Self {
            designation: NoTravel,
            width: 3.0,
            can_enter_from_inside: false,
            can_enter_from_outside: false,
        }
    }
}
