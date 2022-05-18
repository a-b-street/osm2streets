use crate::road_parts::RoadEdge::Kerb;
use crate::units::LaneSide::{Inside, Outside};
use crate::units::{Directions, LaneSide, Meters};

#[derive(Clone, Debug, PartialEq)]
pub enum Carriage {
    Truck,
    Bus,
    Car,
    Bike,
    Foot,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Designation {
    /// Think "carriageway" carriages, anything from trucks to mopeds to drawn carts.
    Travel(Carriage),
    /// Loading zones for trucks too, with short stay?
    Parking(Carriage),
    /// Verge and stuff, those outdoor eating areas
    Amenity,
}

pub trait CrossSection {
    /// Can this buffer/line be crossed (inward, outward) (for changing lanes, overtaking, etc)
    fn can_enter_from(&self, dir: LaneSide) -> bool;

    /// How wide the created "buffer" area is.
    ///
    /// Lines have width=0, because they lay on the lane surface, instead of creating their own
    /// "buffer" area.
    fn width(&self) -> Meters;
}

#[derive(Clone, Debug, PartialEq)]
pub enum CorridorElement {
    Buffer(Buffer),
    Lane(Lane),
}

#[derive(Clone, Debug, PartialEq)]
pub enum RoadEdge {
    /// Not an edge at all, but a continuation into more road surface.
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
    pub dir: Directions,
    pub designation: Designation,
    pub width: Meters,
    pub can_enter_from_inside: bool,
    pub can_enter_from_outside: bool,
}

impl Lane {
    pub fn foot() -> Self {
        Self {
            dir: Directions::BothWays,
            designation: Designation::Travel(Carriage::Foot),
            width: 1.5,
            can_enter_from_inside: true,
            can_enter_from_outside: true,
        }
    }
    pub fn bike() -> Self {
        Self {
            dir: Directions::Forward,
            designation: Designation::Travel(Carriage::Bike),
            width: 1.0,
            can_enter_from_inside: true,
            can_enter_from_outside: true,
        }
    }
    pub fn car() -> Self {
        Self {
            dir: Directions::Forward,
            designation: Designation::Travel(Carriage::Car),
            width: 3.5,
            can_enter_from_inside: true, // start by assume overtaking is allowed.
            can_enter_from_outside: true, // Not usually any reason to disallow entry from the outside.
        }
    }
    pub fn bus() -> Self {
        Self {
            designation: Designation::Travel(Carriage::Bus),
            ..Self::car()
        }
    }
    pub fn truck() -> Self {
        Self {
            designation: Designation::Travel(Carriage::Truck),
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
    fn can_enter_from(&self, dir: LaneSide) -> bool {
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
    fn can_enter_from(&self, _dir: LaneSide) -> bool {
        false
    }
    fn width(&self) -> Meters {
        self.width
    }
}

impl CrossSection for BorderLine {
    fn can_enter_from(&self, dir: LaneSide) -> bool {
        match dir {
            Inside => self.can_enter_from_inside,
            Outside => self.can_enter_from_outside,
        }
    }

    fn width(&self) -> Meters {
        0.0
    }
}
