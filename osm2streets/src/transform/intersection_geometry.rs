use abstutil::Timer;
use geom::{Circle, Distance};

use crate::{IntersectionControl, IntersectionKind, StreetNetwork};

pub fn generate(streets: &mut StreetNetwork, timer: &mut Timer) {
    let mut remove_dangling_nodes = Vec::new();
    timer.start_iter(
        "find each intersection polygon",
        streets.intersections.len(),
    );
    // It'd be nice to mutate in the loop, but the borrow checker won't let us
    let mut set_polygons = Vec::new();
    let mut make_stop_signs = Vec::new();

    // And actually, we don't want to mutate center_lines until the very end. The input to the
    // geometry algorithm should be UNTRIMMED center lines. Every road will get trimmed twice, once
    // at each end. When calculating the opposite end, we want to use the full untrimmed line for
    // all roads.
    let mut trimmed_center_lines = Vec::new();

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
                    trimmed_center_lines.push((r, i.id, pl));
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

    for (r, i, pl) in trimmed_center_lines {
        let road = streets.roads.get_mut(&r).unwrap();
        let maybe_slice = if i == road.src_i {
            // pl is trimmed on the start side
            road.center_line.safe_get_slice_starting_at(pl.first_pt())
        } else {
            road.center_line.safe_get_slice_ending_at(pl.last_pt())
        };
        if let Some(slice) = maybe_slice {
            road.center_line = slice;
        } else {
            // This happens when trimming on the other side actually "eats away" past this side.
            // The road is probably an internal_junction_road. The two intersection polygons will
            // physically overlap each other.
            //
            // TODO Or... service_road_loop's deadend at the bottom. We need to EXTEND the line
            // sometimes.
            error!("Can't trim {r} on the {i} end");
        }
    }

    fix_map_edges(streets);
}

fn fix_map_edges(streets: &mut StreetNetwork) {
    // Some roads near map edges get completely squished. Stretch them out here. Attempting to do
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
        if road.center_line.length() >= min_len {
            continue;
        }
        if road.dst_i == i.id {
            road.center_line = road.center_line.extend_to_length(min_len);
        } else {
            road.center_line = road
                .center_line
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
        for (r, pl) in results.trimmed_center_pts {
            streets.roads.get_mut(&r).unwrap().center_line = pl;
        }
        info!(
            "Shifted map edge {} out a bit to make the road a reasonable length",
            i.id
        );
    }
    for (i, polygon) in set_polygons {
        streets.intersections.get_mut(&i).unwrap().polygon = polygon;
    }
}
