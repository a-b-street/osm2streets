use std::collections::BTreeMap;

use abstutil::Tags;
use geom::{GPSBounds, Pt2D};

pub use self::multipolygon::glue_multipolygon;
use osm2streets::osm::{NodeID, OsmID, RelationID, WayID};

mod clip;
mod multipolygon;
mod reader;

pub struct Document {
    // This is guaranteed to be filled out after Document::read
    pub gps_bounds: Option<GPSBounds>,
    pub nodes: BTreeMap<NodeID, Node>,
    pub ways: BTreeMap<WayID, Way>,
    pub relations: BTreeMap<RelationID, Relation>,

    /// These ways share a WayID, but each have different pts
    pub clipped_copied_ways: Vec<(WayID, Way)>,
}

pub struct Node {
    pub pt: Pt2D,
    pub tags: Tags,
}

#[derive(Clone)]
pub struct Way {
    // Duplicates geometry, because it's convenient
    pub nodes: Vec<NodeID>,
    pub pts: Vec<Pt2D>,
    pub tags: Tags,
    pub version: Option<usize>,
}

pub struct Relation {
    pub tags: Tags,
    /// Role, member
    pub members: Vec<(String, OsmID)>,
}
