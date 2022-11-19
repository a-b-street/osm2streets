use std::collections::{BTreeMap, VecDeque};

use anyhow::Result;

use crate::{
    osm, CommonEndpoint, ControlType, OriginalRoad, RestrictionType, RoadWithEndpoints,
    StreetNetwork,
};

// TODO After collapsing a road, trying to drag the surviving intersection in map_editor crashes. I
// bet the underlying problem there would help debug automated transformations near merged roads
// too.
//
// TODO Revisit this after opaque IDs are done. I bet it'll be solved.

impl StreetNetwork {
    /// Collapses a road, merging the two intersections together. Returns (the surviving
    /// intersection, the deleted intersection, deleted roads, new roads)
    pub fn collapse_short_road(
        &mut self,
        short: OriginalRoad,
    ) -> Result<(
        osm::NodeID,
        osm::NodeID,
        Vec<OriginalRoad>,
        Vec<OriginalRoad>,
    )> {
        let (src_i, dst_i) = {
            let r = &self.roads[&short];
            (r.src_i, r.dst_i)
        };

        let short_endpts = RoadWithEndpoints::new(&self.roads[&short]);

        // If either intersection attached to this road has been deleted, then we're probably
        // dealing with a short segment in the middle of a cluster of intersections. Just delete
        // the segment and move on.
        //
        // TODO Revisit after opaque IDs. How does this happen?
        if !self.intersections.contains_key(&src_i) || !self.intersections.contains_key(&dst_i) {
            self.remove_road(short);
            bail!("One endpoint of {short} has already been deleted, skipping",);
        }

        // First a sanity check.
        if self.intersections[&src_i].control == ControlType::Border
            || self.intersections[&dst_i].control == ControlType::Border
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

        if src_i == dst_i {
            bail!("Can't collapse {short} -- it's a loop on {src_i}");
        }
        // Remember the original connections to src_i before we merge. None of these will change
        // IDs.
        let mut connected_to_src_i = self.intersections[&src_i].roads.clone();
        connected_to_src_i.retain(|x| *x != short);

        // Retain some geometry...
        {
            let mut trim_roads_for_merging = BTreeMap::new();
            for i in [src_i, dst_i] {
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
                        if trim_roads_for_merging.contains_key(&(road.id.osm_way_id, true)) {
                            panic!(
                                "trim_roads_for_merging has a src_i duplicate for {}",
                                road.id
                            );
                        }
                        trim_roads_for_merging.insert((road.id.osm_way_id, true), pl.first_pt());
                    } else {
                        if trim_roads_for_merging.contains_key(&(road.id.osm_way_id, false)) {
                            panic!(
                                "trim_roads_for_merging has a dst_i duplicate for {}",
                                road.id
                            );
                        }
                        trim_roads_for_merging.insert((road.id.osm_way_id, false), pl.last_pt());
                    }
                }
            }
            self.intersections
                .get_mut(&src_i)
                .unwrap()
                .trim_roads_for_merging
                .extend(trim_roads_for_merging);
        }

        self.remove_road(short);

        // Arbitrarily keep src_i and destroy dst_i. Don't actually remove the intersection until
        // later; remove_road needs the intersection to exist

        // Fix up all roads connected to dst_i. Delete them and create a new copy; the ID changes,
        // since one intersection changes.
        let mut deleted = vec![short];
        let mut created = Vec::new();
        let mut old_to_new = BTreeMap::new();
        let mut new_to_old = BTreeMap::new();
        for r in self.intersections[&dst_i].roads.clone() {
            deleted.push(r);
            let mut road = self.remove_road(r);
            let old_endpts = RoadWithEndpoints::new(&road);

            let mut new_id = r;
            if road.src_i == dst_i {
                new_id.i1 = src_i;
                road.src_i = src_i;
            } else {
                assert_eq!(road.dst_i, dst_i);
                new_id.i2 = src_i;
                road.dst_i = src_i;
            }
            road.id = new_id;

            if new_id.i1 == new_id.i2 {
                // When collapsing many roads around some junction, we wind up with loops. We can
                // immediately discard those.
                continue;
            }

            old_to_new.insert(r, new_id);
            new_to_old.insert(new_id, old_endpts);

            self.insert_road(road);
            created.push(new_id);
        }

        // If the intersection types differ, upgrade the surviving interesting.
        {
            // Don't use delete_intersection; we're manually fixing up connected roads
            let i = self.intersections.remove(&dst_i).unwrap();
            if i.control == ControlType::TrafficSignal {
                self.intersections.get_mut(&src_i).unwrap().control = ControlType::TrafficSignal;
            }
        }

        // If we're deleting the target of a simple restriction somewhere, update it.
        for (from_id, road) in &mut self.roads {
            let from_endpt = new_to_old
                .get(from_id)
                .cloned()
                .unwrap_or(RoadWithEndpoints::new(road));

            let mut fix_trs = Vec::new();
            for (rt, to) in road.turn_restrictions.drain(..) {
                if to == short && rt == RestrictionType::BanTurns {
                    // Remove this restriction, replace it with a new one to each of the successors
                    // of the deleted road. Depending if the intersection we kept is the one
                    // connecting these two roads, the successors differ.
                    if from_endpt.common_endpoint(&short_endpts) == CommonEndpoint::One(src_i) {
                        for x in &created {
                            fix_trs.push((rt, *x));
                        }
                    } else {
                        // It cant be CommonEndpoint::Both, because we bail out on loop roads
                        // earlier
                        for x in &connected_to_src_i {
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
                    // Depending which intersection we're deleting, the ID of 'to' might change
                    let to_id = old_to_new.get(to).cloned().unwrap_or(*to);
                    add.push((RestrictionType::BanTurns, to_id));
                    false
                } else {
                    true
                }
            });
            road.turn_restrictions.extend(add);
        }

        Ok((src_i, dst_i, deleted, created))
    }
}

/// Collapse all roads marked with `junction=intersection`
pub fn collapse_all_junction_roads(streets: &mut StreetNetwork) {
    let mut queue: VecDeque<OriginalRoad> = VecDeque::new();
    for (id, road) in &streets.roads {
        if road.osm_tags.is("junction", "intersection") {
            queue.push_back(*id);
        }
    }

    let mut i = 0;
    while !queue.is_empty() {
        let id = queue.pop_front().unwrap();

        // The road might've been deleted by a previous collapse_short_road call
        if !streets.roads.contains_key(&id) {
            continue;
        }

        i += 1;
        streets.maybe_start_debug_step(format!("collapse road {i}"));
        streets.debug_road(id, "collapse");
        match streets.collapse_short_road(id) {
            Ok((_, _, _, new_roads)) => {
                // Some road IDs still in the queue might have changed, so check the new_roads for
                // anything we should try to collapse
                for r in new_roads {
                    if streets.roads[&r].osm_tags.is("junction", "intersection") {
                        queue.push_back(r);
                    }
                }
            }
            Err(err) => {
                warn!("Not collapsing short road / junction=intersection: {}", err);
            }
        }
    }
}
