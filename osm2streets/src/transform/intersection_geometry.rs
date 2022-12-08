use abstutil::Timer;
use geom::{Circle, Distance};

use crate::{IntersectionControl, StreetNetwork};

pub fn generate(streets: &mut StreetNetwork, timer: &mut Timer) {
    // TODO intersection_polygon assumes untrimmed lines as input, so reset here. Once we always
    // maintain a trimmed center_line and this transformation goes away entirely, we'll have to
    // revisit how this works.
    for road in streets.roads.values_mut() {
        road.update_center_line(streets.config.driving_side);
    }

    let mut remove_dangling_nodes = Vec::new();
    timer.start_iter(
        "find each intersection polygon",
        streets.intersections.len(),
    );
    // It'd be nice to mutate in the loop, but the borrow checker won't let us
    let mut set_polygons = Vec::new();
    let mut make_stop_signs = Vec::new();
    for i in streets.intersections.values() {
        timer.next();
        let input_roads = i
            .roads
            .iter()
            .map(|r| streets.roads[r].to_input_road())
            .collect::<Vec<_>>();
        match crate::intersection_polygon(i.id, input_roads, &i.trim_roads_for_merging) {
            Ok(results) => {
                set_polygons.push((i.id, results.intersection_polygon));
                for (r, pl) in results.trimmed_center_pts {
                    streets.roads.get_mut(&r).unwrap().center_line = pl;
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

                    // Also don't attempt to make Movements later!
                    make_stop_signs.push(i.id);
                } else {
                    remove_dangling_nodes.push(i.id);
                }
            }
        }
    }
    for (i, polygon) in set_polygons {
        streets.intersections.get_mut(&i).unwrap().polygon = polygon;
    }
    for i in make_stop_signs {
        streets.intersections.get_mut(&i).unwrap().control = IntersectionControl::Signed;
    }
    for i in remove_dangling_nodes {
        streets.intersections.remove(&i).unwrap();
    }
}
