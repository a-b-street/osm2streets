use std::collections::BTreeMap;

use geom::{Distance, Polygon, Pt2D};
use serde::{Deserialize, Serialize};

use crate::{osm, ConflictType, ControlType, IntersectionComplexity, Movement, OriginalRoad};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Intersection {
    pub id: osm::NodeID,

    /// Represents the original place where OSM center-lines meet. This may be meaningless beyond
    /// StreetNetwork; roads and intersections get merged and deleted.
    pub point: Pt2D,
    /// This will be a placeholder until `Transformation::GenerateIntersectionGeometry` runs.
    pub polygon: Polygon,
    pub complexity: IntersectionComplexity,
    pub conflict_level: ConflictType,
    pub control: ControlType,
    pub elevation: Distance,

    /// All roads connected to this intersection. They may be incoming or outgoing relative to this
    /// intersection. They're ordered clockwise aroundd the intersection.
    pub roads: Vec<OriginalRoad>,
    pub movements: Vec<Movement>,

    // true if src_i matches this intersection (or the deleted/consolidated one, whatever)
    pub trim_roads_for_merging: BTreeMap<(osm::WayID, bool), Pt2D>,
}

impl Intersection {
    pub fn new(
        id: osm::NodeID,
        point: Pt2D,
        complexity: IntersectionComplexity,
        conflict_level: ConflictType,
        control: ControlType,
    ) -> Self {
        Self {
            id,
            point,
            polygon: Polygon::dummy(),
            complexity,
            conflict_level,
            control,
            // Filled out later
            roads: Vec::new(),
            movements: Vec::new(),
            elevation: Distance::ZERO,
            trim_roads_for_merging: BTreeMap::new(),
        }
    }

    pub fn is_border(&self) -> bool {
        self.control == ControlType::Border
    }
}
