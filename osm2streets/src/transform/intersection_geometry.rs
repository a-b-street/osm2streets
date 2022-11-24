use abstutil::Timer;
use geom::{Circle, Distance};

use crate::{IntersectionControl, IntersectionKind, StreetNetwork};

pub fn generate(streets: &mut StreetNetwork, timer: &mut Timer) {
    // Initialize trimmed_center_line to the corrected center
    for road in streets.roads.values_mut() {
        let pl = road.untrimmed_road_geometry().0;
        road.trimmed_center_line = pl;
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
                for (r, (pl, _)) in results.trimmed_center_pts {
                    streets.roads.get_mut(&r).unwrap().trimmed_center_line = pl;
                }
            }
            Err(err) => {
                error!("Can't make intersection geometry for {}: {}", i.id, err);

                // If we haven't removed disconnected roads, we may have dangling nodes around.
                if let Some(r) = i.roads.iter().next() {
                    // Don't trim lines back at all
                    let road = &streets.roads[r];
                    let pt = if road.src_i == i.id {
                        road.trimmed_center_line.first_pt()
                    } else {
                        road.trimmed_center_line.last_pt()
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

    fix_borders(streets);
}

fn fix_borders(streets: &mut StreetNetwork) {
    // Some roads near borders get completely squished. Stretch them out here. Attempting to do
    // this in the streets_reader layer doesn't work, because predicting how much roads will be
    // trimmed is impossible.
    let min_len = Distance::meters(5.0);
    let mut set_polygons = Vec::new();
    for i in streets.intersections.values() {
        if i.kind != IntersectionKind::MapEdge {
            continue;
        }
        let r = i.roads.iter().next().unwrap();
        let road = streets.roads.get_mut(r).unwrap();
        if road.trimmed_center_line.length() >= min_len {
            continue;
        }
        if road.dst_i == i.id {
            road.trimmed_center_line = road.trimmed_center_line.extend_to_length(min_len);
        } else {
            road.trimmed_center_line = road
                .trimmed_center_line
                .reversed()
                .extend_to_length(min_len)
                .reversed();
        }

        // Same boilerplate as above
        let input_roads = i
            .roads
            .iter()
            .map(|r| streets.roads[r].to_input_road())
            .collect::<Vec<_>>();
        let results = crate::intersection_polygon(
            i.id,
            input_roads,
            &streets.intersections[&i.id].trim_roads_for_merging,
        )
        .unwrap();
        set_polygons.push((i.id, results.intersection_polygon));
        for (r, (pl, _)) in results.trimmed_center_pts {
            streets.roads.get_mut(&r).unwrap().trimmed_center_line = pl;
        }
        info!(
            "Shifted border {} out a bit to make the road a reasonable length",
            i.id
        );
    }
    for (i, polygon) in set_polygons {
        streets.intersections.get_mut(&i).unwrap().polygon = polygon;
    }
}
