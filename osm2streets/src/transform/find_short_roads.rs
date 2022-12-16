use geom::Distance;

use crate::{IntersectionControl, RoadID, StreetNetwork};

/// Combines a few different sources/methods to decide which roads are short. Marks them for
/// merging.
///
/// 1) Anything tagged in OSM
/// 2) Anything a temporary local merge_osm_ways.json file
/// 3) If `consolidate_all` is true, an experimental distance-based heuristic
pub fn find_short_roads(streets: &mut StreetNetwork, consolidate_all: bool) -> Vec<RoadID> {
    let mut roads = Vec::new();
    for (id, road) in &streets.roads {
        if road.internal_junction_road {
            roads.push(*id);
            continue;
        }

        if consolidate_all && distance_heuristic(*id, streets) {
            roads.push(*id);
        }
    }

    // Use this to quickly test overrides to some ways before upstreaming in OSM. Since these IDs
    // might be based on already merged roads, do these last.
    for road in streets.roads.values() {
        if road
            .osm_ids
            .iter()
            .any(|id| streets.config.merge_osm_ways.contains(id))
        {
            roads.push(road.id);
        }
    }

    streets.mark_short_roads(roads)
}

fn distance_heuristic(id: RoadID, streets: &StreetNetwork) -> bool {
    let road_length = if let Some(pl) = streets.estimate_trimmed_geometry(id) {
        pl.length()
    } else {
        // The road or something near it collapsed down into a single point or something. This can
        // happen while merging several short roads around a single junction.
        return false;
    };

    // Any road anywhere shorter than this should get merged.
    road_length < Distance::meters(5.0)
}

impl StreetNetwork {
    fn mark_short_roads(&mut self, list: Vec<RoadID>) -> Vec<RoadID> {
        for id in &list {
            self.roads.get_mut(id).unwrap().internal_junction_road = true;
        }
        list
    }

    /// A heuristic to find short roads near traffic signals
    pub fn find_traffic_signal_clusters(&mut self) -> Vec<RoadID> {
        let threshold = Distance::meters(20.0);

        // Simplest start: look for short roads connected to traffic signals.
        //
        // (This will miss sequences of short roads with stop signs in between a cluster of traffic
        // signals)
        //
        // After trying out around Loop 101, what we really want to do is find clumps of 2 or 4
        // traffic signals, find all the segments between them, and merge those.
        let mut results = Vec::new();
        for road in self.roads.values() {
            if road.internal_junction_road {
                continue;
            }
            let src_i = &self.intersections[&road.src_i];
            let dst_i = &self.intersections[&road.dst_i];
            if src_i.is_map_edge() || dst_i.is_map_edge() {
                continue;
            }
            if src_i.control != IntersectionControl::Signalled
                && dst_i.control != IntersectionControl::Signalled
            {
                continue;
            }
            if road
                .untrimmed_road_geometry(self.config.driving_side)
                .length()
                <= threshold
            {
                results.push(road.id);
            }
        }

        self.mark_short_roads(results)
    }
}
