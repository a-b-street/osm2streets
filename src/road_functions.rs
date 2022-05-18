use crate::road_parts::{Buffer, CorridorElement, Lane, RoadEdge};

struct CrossWay(Lane);

impl CrossWay {
    pub fn unmarked() -> CrossWay {
        CrossWay(Lane {
            width: 2.0,
            ..Lane::foot()
        })
    }
}

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
    pub fn lanes(&self) -> Vec<&Lane> {
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
    pub fn residential() -> Self {
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
