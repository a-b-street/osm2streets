use std::fmt;

use abstutil::{deserialize_usize, serialize_usize};
use serde::{Deserialize, Serialize};

use crate::Road;

/// Opaque and non-contiguous
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RoadID(
    #[serde(
        serialize_with = "serialize_usize",
        deserialize_with = "deserialize_usize"
    )]
    pub usize,
);

impl fmt::Display for RoadID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Road #{}", self.0)
    }
}

/// Opaque and non-contiguous
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct IntersectionID(
    #[serde(
        serialize_with = "serialize_usize",
        deserialize_with = "deserialize_usize"
    )]
    pub usize,
);

impl fmt::Display for IntersectionID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Intersection #{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LaneID {
    pub road: RoadID,
    /// Index into `lane_specs_ltr`
    #[serde(
        serialize_with = "serialize_usize",
        deserialize_with = "deserialize_usize"
    )]
    pub index: usize,
}

impl fmt::Display for LaneID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Lane {} of {}", self.index, self.road)
    }
}

/// It's sometimes useful to track both a road's ID and endpoints together. Use this sparingly.
#[derive(Clone)]
pub struct RoadWithEndpoints {
    pub road: RoadID,
    pub src_i: IntersectionID,
    pub dst_i: IntersectionID,
}

impl RoadWithEndpoints {
    pub fn new(road: &Road) -> Self {
        Self {
            road: road.id,
            src_i: road.src_i,
            dst_i: road.dst_i,
        }
    }

    /// Note the special case of roads that're loops on a single intersection -- the `other_side`
    /// is the same as the input in that case.
    pub fn other_side(&self, i: IntersectionID) -> IntersectionID {
        if self.src_i == i {
            self.dst_i
        } else if self.dst_i == i {
            self.src_i
        } else {
            panic!("{} doesn't have {} on either side", self.road, i);
        }
    }
}

#[derive(PartialEq)]
pub enum CommonEndpoint {
    /// Two lanes or roads share one endpoint
    One(IntersectionID),
    /// Two lanes or roads share both endpoints, because they're both lanes belonging to the same
    /// road, or there are two different roads connecting the same pair of intersections
    Both,
    /// Two lanes or roads don't have any common endpoints
    None,
}

impl CommonEndpoint {
    pub fn new(
        obj1: (IntersectionID, IntersectionID),
        obj2: (IntersectionID, IntersectionID),
    ) -> Self {
        let src = obj1.0 == obj2.0 || obj1.0 == obj2.1;
        let dst = obj1.1 == obj2.0 || obj1.1 == obj2.1;
        if src && dst {
            return Self::Both;
        }
        if src {
            return Self::One(obj1.0);
        }
        if dst {
            return Self::One(obj1.1);
        }
        Self::None
    }
}
