use std::collections::BTreeSet;

use crate::{RoadID, StreetNetwork};

/// Some roads might be totally disconnected from the largest clump because of how the map's
/// bounding polygon was drawn, or bad map data, or which roads are filtered from OSM. Remove them.
pub fn remove_disconnected_roads(streets: &mut StreetNetwork) {
    // This is a simple floodfill, not Tarjan's. Assumes all roads bidirectional.

    let mut partitions: Vec<Vec<RoadID>> = Vec::new();
    let mut unvisited_roads: BTreeSet<RoadID> = streets
        .roads
        .iter()
        .filter_map(|(id, r)| if r.is_light_rail() { None } else { Some(*id) })
        .collect();

    while !unvisited_roads.is_empty() {
        let mut queue_roads: Vec<RoadID> = vec![*unvisited_roads.iter().next().unwrap()];
        let mut current_partition: Vec<RoadID> = Vec::new();
        while !queue_roads.is_empty() {
            let current = queue_roads.pop().unwrap();
            if !unvisited_roads.contains(&current) {
                continue;
            }
            unvisited_roads.remove(&current);
            current_partition.push(current);

            for i in streets.roads[&current].endpoints() {
                queue_roads.extend(streets.intersections[&i].roads.clone());
            }
        }
        partitions.push(current_partition);
    }

    partitions.sort_by_key(|roads| roads.len());
    partitions.reverse();
    for p in partitions.iter().skip(1) {
        for id in p {
            info!("Removing {} because it's disconnected from most roads", id);
            streets.remove_road(*id);
        }
    }

    // Also remove cul-de-sacs here. TODO Support them properly, but for now, they mess up parking
    // hint matching (loop PolyLine) and pathfinding later.
    streets.retain_roads(|r| r.src_i != r.dst_i);

    // Remove intersections without any roads
    streets.intersections.retain(|_, i| !i.roads.is_empty());
}
