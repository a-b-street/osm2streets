use abstutil::Timer;
use anyhow::Result;

use osm2streets::{ControlType, IntersectionComplexity, IntersectionID, StreetNetwork};

// TODO This needs to update turn restrictions too
pub fn clip_map(streets: &mut StreetNetwork, timer: &mut Timer) -> Result<()> {
    timer.start("clipping map to boundary");

    // So we can use retain without borrowing issues
    let boundary_polygon = streets.boundary_polygon.clone();
    let boundary_ring = boundary_polygon.get_outer_ring();

    // First, just remove roads that both start and end outside the boundary polygon.
    streets.retain_roads(|r| {
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
    let intersection_ids: Vec<IntersectionID> = streets.intersections.keys().cloned().collect();
    for old_id in intersection_ids {
        if streets
            .boundary_polygon
            .contains_pt(streets.intersections[&old_id].point)
        {
            continue;
        }

        let mut old_intersection = streets.intersections.remove(&old_id).unwrap();
        old_intersection.complexity = IntersectionComplexity::MapEdge;
        old_intersection.control = ControlType::Border;

        if old_intersection.roads.len() <= 1 {
            // We don't need to make copies of the intersection; put it back
            streets.intersections.insert(old_id, old_intersection);
            continue;
        }
        for r in old_intersection.roads.clone() {
            let mut copy = old_intersection.clone();
            copy.roads = vec![r];
            copy.id = streets.next_intersection_id();
            // Leave osm_ids alone; all copies of this intersection share provenance

            let road = streets.roads.get_mut(&r).unwrap();
            if road.src_i == old_id {
                road.src_i = copy.id;
            }
            if road.dst_i == old_id {
                road.dst_i = copy.id;
            }

            streets.intersections.insert(copy.id, copy);

            // classify_intersections hasn't happened yet, so we don't need to update the copied
            // intersection
        }
    }

    // Now for all of the border intersections, find the one road they connect to and trim their
    // points.
    for intersection in streets.intersections.values_mut() {
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

        if road.src_i == intersection.id {
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
