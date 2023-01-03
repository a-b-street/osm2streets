use std::collections::BTreeMap;

use anyhow::Result;

use crate::{
    IntersectionControl, IntersectionID, IntersectionKind, RestrictionType, RoadID, StreetNetwork,
};

// TODO After collapsing a road, trying to drag the surviving intersection in map_editor crashes. I
// bet the underlying problem there would help debug automated transformations near merged roads
// too.
//
// TODO Revisit this after opaque IDs are done. I bet it'll be solved.

impl StreetNetwork {
    /// Collapses a road, merging the two intersections together. Returns (the surviving
    /// intersection, the deleted intersection). (Note these will be the same when the road's a
    /// loop.)
    pub fn collapse_short_road(
        &mut self,
        short_r: RoadID,
    ) -> Result<(IntersectionID, IntersectionID)> {
        // Arbitrarily keep src_i and destroy dst_i.
        let (keep_i, destroy_i) = {
            let r = &self.roads[&short_r];
            (r.src_i, r.dst_i)
        };

        // First a sanity check.
        if self.intersections[&keep_i].kind == IntersectionKind::MapEdge
            || self.intersections[&destroy_i].kind == IntersectionKind::MapEdge
        {
            bail!("{short_r} touches a map edge");
        }

        // A previous call to this method on nearby roads could produce loop roads. If we later try
        // to collapse this, all we need to do is remove it.
        if keep_i == destroy_i {
            self.remove_road(short_r);
            return Ok((keep_i, keep_i));
        }

        // Remember the original roads attached to each intersection before we merge.
        let mut connected_to_keep_i = self.intersections[&keep_i].roads.clone();
        let mut connected_to_destroy_i = self.intersections[&destroy_i].roads.clone();
        connected_to_keep_i.retain(|x| *x != short_r);
        connected_to_destroy_i.retain(|x| *x != short_r);

        // Retain some geometry...
        {
            let mut trim_roads_for_merging = BTreeMap::new();
            for i in [keep_i, destroy_i] {
                for road in self.roads_per_intersection(i) {
                    // If we keep this in there, it might accidentally overwrite the
                    // trim_roads_for_merging key for a surviving road!
                    if road.id == short_r {
                        continue;
                    }
                    // If we're going to delete this later, don't bother!
                    if road.internal_junction_road {
                        continue;
                    }

                    if let Some(pl) = self.estimate_trimmed_geometry(road.id) {
                        if road.src_i == i {
                            if trim_roads_for_merging.contains_key(&(road.id, true)) {
                                panic!(
                                    "trim_roads_for_merging has a src_i duplicate for {}",
                                    road.id
                                );
                            }
                            trim_roads_for_merging.insert((road.id, true), pl.first_pt());
                        } else {
                            if trim_roads_for_merging.contains_key(&(road.id, false)) {
                                panic!(
                                    "trim_roads_for_merging has a dst_i duplicate for {}",
                                    road.id
                                );
                            }
                            trim_roads_for_merging.insert((road.id, false), pl.last_pt());
                        }
                    }
                }
            }
            self.intersections
                .get_mut(&keep_i)
                .unwrap()
                .trim_roads_for_merging
                .extend(trim_roads_for_merging);
        }

        self.remove_road(short_r);

        let destroy_i = self.intersections.remove(&destroy_i).unwrap();

        // If the intersection types differ, upgrade the surviving interesting.
        if destroy_i.control == IntersectionControl::Signalled {
            self.intersections.get_mut(&keep_i).unwrap().control = IntersectionControl::Signalled;
        }

        // Remember the merge
        self.intersections
            .get_mut(&keep_i)
            .unwrap()
            .osm_ids
            .extend(destroy_i.osm_ids);

        // Fix the endpoint of all roads connected to destroy_i.
        for r in destroy_i.roads {
            self.intersections.get_mut(&keep_i).unwrap().roads.push(r);

            let road = self.roads.get_mut(&r).unwrap();
            if road.src_i == destroy_i.id {
                road.src_i = keep_i;
            } else {
                assert_eq!(road.dst_i, destroy_i.id);
                road.dst_i = keep_i;
            }
            // Consider when two dual carriageways intersect. After collapsing 3 of the short
            // roads, the last short road will wind up with src_i == dst_i. We could consider
            // removing the road here, or revisiting the special case above of collapsing a loop.
        }

        // We just connected a bunch of things to keep_i. Fix ordering and movements.
        self.sort_roads(keep_i);
        self.update_i(keep_i);

        // TODO Fix up turn restrictions. Many cases:
        // [ ] road we're deleting has simple restrictions
        // [ ] road we're deleting has complicated restrictions
        // [X] road we're deleting is the target of a simple BanTurns restriction
        // [ ] road we're deleting is the target of a simple OnlyAllowTurns restriction
        // [ ] road we're deleting is the target of a complicated restriction
        // [X] road we're deleting is the 'via' of a complicated restriction
        // [ ] road we're deleting has turn lanes that wind up orphaning something

        // If we're deleting the target of a simple restriction somewhere, update it.
        for road in self.roads.values_mut() {
            let mut fix_trs = Vec::new();
            for (rt, to) in road.turn_restrictions.drain(..) {
                if to == short_r && rt == RestrictionType::BanTurns {
                    // Remove this restriction, and replace it with a new one to each of the
                    // successors of the deleted road. Depending if the intersection we kept is the
                    // one connecting these two roads, the successors differ.
                    if connected_to_keep_i.contains(&road.id) {
                        for x in &connected_to_destroy_i {
                            fix_trs.push((rt, *x));
                        }
                    } else {
                        for x in &connected_to_keep_i {
                            fix_trs.push((rt, *x));
                        }
                    }
                } else {
                    fix_trs.push((rt, to));
                }
            }
            road.turn_restrictions = fix_trs;
        }

        // If we're deleting the 'via' of a complicated restriction somewhere, change it to a
        // simple restriction.
        for road in self.roads.values_mut() {
            let mut add = Vec::new();
            road.complicated_turn_restrictions.retain(|(via, to)| {
                if *via == short_r {
                    add.push((RestrictionType::BanTurns, *to));
                    false
                } else {
                    true
                }
            });
            road.turn_restrictions.extend(add);
        }

        Ok((keep_i, destroy_i.id))
    }
}
