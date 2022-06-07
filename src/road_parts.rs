#![allow(dead_code)]

use crate::units::preamble::*;
use Carriage::*;
use Designation::*;

use crate::road_parts::RoadEdge::Kerb;
use crate::units::{Meters, RoadSide, TrafficDirections};

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
// TODO, defer to raw_map::osm::RoadRank and others.
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
    /// Think "carriageway" carriages, anything from trucks to mopeds to drawn carts.
    Travel(Carriage),
    /// Areas of the road that are explicitly not for (normal) driving.
    /// Often painted, sometimes w/barriers.
    NoTravel,
    /// Loading zones for trucks too, with short stay?
    Parking(Carriage),
    /// Verges without parking, those outdoor eating areas, for example.
    Amenity,
}

pub trait CrossSection {
    /// Can this buffer/line be crossed (inward, outward) (for changing lanes, overtaking, etc)
    fn can_enter_from(&self, dir: RoadSide) -> bool;

    /// How wide the created "buffer" area is.
    ///
    /// Lines have width=0, because they lay on the lane surface, instead of creating their own
    /// "buffer" area that you can occupy.
    fn width(&self) -> Meters;
}

/// A box for elements that make up the roadway. Should I be using a trait? I don't know.
//TODO merge Lane and Buffer to remove this indirection.
#[derive(Clone, Debug, PartialEq)]
pub enum E {
    Buffer(Buffer),
    Lane(Lane),
}

/// What is the nature of the edge of this area of road?
#[derive(Clone, Debug, PartialEq)]
pub enum RoadEdge {
    /// Not actually the edge of the road, but a continuation into more road surface.
    /// Expect Join to a Buffer for railings/bollards.
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
pub struct Lane {
    pub dir: TrafficDirections,
    pub designation: Designation,
    pub width: Meters,
    pub can_enter_from_inside: bool,
    pub can_enter_from_outside: bool,
}

/// These thingos
impl Lane {
    pub fn track() -> Self {
        Self {
            designation: Travel(Cars),
            width: 4.0,
            ..Lane::foot()
        }
    }
    pub fn foot() -> Self {
        Self {
            dir: TrafficDirections::BothWays,
            designation: Travel(Foot),
            width: 1.5,
            can_enter_from_inside: true,
            can_enter_from_outside: true,
        }
    }
    pub fn bike() -> Self {
        Self {
            dir: TrafficDirections::Forward,
            designation: Travel(Bike),
            width: 1.0,
            can_enter_from_inside: true,
            can_enter_from_outside: true,
        }
    }
    pub fn service() -> Self {
        Self {
            designation: Travel(Cars),
            width: 4.0,
            ..Lane::foot() // negotiated lane like foot traffic
        }
    }
    pub fn car() -> Self {
        Self {
            dir: TrafficDirections::Forward,
            designation: Travel(Cars),
            width: 3.5,
            can_enter_from_inside: true, // start by assume overtaking is allowed.
            can_enter_from_outside: true, // Not usually any reason to disallow entry from the outside.
        }
    }
    pub fn bus() -> Self {
        Self {
            designation: Travel(Bus),
            ..Self::car()
        }
    }
    pub fn truck() -> Self {
        Self {
            designation: Travel(Truck),
            ..Self::car()
        }
    }
}

/// All interruptions to the Carriageway, medians, painted buffers, verges, ... with width etc.
/// From painted areas, to curbs with all sorts of features inside.
#[derive(Clone, Debug, PartialEq)]
pub struct Buffer {
    /// How the road joins the buffer. Expect `RoadEdge::Joined` for painted buffers.
    edge: RoadEdge,
    width: Meters,
    // features: Vec<Feature>,
    // paint_style: Pattern,
}

impl Buffer {
    pub fn verge() -> Self {
        Self {
            edge: Kerb,
            width: 3.0,
        }
    }
}

pub struct BorderLine {
    can_enter_from_inside: bool,
    can_enter_from_outside: bool,
    // style: LinePattern,
}

impl CrossSection for Lane {
    fn can_enter_from(&self, dir: RoadSide) -> bool {
        match dir {
            Inside => self.can_enter_from_inside,
            Outside => self.can_enter_from_outside,
        }
    }

    fn width(&self) -> Meters {
        self.width
    }
}

impl CrossSection for Buffer {
    fn can_enter_from(&self, _dir: RoadSide) -> bool {
        false
    }
    fn width(&self) -> Meters {
        self.width
    }
}

impl CrossSection for BorderLine {
    fn can_enter_from(&self, dir: RoadSide) -> bool {
        match dir {
            Inside => self.can_enter_from_inside,
            Outside => self.can_enter_from_outside,
        }
    }

    fn width(&self) -> Meters {
        0.0
    }
}
