mod network;

#[allow(dead_code, unused_imports)]

pub type Meters = f32;

#[derive(Clone, Debug, PartialEq)]
pub enum End {
    Front,
    Back,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Direction {
    Forward,
    Backward,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Directions {
    Forward,
    Backward,
    BothWays,
    Alternating,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Side {
    Left,
    Right,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SideDirection {
    Leftward,
    Rightward,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LaneSide {
    Inside,
    Outside,
}

pub enum LaneDirection {
    Inward,
    Outward,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DrivingSide {
    LHT,
    RHT,
}

use Direction::*;
use DrivingSide::*;
use End::*;
use LaneDirection::*;
use LaneDirection::*;
use LaneSide::*;
use Side::*;
use SideDirection::*;

impl DrivingSide {
    fn get_direction(&self, side: Side) -> Direction {
        match (self, side) {
            (Self::LHT, Side::Left) | (Self::RHT, Side::Right) => Direction::Forward,
            (Self::LHT, Side::Right) | (Self::RHT, Side::Left) => Direction::Backward,
        }
    }
    fn get_side(&self, dir: Direction) -> Side {
        match (self, dir) {
            (Self::LHT, Direction::Forward) | (Self::RHT, Direction::Backward) => Side::Left,
            (Self::LHT, Direction::Backward) | (Self::RHT, Direction::Forward) => Side::Right,
        }
    }
}

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
    dir: Directions,
    designation: Designation,
    width: Meters,
    can_enter_from_inside: bool,
    can_enter_from_outside: bool,
}

struct CrossWay(Lane);

impl CrossWay {
    fn unmarked() -> CrossWay {
        CrossWay(Lane {
            width: 2.0,
            ..Lane::foot()
        })
    }
}

impl Lane {
    fn foot() -> Self {
        Self {
            dir: Directions::BothWays,
            designation: Designation::Travel(Carriage::Foot),
            width: 1.5,
            can_enter_from_inside: true,
            can_enter_from_outside: true,
        }
    }
    fn bike() -> Self {
        Self {
            dir: Directions::Forward,
            designation: Designation::Travel(Carriage::Bike),
            width: 1.0,
            can_enter_from_inside: true,
            can_enter_from_outside: true,
        }
    }
    fn car() -> Self {
        Self {
            dir: Directions::Forward,
            designation: Designation::Travel(Carriage::Car),
            width: 3.5,
            can_enter_from_inside: true, // start by assume overtaking is allowed.
            can_enter_from_outside: true, // Not usually any reason to disallow entry from the outside.
        }
    }
    fn bus() -> Self {
        Self {
            designation: Designation::Travel(Carriage::Bus),
            ..Self::car()
        }
    }
    fn truck() -> Self {
        Self {
            designation: Designation::Travel(Carriage::Truck),
            ..Self::car()
        }
    }
}

// trait AreaMarking {
//     fn hasBorder(&self) -> bool;
//     fn hasFill(&self) -> bool;
// }

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

pub struct BorderLine {
    can_enter_from_inside: bool,
    can_enter_from_outside: bool,
    // style: LinePattern,
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

#[derive(Clone, Debug, PartialEq)]
enum CorridorElement {
    Buffer(Buffer),
    Lane(Lane),
}

// impl CrossSection for CorridorElement { }

/// RoadWay Road : a collection of Lanes (and Buffers), travelling in the same direction; A "half street" if you will, or a slip road or slip lane.
#[derive(Clone, Debug, PartialEq)]
pub struct RoadWay {
    /// Lanes, inside (fast lane) out (slow lanes, footpaths). Directions is almost always forward.
    elements: Vec<CorridorElement>,
    // /// The transverse lines
    // seperators: Vec<Separator>,
    /// How this roadway transitions into the adjacent area on the inside.
    inner: RoadEdge,
    /// How this roadway transitions into the adjacent area on the outside.
    outer: RoadEdge,
}

impl RoadWay {
    fn lanes(&self) -> Vec<&Lane> {
        self.elements
            .iter()
            .filter_map(|el| {
                if let CorridorElement::Lane(l) = el {
                    Some(l)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl RoadWay {
    fn residential() -> Self {
        Self {
            inner: RoadEdge::Join,
            elements: vec![
                CorridorElement::Lane(Lane::car()),
                CorridorElement::Buffer(Buffer::verge()),
                CorridorElement::Lane(Lane::foot()),
            ],
            outer: RoadEdge::Barrier,
        }
    }
}

/// Intersections are the joints of the RoadNetwork, they join two or more RoadWays together.
/// Represents either the crosssection of a road that is changing properties,
/// or the area where multiple lanes of travel overlap (where the road markings would
#[derive(Clone, Debug, PartialEq)]
pub enum IntersectionType {
    /// Turning circles, road end signs, train terminus thingos, edge of the map?
    Terminus,
    /// A perpendicular line across a road, where conditions change.
    // Pedestrian crossings have actual width, but if conditions change instantaneously,
    // it's the same shape as a crossing, but with width=0.
    Crossing,
    /// A "major" road crosses one or more "minor" roads, which yield to it.
    // I wonder what more than two major incoming roads look like? Dangerous? Missing yield signs, most likely?
    MajorMinor,
    /// An area of the road where priority is shared over time, by lights, negotiation and priority, etc.
    // You would expect normal lane markings to be missing, sometimes with some helpful markings added
    // (lane dividers for multi-lane turns, etc.)
    Intersection,
}

#[derive(Clone, Debug, PartialEq)]
enum ControlType {
    Lights,
    Signed,
    Uncontrolled,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Intersection {
    t: IntersectionType,
    control: ControlType,
}

impl Default for Intersection {
    fn default() -> Self {
        Self {
            t: IntersectionType::Intersection,
            control: ControlType::Uncontrolled,
        }
    }
}

impl Intersection {
    // fn area(&self) -> Polygon { unimplemented!() }
}
