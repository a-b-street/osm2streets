use std::collections::{BTreeMap, BTreeSet};

use geom::Distance;

use crate::{IntersectionID, Road, RoadID, RoadWithEndpoints, StreetNetwork};

pub fn merge(streets: &mut StreetNetwork) {
    for i in streets.intersections.keys() {
        // Progressively detect more stuff. Display the most detail possible.
        if let Some(mc) = MultiConnection::new(streets, *i) {
            // TODO Ignore opposite direction of one we've already found?
            if let Some(dc1) = DualCarriagewayPt1::new(streets, &mc) {
                if let Some(dc2) = DualCarriagewayPt2::new(streets, &dc1) {
                    dc2.debug(streets);
                } else {
                    dc1.debug(streets);
                }
            } else {
                mc.debug(streets);
            }

            // TODO Just work on one right now
            break;
        }
    }
}

// TODO We should do this in classify_intersections.rs?
// Step 1: just find where dual carriageways start or end
struct MultiConnection {
    i: IntersectionID,
    side1: RoadID,
    side2: RoadID,
    road_name: String,
}

impl MultiConnection {
    fn new(streets: &StreetNetwork, i: IntersectionID) -> Option<Self> {
        let roads = streets.roads_per_intersection(i);
        if roads.len() < 3 {
            return None;
        }

        // First group roads by name.
        let mut roads_by_name: BTreeMap<String, Vec<&Road>> = BTreeMap::new();
        for road in roads {
            // Skip unnamed roads for now
            if let Some(name) = &road.name {
                roads_by_name
                    .entry(name.to_string())
                    .or_insert_with(Vec::new)
                    .push(road);
            }
        }

        // Look for a group of 3. Two should be one-way for driving, the other shouldn't.
        for (road_name, groups) in roads_by_name {
            if groups.len() != 3 {
                continue;
            }
            let mut oneway_roads = Vec::new();
            let mut bidi_roads = Vec::new();
            for road in groups {
                if road.oneway_for_driving().is_some() {
                    oneway_roads.push(road);
                } else {
                    bidi_roads.push(road);
                }
            }

            if oneway_roads.len() != 2 || bidi_roads.len() != 1 {
                continue;
            }

            // Preserving notes about old detection:
            // - look for dual_carriageway=yes tag, but since it's rarely there, maybe just use as an "opt
            //   in" technique
            // - make sure the 2 oneway roads are within 10 degrees of the bidi road?
            // - The two one-ways should point at each other
            // - Maybe one intersection could be a MultiConnection for two roads? For now just take one.

            return Some(Self {
                i,
                side1: oneway_roads.remove(0).id,
                side2: oneway_roads.remove(0).id,
                road_name,
            });
        }

        None
    }

    fn debug(&self, streets: &StreetNetwork) {
        streets.debug_intersection(self.i, "join/split that isnt DC");
        streets.debug_road(self.side1, "side1 of failed DC");
        streets.debug_road(self.side2, "side2 of failed DC");
    }
}

// Step 2: trace both sequences of one-ways between the intersections where a dual carriageway
// splits/rejoins
struct DualCarriagewayPt1 {
    road_name: String,
    src_i: IntersectionID,
    dst_i: IntersectionID,
    // side1 points from src_i to dst_i
    side1: Vec<RoadWithEndpoints>,
    // side2 points from dst_i to src_i
    side2: Vec<RoadWithEndpoints>,
}

