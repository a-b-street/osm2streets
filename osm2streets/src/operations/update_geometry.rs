use geom::{Circle, Distance};

use crate::{IntersectionID, Road, StreetNetwork};

impl StreetNetwork {
    /// Recalculates trim distances and intersection geometry. This is idempotent; it doesn't use
    /// any results from the previous call.
    pub(crate) fn update_geometry(&mut self, id: IntersectionID) {
        let i = &self.intersections[&id];

        // Update the polygon and set trim distances for roads
        let input_roads = i
            .roads
            .iter()
            .map(|r| self.roads[r].to_input_road(self.config.driving_side))
            .collect::<Vec<_>>();
        match crate::intersection_polygon(i.id, i.kind, input_roads, &i.trim_roads_for_merging) {
            Ok(results) => {
                self.intersections.get_mut(&id).unwrap().polygon = results.intersection_polygon;

                for (r, dist) in results.trim_starts {
                    self.roads.get_mut(&r).unwrap().trim_start = dist;
                }
                for (r, dist) in results.trim_ends {
                    self.roads.get_mut(&r).unwrap().trim_end = dist;
                }
                for (pt, label) in results.debug {
                    self.debug_point(pt, label);
                }
            }
            Err(err) => {
                error!("Can't make intersection geometry for {}: {}", i.id, err);

                let r = i.roads[0];
                // Don't trim lines back at all
                let road = &self.roads[&r];
                let pt = if road.src_i == i.id {
                    road.reference_line.first_pt()
                } else {
                    road.reference_line.last_pt()
                };
                self.intersections.get_mut(&id).unwrap().polygon =
                    Circle::new(pt, Distance::meters(3.0)).to_polygon();
            }
        }

        // Update road center lines based on the trim. Note update_geometry works on one
        // intersection at a time, so it's possible some roads haven't been trimmed yet on the
        // other side.
        for r in &self.intersections[&id].roads {
            let road = self.roads.get_mut(r).unwrap();

            let untrimmed = road.get_untrimmed_center_line(self.config.driving_side);
            if let Some(pl) =
                Road::trim_polyline_both_ends(untrimmed.clone(), road.trim_start, road.trim_end)
            {
                road.center_line = pl;
            } else {
                error!("{} got trimmed into oblivion, collapse it later", road.id);
                road.center_line = untrimmed;
                // Collapse it later
                road.internal_junction_road = true;
            }
        }
    }
}
