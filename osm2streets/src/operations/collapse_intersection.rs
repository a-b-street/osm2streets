use std::collections::BTreeSet;

use geom::{PolyLine, Pt2D};

use crate::{IntersectionID, RoadID, StreetNetwork};

impl StreetNetwork {
    /// Delete an intersection with exactly two roads. Turn the two roads into one. It's the
    /// caller's responsibility to only call this when appropriate; arbitrarily one of the road's
    /// lanes will be retained.
    pub fn collapse_intersection(&mut self, i: IntersectionID) {
        let roads = self.intersections[&i].roads.clone();
        assert_eq!(roads.len(), 2);
        // Arbitrarily keep the first and delete the second
        let keep_r = roads[0];
        let destroy_r = roads[1];
        assert_ne!(keep_r, destroy_r);

        // Skip loops; they break. Easiest way to detect is see how many total vertices we've got.
        {
            let mut endpts = BTreeSet::new();
            endpts.extend(self.roads[&keep_r].endpoints());
            endpts.extend(self.roads[&destroy_r].endpoints());
            if endpts.len() != 3 {
                info!("Not collapsing degenerate {i}, because it's a loop");
                return;
            }
        }

        // We could be more careful merging highway_type, layer, name, and other attributes, but in
        // practice, it doesn't matter for the short segments we're merging.
        let mut keep_road = self.remove_road(keep_r);
        let destroy_road = self.remove_road(destroy_r);
        self.intersections.remove(&i).unwrap();

        // Remember the merge
        keep_road.osm_ids.extend(destroy_road.osm_ids);

        // There are 4 cases, easy to understand on paper. Preserve the original direction of
        // keep_r. Work with points, not PolyLine::extend. We want to RDP simplify before
        // finalizing.
        let mut new_pts;
        let (new_src_i, new_dst_i) = if keep_road.dst_i == destroy_road.src_i {
            new_pts = keep_road.untrimmed_center_line.clone().into_points();
            new_pts.extend(destroy_road.untrimmed_center_line.into_points());
            (keep_road.src_i, destroy_road.dst_i)
        } else if keep_road.dst_i == destroy_road.dst_i {
            new_pts = keep_road.untrimmed_center_line.clone().into_points();
            new_pts.extend(destroy_road.untrimmed_center_line.reversed().into_points());
            (keep_road.src_i, destroy_road.src_i)
        } else if keep_road.src_i == destroy_road.src_i {
            new_pts = destroy_road.untrimmed_center_line.into_points();
            new_pts.reverse();
            new_pts.extend(keep_road.untrimmed_center_line.clone().into_points());
            (destroy_road.dst_i, keep_road.dst_i)
        } else if keep_road.src_i == destroy_road.dst_i {
            new_pts = destroy_road.untrimmed_center_line.into_points();
            new_pts.extend(keep_road.untrimmed_center_line.clone().into_points());
            (destroy_road.src_i, keep_road.dst_i)
        } else {
            unreachable!()
        };
        // Sanity check
        assert!(i != new_src_i && i != new_dst_i);
        // Simplify curves and dedupe points. The epsilon was tuned for only one location that was
        // breaking
        let epsilon = 1.0;
        keep_road.untrimmed_center_line = PolyLine::must_new(Pt2D::simplify_rdp(new_pts, epsilon));

        // Keep the same ID, but fix the endpoints
        keep_road.src_i = new_src_i;
        keep_road.dst_i = new_dst_i;
        self.insert_road(keep_road);

        // We may need to fix up turn restrictions. destroy_r becomes keep_r.
        let rewrite = |x: &mut RoadID| {
            if *x == destroy_r {
                *x = keep_r;
            }
        };
        for road in self.roads.values_mut() {
            for (_, id) in &mut road.turn_restrictions {
                rewrite(id);
            }

            for (id1, id2) in &mut road.complicated_turn_restrictions {
                rewrite(id1);
                rewrite(id2);
            }
        }
    }
}
