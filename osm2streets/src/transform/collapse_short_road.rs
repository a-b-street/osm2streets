use std::collections::{BTreeMap, VecDeque};

use anyhow::Result;

use crate::{ControlType, IntersectionID, RestrictionType, RoadID, StreetNetwork};

// TODO After collapsing a road, trying to drag the surviving intersection in map_editor crashes. I
// bet the underlying problem there would help debug automated transformations near merged roads
// too.
//
// TODO Revisit this after opaque IDs are done. I bet it'll be solved.

impl StreetNetwork {
    /// Collapses a road, merging the two intersections together. Returns (the surviving
    /// intersection, the deleted intersection). The caller can see all roads connected to the
    /// surviving intersection.
    pub fn collapse_short_road(
        &mut self,
        short: RoadID,
    ) -> Result<(IntersectionID, IntersectionID)> {
        // Arbitrarily keep src_i and destroy dst_i.
        let (keep_i, destroy_i) = {
            let r = &self.roads[&short];
            (r.src_i, r.dst_i)
        };

        // If either intersection attached to this road has been deleted, then we're probably
        // dealing with a short segment in the middle of a cluster of intersections. Just delete
        // the segment and move on.
        //
        // TODO Revisit after opaque IDs. How does this happen?
        if !self.intersections.contains_key(&keep_i) || !self.intersections.contains_key(&destroy_i)
        {
            self.remove_road(short);
            bail!("One endpoint of {short} has already been deleted, skipping",);
        }

        // First a sanity check.
        if self.intersections[&keep_i].control == ControlType::Border
            || self.intersections[&destroy_i].control == ControlType::Border
        {
            bail!("{} touches a border", short);
        }

        // TODO Fix up turn restrictions. Many cases:
        // [ ] road we're deleting has simple restrictions
        // [ ] road we're deleting has complicated restrictions
        // [X] road we're deleting is the target of a simple BanTurns restriction
        // [ ] road we're deleting is the target of a simple OnlyAllowTurns restriction
        // [ ] road we're deleting is the target of a complicated restriction
        // [X] road we're deleting is the 'via' of a complicated restriction
        // [ ] road we're deleting has turn lanes that wind up orphaning something

        if keep_i == destroy_i {
            bail!("Can't collapse {short} -- it's a loop on {keep_i}");
        }

        // Remember the original roads attached to each intersection before we merge.
        let mut connected_to_keep_i = self.intersections[&keep_i].roads.clone();
        let mut connected_to_destroy_i = self.intersections[&destroy_i].roads.clone();
        connected_to_keep_i.retain(|x| *x != short);
        connected_to_destroy_i.retain(|x| *x != short);

        // Retain some geometry...
        {
            let mut trim_roads_for_merging = BTreeMap::new();
            for i in [keep_i, destroy_i] {
                for road in self.roads_per_intersection(i) {
                    // If we keep this in there, it might accidentally overwrite the
                    // trim_roads_for_merging key for a surviving road!
                    if road.id == short {
                        continue;
                    }
                    // If we're going to delete this later, don't bother!
                    if road.osm_tags.is("junction", "intersection") {
                        continue;
                    }

                    let pl = self.estimate_trimmed_geometry(road.id).unwrap();
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
            self.intersections
                .get_mut(&keep_i)
                .unwrap()
                .trim_roads_for_merging
                .extend(trim_roads_for_merging);
        }

        self.remove_road(short);

        let destroy_i = self.intersections.remove(&destroy_i).unwrap();

        // If the intersection types differ, upgrade the surviving interesting.
        if destroy_i.control == ControlType::TrafficSignal {
            self.intersections.get_mut(&keep_i).unwrap().control = ControlType::TrafficSignal;
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
        }

        // We just connected a bunch of things to keep_i. Fix ordering and movements.
        self.sort_roads(keep_i);
        self.recalculate_movements(keep_i);

        // If we're deleting the target of a simple restriction somewhere, update it.
        for road in self.roads.values_mut() {
            let mut fix_trs = Vec::new();
            for (rt, to) in road.turn_restrictions.drain(..) {
                if to == short && rt == RestrictionType::BanTurns {
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
                if *via == short {
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

/// Collapse all roads marked with `junction=intersection`
pub fn collapse_all_junction_roads(streets: &mut StreetNetwork) {
    let mut queue: VecDeque<RoadID> = VecDeque::new();
    for (id, road) in &streets.roads {
        if road.osm_tags.is("junction", "intersection") {
            queue.push_back(*id);
        }
    }

    let mut i = 0;
    while !queue.is_empty() {
        let id = queue.pop_front().unwrap();
        i += 1;
        streets.maybe_start_debug_step(format!("collapse road {i}"));
        streets.debug_road(id, "collapse");
        if let Err(err) = streets.collapse_short_road(id) {
            warn!("Not collapsing short road / junction=intersection: {}", err);
        }
    }
}
