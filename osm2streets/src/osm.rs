//! Useful utilities for working with OpenStreetMap.

use std::fmt;

use serde::{Deserialize, Serialize};

// This is a commonly used key in the codebase, so worthy of a bit of typo-prevention
pub const HIGHWAY: &str = "highway";

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum RoadRank {
    Local,
    Arterial,
    Highway,
}

impl RoadRank {
    pub fn from_highway(hwy: &str) -> RoadRank {
        match hwy {
            "motorway" | "motorway_link" => RoadRank::Highway,
            "trunk" | "trunk_link" => RoadRank::Highway,
            "primary" | "primary_link" => RoadRank::Arterial,
            "secondary" | "secondary_link" => RoadRank::Arterial,
            "tertiary" | "tertiary_link" => RoadRank::Arterial,
            _ => RoadRank::Local,
        }
    }

    /// Larger number means a bigger road, according to https://wiki.openstreetmap.org/wiki/Key:highway
    pub fn detailed_from_highway(hwy: &str) -> usize {
        for (idx, x) in vec![
            "motorway",
            "motorway_link",
            "trunk",
            "trunk_link",
            "primary",
            "primary_link",
            "secondary",
            "secondary_link",
            "tertiary",
            "tertiary_link",
            "unclassified",
            "residential",
            "cycleway",
            "track",
        ]
        .into_iter()
        .enumerate()
        {
            if hwy == x {
                return 100 - idx;
            }
        }
        // Everything else gets lowest priority
        0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NodeID(pub i64);
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WayID(pub i64);
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RelationID(pub i64);

impl fmt::Display for NodeID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "https://www.openstreetmap.org/node/{}", self.0)
    }
}
impl fmt::Display for WayID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "https://www.openstreetmap.org/way/{}", self.0)
    }
}
impl fmt::Display for RelationID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "https://www.openstreetmap.org/relation/{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OsmID {
    Node(NodeID),
    Way(WayID),
    Relation(RelationID),
}
impl fmt::Display for OsmID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OsmID::Node(n) => write!(f, "{}", n),
            OsmID::Way(w) => write!(f, "{}", w),
            OsmID::Relation(r) => write!(f, "{}", r),
        }
    }
}
impl OsmID {
    pub fn inner(self) -> i64 {
        match self {
            OsmID::Node(n) => n.0,
            OsmID::Way(w) => w.0,
            OsmID::Relation(r) => r.0,
        }
    }
}
