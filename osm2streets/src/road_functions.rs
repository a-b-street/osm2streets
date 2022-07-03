#![allow(unused)]

use std::fmt::{Display, Formatter};

use crate::road_parts::{Designation, Lane, RoadEdge};

struct CrossWay(Lane);

impl CrossWay {
    pub fn unmarked() -> CrossWay {
        CrossWay(Lane {
            width: 2.0,
            ..Lane::foot()
        })
    }
}

/// A collection of Lanes, travelling in the same direction. A "half street" if you will.
#[derive(Clone, Debug, PartialEq)]
pub struct RoadWay {
    /// Lanes, inside (fast lane) out (slow lanes, footpaths). Directions is almost always forward.
    pub elements: Vec<Lane>,
    // /// The transverse lines
    // seperators: Vec<Separator>,
    /// How this roadway transitions into the adjacent area on the inside.
    pub inner: RoadEdge,
    /// How this roadway transitions into the adjacent area on the outside.
    pub outer: RoadEdge,
}

impl RoadWay {
    pub fn lanes(&self) -> Vec<&Lane> {
        self.elements
            .iter()
            .filter_map(|el| {
                if let Designation::NoTravel = &el.designation {
                    None
                } else {
                    Some(el)
                }
            })
            .collect()
    }

    // Fluid style setters for builder-like uses.
    fn set_overtaking(mut self, overtaking: bool) -> Self {
        if let Some(l) = self.elements.get_mut(0) {
            l.can_enter_from_inside = false;
        }
        self
    }
}

/// Defaults that will probably come from osm2lanes in the full picture.
impl RoadWay {
    // pub fn new(elements: Iterator<Into<E>>, Option<(RoadEdge, RoadEdge>) -> Self {}

    pub fn track() -> Self {
        Self {
            inner: RoadEdge::Sudden,
            elements: vec![Lane::track()],
            outer: RoadEdge::Sudden,
        }
    }
    pub fn rural() -> Self {
        Self {
            inner: RoadEdge::Join,
            elements: vec![Lane::car()],
            outer: RoadEdge::Sudden,
        }
    }
    pub fn local() -> Self {
        Self {
            // edges: enum_map!{ Inner => RoadEdge::Join, Outer => RoadEdge::Barrier}, // enum-map
            inner: RoadEdge::Join,
            elements: vec![Lane::car(), Lane::verge(), Lane::foot()],
            outer: RoadEdge::Barrier,
        }
    }
    pub fn arterial() -> Self {
        Self {
            inner: RoadEdge::Join,
            elements: vec![Lane::car(), Lane::verge(), Lane::foot()],
            outer: RoadEdge::Barrier,
        }
        .set_overtaking(false)
    }
}

/// Intersections are the joints of the RoadNetwork, they join two or more RoadWays. Represents
/// - the cross-sectional slice of a road that is changing properties or split, or
/// - the negotiated area where multiple lanes of travel overlap (where road markings would cease).
#[derive(Clone, Debug, PartialEq)]
pub enum IntersectionType {
    /// An intersection that is missing some connected roads or data (e.g. at the edge of the map).
    Unknown,
    /// The edge of the data that we have.
    MapEdge,
    /// Turning circles, road end signs, train terminus thingos, edge of the map?
    Terminus,
    /// A slice where conditions change, but no yielding.
    Slice,
    /// A perpendicular line across a road, where conditions change.
    // Pedestrian crossings have actual width, but if conditions change instantaneously,
    // it's the same shape as a crossing, but with width=0.
    // There might be a yield: give way (zebra), traffic light (or other control) stop line.
    Crossing,
    /// One or more "minor" roads merge into (yielding) or out of a "major" road.
    // The edge line would be interupted, replaced with some merge line, like:
    // a dotted merge line, a dashed give way line, a solid stop line.
    // I wonder what more than two major incoming roads look like? Dangerous? Missing yield signs, most likely?
    Merge,
    /// An area of the road where priority is shared over time, by lights, negotiation and priority, etc.
    // You would expect normal lane markings to be missing, sometimes with some helpful markings added
    // (lane dividers for multi-lane turns, etc.)
    RoadIntersection,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ControlType {
    Lights,
    Signed,
    Uncontrolled,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Intersection {
    pub t: IntersectionType,
    pub control: ControlType,
}

impl Default for Intersection {
    fn default() -> Self {
        Self {
            t: IntersectionType::Unknown,
            control: ControlType::Uncontrolled,
        }
    }
}

impl Intersection {
    pub fn slice() -> Self {
        Self {
            t: IntersectionType::Slice,
            control: ControlType::Uncontrolled,
        }
    }
    pub fn merge() -> Self {
        Self {
            t: IntersectionType::Merge,
            control: ControlType::Uncontrolled,
        }
    }
    pub fn intersection() -> Self {
        Self {
            t: IntersectionType::RoadIntersection,
            control: ControlType::Uncontrolled,
        }
    }
    pub fn turning_circle() -> Self {
        Self {
            t: IntersectionType::Terminus,
            control: ControlType::Uncontrolled,
        }
    }
    // fn area(&self) -> Polygon { unimplemented!() }
}

impl Display for RoadWay {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} lanes", self.lanes().len())
    }
}

impl Display for Intersection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let IntersectionType::RoadIntersection = &self.t {
            write!(f, "{:?} ", self.control)?
        }
        write!(f, "{:?}", self.t)
    }
}
