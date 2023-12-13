//! Useful utilities for working with OpenStreetMap.

pub use osm_reader::{NodeID, OsmID, RelationID, WayID};

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
