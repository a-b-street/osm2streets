use std::collections::BTreeMap;

use geom::{Distance, Polygon, Pt2D};
use serde::{Deserialize, Serialize};

use crate::{
    osm, IntersectionControl, IntersectionID, IntersectionKind, Movement, RoadID, StreetNetwork,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Intersection {
    pub id: IntersectionID,
    /// The OSM nodes making up this intersection. Multiple intersections may share the same OSM
    /// nodes (when an out-of-bounds intersection connected to multiple roads is clipped). One
    /// intersection may have multiple OSM nodes (when the intersection is consolidated).
    pub osm_ids: Vec<osm::NodeID>,

    /// Represents the original place where OSM center-lines meet. This may be meaningless beyond
    /// StreetNetwork; roads and intersections get merged and deleted.
    pub point: Pt2D,
    /// This will be a placeholder until `Transformation::GenerateIntersectionGeometry` runs.
    pub polygon: Polygon,
    pub kind: IntersectionKind,
    pub control: IntersectionControl,
    pub elevation: Distance,

    /// All roads connected to this intersection. They may be incoming or outgoing relative to this
    /// intersection. They're ordered clockwise aroundd the intersection.
    pub roads: Vec<RoadID>,
    pub movements: Vec<Movement>,

    // true if src_i matches this intersection (or the deleted/consolidated one, whatever)
    // TODO Store start/end trim distance on _every_ road
    pub trim_roads_for_merging: BTreeMap<(RoadID, bool), Pt2D>,
}

impl StreetNetwork {
    pub fn next_intersection_id(&mut self) -> IntersectionID {
        let id = IntersectionID(self.intersection_id_counter);
        self.intersection_id_counter += 1;
        id
    }

    /// This creates a new intersection based on one or more real OSM nodes, assigning an ID and
    /// returning it.
    pub fn insert_intersection(
        &mut self,
        osm_ids: Vec<osm::NodeID>,
        point: Pt2D,
        t: IntersectionKind,
        control: IntersectionControl,
    ) -> IntersectionID {
        let id = self.next_intersection_id();
        self.intersections.insert(
            id,
            Intersection {
                id,
                osm_ids,
                point,
                polygon: Polygon::dummy(),
                kind: t,
                control,
                // Filled out later
                roads: Vec::new(),
                movements: Vec::new(),
                elevation: Distance::ZERO,
                trim_roads_for_merging: BTreeMap::new(),
            },
        );
        id
    }
}

impl Intersection {
    pub fn is_map_edge(&self) -> bool {
        self.kind == IntersectionKind::MapEdge
    }
}
