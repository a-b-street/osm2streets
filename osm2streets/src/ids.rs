use std::fmt;

use serde::{Deserialize, Serialize};

use crate::osm::{NodeID, WayID};
use crate::Road;

/// Refers to a road segment between two nodes, using OSM IDs. Note OSM IDs are not stable over
/// time.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OriginalRoad {
    pub osm_way_id: WayID,
    pub i1: NodeID,
    pub i2: NodeID,
}

impl fmt::Display for OriginalRoad {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "OriginalRoad({} from {} to {}",
            self.osm_way_id, self.i1, self.i2
        )
    }
}
impl fmt::Debug for OriginalRoad {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl OriginalRoad {
    pub fn new(way: i64, (i1, i2): (i64, i64)) -> OriginalRoad {
        OriginalRoad {
            osm_way_id: WayID(way),
            i1: NodeID(i1),
            i2: NodeID(i2),
        }
    }

    pub fn has_common_endpoint(&self, other: OriginalRoad) -> bool {
        if self.i1 == other.i1 || self.i1 == other.i2 {
            return true;
        }
        if self.i2 == other.i1 || self.i2 == other.i2 {
            return true;
        }
        false
    }

    // TODO Doesn't handle two roads between the same pair of intersections
    pub fn common_endpt(&self, other: OriginalRoad) -> NodeID {
        if self.i1 == other.i1 || self.i1 == other.i2 {
            return self.i1;
        }
        if self.i2 == other.i1 || self.i2 == other.i2 {
            return self.i2;
        }
        panic!("{:?} and {:?} have no common_endpt", self, other);
    }
}

/// It's sometimes useful to track both a road's ID and endpoints together. Use this sparingly.
#[derive(Clone)]
pub struct RoadWithEndpoints {
    pub road: OriginalRoad,
    pub src_i: NodeID,
    pub dst_i: NodeID,
}

impl RoadWithEndpoints {
    pub fn new(road: &Road) -> Self {
        Self {
            road: road.id,
            src_i: road.src_i,
            dst_i: road.dst_i,
        }
    }

    pub fn other_side(&self, i: NodeID) -> NodeID {
        if self.src_i == i {
            self.dst_i
        } else if self.dst_i == i {
            self.src_i
        } else {
            panic!("{} doesn't have {} on either side", self.road, i);
        }
    }
}