impl DualCarriagewayPt1 {
    fn new(streets: &StreetNetwork, mc: &MultiConnection) -> Option<Self> {
        let (side1, dst_i_v1) = Self::trace_side(streets, mc.side1, mc.i, &mc.road_name)?;
        let (side2, dst_i_v2) = Self::trace_side(streets, mc.side2, mc.i, &mc.road_name)?;

        // TODO Something very odd has happened. Make a new copy of the map for debugging and label
        // the strangeness.
        if dst_i_v1 != dst_i_v2 {
            return None;
        }

        let mut side1 = Self::orient_oneways(side1)?;
        let mut side2 = Self::orient_oneways(side2)?;

        // Which one goes from src_i->dst_i?
        let src_i = mc.i;
        let dst_i = dst_i_v1;
        for swap in [false, true] {
            if swap {
                std::mem::swap(&mut side1, &mut side2);
            }
            let side1_endpts = (side1[0].src_i, side1.last().as_ref().unwrap().dst_i);
            let side2_endpts = (side2[0].src_i, side2.last().as_ref().unwrap().dst_i);

            if side1_endpts == (src_i, dst_i) {
                if side2_endpts != (dst_i, src_i) {
                    // Why doesn't the other side point the opposite way?
                    return None;
                }
                return Some(Self {
                    road_name: mc.road_name.clone(),
                    src_i,
                    dst_i,
                    side1,
                    side2,
                });
            }
        }
        None
    }

    fn debug(&self, streets: &StreetNetwork) {
        streets.debug_intersection(self.src_i, format!("start of {}", self.road_name));
        streets.debug_intersection(self.dst_i, "end");
        for (idx, r) in self.side1.iter().enumerate() {
            streets.debug_road(r.road, format!("side1, {idx}"));
        }
        for (idx, r) in self.side2.iter().enumerate() {
            streets.debug_road(r.road, format!("side2, {idx}"));
        }
    }

    // Chase a one-way while the road name stays the same. Also returns the last intersection
    // found, where the one-ways end.
    fn trace_side(
        streets: &StreetNetwork,
        start: RoadID,
        join: IntersectionID,
        road_name: &str,
    ) -> Option<(Vec<RoadWithEndpoints>, IntersectionID)> {
        let mut sequence = vec![RoadWithEndpoints::new(&streets.roads[&start])];

        let mut current = sequence[0].clone();
        let mut last_i = join;
        'LOOP: loop {
            let other_side = current.other_side(last_i);
            for road in streets.roads_per_intersection(other_side) {
                // TODO Helper method to just find roads originating at other_side and pointing
                // away (or towards) something?
                if road.id == current.road {
                    continue;
                }
                if road.name == Some(road_name.to_string()) {
                    if road.oneway_for_driving().is_some() {
                        current = RoadWithEndpoints::new(road);
                        sequence.push(current.clone());
                        last_i = other_side;
                        continue 'LOOP;
                    }
                    // We found the bidirectional piece. Assume it's the other end.
                    return Some((sequence, other_side));
                }
            }
            // We didn't find a next step?
            return None;
        }
    }

    // The input should already be ordered so that the first road points at the second, or reversed
    // relative to the way the one-ways are defined. Flip the order if needed.
    fn orient_oneways(mut seq: Vec<RoadWithEndpoints>) -> Option<Vec<RoadWithEndpoints>> {
        for reverse in [false, true] {
            if reverse {
                seq.reverse();
            }
            if seq.windows(2).all(|pair| pair[0].dst_i == pair[1].src_i) {
                return Some(seq);
            }
        }
        // The input was broken somehow
        return None;
    }
}

// Step 3: find "branch" roads that lead away from either side, and "bridge" roads linking the two
// sides
struct DualCarriagewayPt2 {
    road_name: String,
    src_i: IntersectionID,
    dst_i: IntersectionID,
    // side1 points from src_i to dst_i
    side1: Vec<RoadWithEndpoints>,
    // side2 points from dst_i to src_i
    side2: Vec<RoadWithEndpoints>,

    // The branches also track the linear untrimmed distance from the beginning of the side
    side1_branches: Vec<(RoadID, Distance)>,
    side2_branches: Vec<(RoadID, Distance)>,
    // The linear untrimmed distance is relative to side1 (src_i -> dst_i). Only bridges consisting of a
    // single RoadID are detected; no multi-step ones yet.
    bridges: Vec<(RoadID, Distance)>,

