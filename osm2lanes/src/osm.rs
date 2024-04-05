//! Useful utilities for working with OpenStreetMap.

pub use osm_reader::{NodeID, OsmID, RelationID, WayID};

// This is a commonly used key in the codebase, so worthy of a bit of typo-prevention
pub const HIGHWAY: &str = "highway";
