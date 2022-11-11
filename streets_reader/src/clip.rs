use abstutil::Timer;
use anyhow::Result;
use osm2streets::{osm, ControlType, IntersectionComplexity, StreetNetwork};

// TODO This needs to update turn restrictions too
pub fn clip_map(streets: &mut StreetNetwork, timer: &mut Timer) -> Result<()> {
    timer.start("clipping map to boundary");

    // So we can use retain without borrowing issues
    let boundary_polygon = streets.boundary_polygon.clone();
    let boundary_ring = boundary_polygon.get_outer_ring();

    // First, just remove roads that both start and end outside the boundary polygon.
    streets.retain_roads(|_, r| {
        let first_in = boundary_polygon.contains_pt(r.untrimmed_center_line.first_pt());
        let last_in = boundary_polygon.contains_pt(r.untrimmed_center_line.last_pt());
        let light_rail_ok = if r.is_light_rail() {
            // Make sure it's in the boundary somewhere
            r.untrimmed_center_line
                .points()
                .iter()
                .any(|pt| boundary_polygon.contains_pt(*pt))
        } else {
            false
        };
        first_in || last_in || light_rail_ok
    });

    // Get rid of orphaned intersections too
    streets.intersections.retain(|_, i| !i.roads.is_empty());

    // Look for intersections outside the map. If they happen to be connected to multiple roads,
    // then we'll need to copy the intersection for each connecting road. This effectively
    // disconnects two roads in the map that would be connected if we left in some
    // partly-out-of-bounds road.
    //
    // Do this in one step, since we have to fix up road IDs carefully. The order of steps in here
    // is a bit sensitive (because remove_road needs both intersections to exist, and due to the
    // borrow checker).
    let intersection_ids: Vec<osm::NodeID> = streets.intersections.keys().cloned().collect();

    // Use negative values to avoid conflicting with OSM
    let mut next_osm_id = -1;

    for id in intersection_ids {
        let intersection = &streets.intersections[&id];
        if streets.boundary_polygon.contains_pt(intersection.point) {
            continue;
        }

        let mut intersection = streets.intersections.get_mut(&id).unwrap();
        intersection.complexity = IntersectionComplexity::MapEdge;
        intersection.control = ControlType::Border;

        if intersection.roads.len() > 1 {
            for r in intersection.roads.clone() {
                let mut road = streets.remove_road(&r);

                let mut copy = streets.intersections[&id].clone();
                copy.roads.clear();

                let new_id = osm::NodeID(next_osm_id);
                next_osm_id -= 1;
                let mut fixed_road_id = r;
                if fixed_road_id.i1 == id {
                    fixed_road_id.i1 = new_id;
                }
                if fixed_road_id.i2 == id {
                    fixed_road_id.i2 = new_id;
                }
                assert_ne!(r, fixed_road_id);
                road.id = fixed_road_id;

                streets.intersections.insert(new_id, copy);
                streets.insert_road(fixed_road_id, road);
            }

            assert!(streets.intersections[&id].roads.is_empty());
            streets.intersections.remove(&id).unwrap();
        }
    }

    // Now for all of the border intersections, find the one road they connect to and trim their
    // points.
    for (i, intersection) in &mut streets.intersections {
        if intersection.control != ControlType::Border {
            continue;
        }
        assert_eq!(intersection.roads.len(), 1);
        let r = intersection.roads[0];

        let road = streets.roads.get_mut(&r).unwrap();
        let border_pts = boundary_ring.all_intersections(&road.untrimmed_center_line);
        if border_pts.is_empty() {
            // The intersection is out-of-bounds, but a road leading to it doesn't cross the
            // boundary?
            warn!("{} interacts with border strangely", r);
            continue;
        }

        if r.i1 == *i {
            // Starting out-of-bounds
            let border_pt = border_pts[0];
            if let Some(pl) = road.untrimmed_center_line.get_slice_starting_at(border_pt) {
                road.untrimmed_center_line = pl;
                intersection.point = road.untrimmed_center_line.first_pt();
            } else {
                warn!("{} interacts with border strangely", r);
                continue;
            }
        } else {
            // Ending out-of-bounds
            // For light rail, the center-line might cross the boundary twice. When we're looking
            // at the outbound end, take the last time we hit the boundary
            let border_pt = *border_pts.last().unwrap();
            if let Some(pl) = road.untrimmed_center_line.get_slice_ending_at(border_pt) {
                road.untrimmed_center_line = pl;
                intersection.point = road.untrimmed_center_line.last_pt();
            } else {
                warn!("{} interacts with border strangely", r);
                continue;
            }
        }
    }

    if streets.roads.is_empty() {
        bail!("There are no roads inside the clipping polygon");
    }

    timer.stop("clipping map to boundary");
    Ok(())
}