    side1_length: Distance,
    side2_length: Distance,
}

impl DualCarriagewayPt2 {
    fn new(streets: &StreetNetwork, orig: &DualCarriagewayPt1) -> Option<Self> {
        // Only calculate bridges relative to side1
        let (side1_branches, bridges) = Self::find_branches_and_bridges(
            streets,
            &orig.side1,
            Self::side_to_intersections(&orig.side2),
        );
        let (side2_branches, _) = Self::find_branches_and_bridges(
            streets,
            &orig.side2,
            Self::side_to_intersections(&orig.side1),
        );

        Some(Self {
            road_name: orig.road_name.clone(),
            src_i: orig.src_i,
            dst_i: orig.dst_i,
            side1: orig.side1.clone(),
            side2: orig.side2.clone(),

            side1_branches,
            side2_branches,
            bridges,

            side1_length: orig
                .side1
                .iter()
                .map(|r| {
                    streets.roads[&r.road]
                        .untrimmed_road_geometry(streets.config.driving_side)
                        .length()
                })
                .sum(),
            side2_length: orig
                .side2
                .iter()
                .map(|r| {
                    streets.roads[&r.road]
                        .untrimmed_road_geometry(streets.config.driving_side)
                        .length()
                })
                .sum(),
        })
    }

    fn side_to_intersections(side: &Vec<RoadWithEndpoints>) -> BTreeSet<IntersectionID> {
        let mut set = BTreeSet::new();
        for r in side {
            set.insert(r.src_i);
            set.insert(r.dst_i);
        }
        set
    }

    // TODO The types are getting gross. Returns (branches, bridges).
    fn find_branches_and_bridges(
        streets: &StreetNetwork,
        side: &Vec<RoadWithEndpoints>,
        other_side_intersections: BTreeSet<IntersectionID>,
    ) -> (Vec<(RoadID, Distance)>, Vec<(RoadID, Distance)>) {
        let mut branches = Vec::new();
        let mut bridges = Vec::new();
        let mut dist = Distance::ZERO;

        for pair in side.windows(2) {
            dist += streets.roads[&pair[0].road]
                .untrimmed_road_geometry(streets.config.driving_side)
                .length();
            let i = pair[0].dst_i;
            for r in streets.intersections[&i].roads.clone() {
                if r == pair[0].road || r == pair[1].road {
                    continue;
                }
                // It's a branch or a bridge. Is the intersection it connects to part of the other
                // side or not?
                if other_side_intersections.contains(&streets.roads[&r].other_side(i)) {
                    bridges.push((r, dist));
                } else {
                    branches.push((r, dist));
                }
            }
        }

        (branches, bridges)
    }

    fn debug(&self, streets: &StreetNetwork) {
        streets.debug_intersection(self.src_i, format!("start of {}", self.road_name));
        streets.debug_intersection(self.dst_i, "end");
        for (idx, r) in self.side1.iter().enumerate() {
            if idx == 0 {
                streets.debug_road(
                    r.road,
                    format!("side1, {}, total length {}", idx, self.side1_length),
                );
            } else {
                streets.debug_road(r.road, format!("side1, {}", idx));
            }
        }
        for (idx, r) in self.side2.iter().enumerate() {
            if idx == 0 {
                streets.debug_road(
                    r.road,
                    format!("side2, {}, total length {}", idx, self.side2_length),
                );
            } else {
                streets.debug_road(r.road, format!("side2, {}", idx));
            }
        }
        for (r, dist) in &self.side1_branches {
            streets.debug_road(*r, format!("side1 branch, {dist} from src_i"));
        }
        for (r, dist) in &self.side2_branches {
            streets.debug_road(*r, format!("side2 branch, {dist} from dst_i"));
        }
        for (r, dist) in &self.bridges {
            streets.debug_road(*r, format!("bridge, {dist} from src_i"));
        }
    }
}
