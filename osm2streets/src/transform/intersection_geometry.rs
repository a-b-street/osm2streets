use abstutil::Timer;
use geom::{Circle, Distance};

use crate::{Road, StreetNetwork};

pub fn generate(streets: &mut StreetNetwork, timer: &mut Timer) {
    timer.start_iter(
        "find each intersection polygon",
        streets.intersections.len(),
    );
    // It'd be nice to mutate in the loop, but the borrow checker won't let us
    let mut remove_dangling_nodes = Vec::new();
    let mut set_polygons = Vec::new();

    // Set trim distances for all roads
    for i in streets.intersections.values() {
        timer.next();
        let input_roads = i
            .roads
            .iter()
            .map(|r| streets.roads[r].to_input_road(streets.config.driving_side))
            .collect::<Vec<_>>();
        match crate::intersection_polygon(i.id, input_roads, &i.trim_roads_for_merging) {
            Ok(results) => {
                set_polygons.push((i.id, results.intersection_polygon));
                for (r, dist) in results.trim_starts {
                    streets.roads.get_mut(&r).unwrap().trim_start = dist;
                }
                for (r, dist) in results.trim_ends {
                    streets.roads.get_mut(&r).unwrap().trim_end = dist;
                }
                for (pt, label) in results.debug {
                    streets.debug_point(pt, label);
                }
            }
            Err(err) => {
                error!("Can't make intersection geometry for {}: {}", i.id, err);

                // If we haven't removed disconnected roads, we may have dangling nodes around.
                if let Some(r) = i.roads.iter().next() {
                    // Don't trim lines back at all
                    let road = &streets.roads[r];
                    let pt = if road.src_i == i.id {
                        road.center_line.first_pt()
                    } else {
                        road.center_line.last_pt()
                    };
                    set_polygons.push((i.id, Circle::new(pt, Distance::meters(3.0)).to_polygon()));
                } else {
                    remove_dangling_nodes.push(i.id);
                }
            }
        }
    }

    for road in streets.roads.values_mut() {
        let untrimmed = road.get_untrimmed_center_line(streets.config.driving_side);
        if let Some(pl) =
            Road::trim_polyline_both_ends(untrimmed.clone(), road.trim_start, road.trim_end)
        {
            road.center_line = pl;
        } else {
            // TODO Mark for collapsing?
            error!("{} got trimmed into oblivion", road.id);
            road.center_line = untrimmed;
        }
    }

    for (i, polygon) in set_polygons {
        streets.intersections.get_mut(&i).unwrap().polygon = polygon;
    }
    for i in remove_dangling_nodes {
        streets.intersections.remove(&i).unwrap();
    }
}
