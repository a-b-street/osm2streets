use std::collections::HashSet;

use anyhow::Result;
use geojson::Feature;
use geom::{Polygon, Ring};

use crate::{
    Intersection, IntersectionID, LaneType, RoadID, RoadSideID, SideOfRoad, StreetNetwork,
};

/// A "tight" cycle of roads and intersections, with a polygon capturing the negative space inside.
pub struct Block {
    pub kind: BlockKind,
    pub steps: Vec<Step>,
    pub polygon: Polygon,
    /// Not counting the boundary (described by steps)
    pub member_roads: HashSet<RoadID>,
    pub member_intersections: HashSet<IntersectionID>,
}

#[derive(Clone, Copy)]
pub enum Step {
    Node(IntersectionID),
    Edge(RoadID),
}

#[derive(Debug)]
pub enum BlockKind {
    /// A "city" block; the space in between sidewals, probably just containing buildings and not
    /// roads
    // TODO Or just "not related to roads". Could be parks/water
    LandUseBlock,
    /// The space between a road and sidewalk. It might be a wide sidewalk or contain grass.
    RoadAndSidewalk,
    /// The space between a road and cycle lane. It should contain some kind of separation.
    RoadAndCycleLane,
    /// The space between a cycle lane and sidewalk. It should contain some kind of separation --
    /// at least a curb.
    CycleLaneAndSidewalk,
    /// The space between one-way roads. Probably has some kind of physical barrier, not just
    /// markings.
    DualCarriageway,
    /// A segment of road and all sidepaths and internal connections
    RoadBundle,
    /// A possibly complex junction; everything in between all the crossings
    IntersectionBundle,
    Unknown,
}

impl StreetNetwork {
    // Start at road's src_i
    // TODO API is getting messy
    pub fn find_block(&self, start: RoadID, left: bool, sidewalks: bool) -> Result<Block> {
        let clockwise = left;
        let steps = walk_around(self, start, clockwise, sidewalks)?;
        let polygon = trace_polygon(self, &steps, clockwise)?;

        let mut member_roads = HashSet::new();
        let mut member_intersections = HashSet::new();
        if sidewalks {
            // Look for roads inside the polygon geometrically
            // TODO Slow; could cache an rtree
            // TODO Incorrect near bridges/tunnels
            for road in self.roads.values() {
                if polygon.contains_pt(road.center_line.middle()) {
                    member_roads.insert(road.id);
                }
            }
            for intersection in self.intersections.values() {
                if polygon.contains_pt(intersection.polygon.center()) {
                    member_intersections.insert(intersection.id);
                }
            }
        }
        let kind = if sidewalks {
            classify_bundle(self, &polygon, &member_roads, &member_intersections)
        } else {
            classify_block(self, &steps)
        };

        Ok(Block {
            kind,
            steps,
            polygon,
            member_roads,
            member_intersections,
        })
    }

    // TODO Messy API again
    pub fn find_all_blocks(&self, sidewalks: bool) -> Result<String> {
        let mut visited_roads: HashSet<RoadSideID> = HashSet::new();
        let mut blocks = Vec::new();

        for r in self.roads.keys() {
            if sidewalks && !self.roads[r].is_footway() {
                continue;
            }

            for side in [SideOfRoad::Left, SideOfRoad::Right] {
                if visited_roads.contains(&RoadSideID { road: *r, side }) {
                    continue;
                }
                if let Ok(block) = self.find_block(*r, side == SideOfRoad::Left, sidewalks) {
                    // TODO Put more info in Step to avoid duplicating logic with trace_polygon
                    for pair in block.steps.windows(2) {
                        match (pair[0], pair[1]) {
                            (Step::Edge(r), Step::Node(i)) => {
                                let road = &self.roads[&r];
                                if road.dst_i == i {
                                    visited_roads.insert(RoadSideID { road: r, side });
                                } else {
                                    visited_roads.insert(RoadSideID {
                                        road: r,
                                        side: side.opposite(),
                                    });
                                }
                            }
                            // Skip... unless for the last case?
                            (Step::Node(_), Step::Edge(_)) => {}
                            _ => unreachable!(),
                        }
                    }
                    blocks.push(block);
                }
            }
        }

        let mut features = Vec::new();
        for block in blocks {
            let mut f = Feature::from(block.polygon.to_geojson(Some(&self.gps_bounds)));
            f.set_property("type", "block");
            f.set_property("kind", format!("{:?}", block.kind));
            features.push(f);
        }
        serialize_features(features)
    }
}

impl Block {
    pub fn render_polygon(&self, streets: &StreetNetwork) -> Result<String> {
        let mut features = Vec::new();

        let mut f = Feature::from(self.polygon.to_geojson(Some(&streets.gps_bounds)));
        f.set_property("type", "block");
        f.set_property("kind", format!("{:?}", self.kind));
        features.push(f);

        // Debugging
        if false {
            for r in &self.member_roads {
                let road = &streets.roads[&r];
                let mut f = Feature::from(
                    road.center_line
                        .make_polygons(road.total_width())
                        .to_geojson(Some(&streets.gps_bounds)),
                );
                f.set_property("type", "member-road");
                features.push(f);
            }
            for i in &self.member_intersections {
                let mut f = Feature::from(
                    streets.intersections[i]
                        .polygon
                        .to_geojson(Some(&streets.gps_bounds)),
                );
                f.set_property("type", "member-intersection");
                features.push(f);
            }
        }

        serialize_features(features)
    }
}

fn walk_around(
    streets: &StreetNetwork,
    start_road: RoadID,
    clockwise: bool,
    sidewalks: bool,
) -> Result<Vec<Step>> {
    let start_i = streets.roads[&start_road].src_i;

    let mut current_i = start_i;
    let mut current_r = start_road;

    let mut steps = vec![Step::Edge(current_r)];

    while current_i != start_i || steps.len() < 2 {
        // Fail for dead-ends (for now, to avoid tracing around the entire clipped map)
        if filter_roads(streets, sidewalks, &streets.intersections[&current_i]).len() == 1 {
            bail!("Found a dead-end at {current_i}");
        }

        let next_i = &streets.intersections[&streets.roads[&current_r].other_side(current_i)];
        let clockwise_roads = filter_roads(streets, sidewalks, next_i);
        let idx = clockwise_roads
            .iter()
            .position(|x| *x == current_r)
            .unwrap();
        let next_idx = if clockwise {
            if idx == clockwise_roads.len() - 1 {
                0
            } else {
                idx + 1
            }
        } else {
            if idx == 0 {
                clockwise_roads.len() - 1
            } else {
                idx - 1
            }
        };
        let next_r = clockwise_roads[next_idx];
        steps.push(Step::Node(next_i.id));
        steps.push(Step::Edge(next_r));
        current_i = next_i.id;
        current_r = next_r;
    }

    Ok(steps)
}

// When we're limiting to sidewalks, get rid of any roads around the intersection that aren't
// crossings or sidewalks
fn filter_roads(
    streets: &StreetNetwork,
    sidewalks: bool,
    intersection: &Intersection,
) -> Vec<RoadID> {
    let mut roads = intersection.roads.clone();
    if !sidewalks {
        return roads;
    }
    roads.retain(|r| streets.roads[r].is_footway());
    roads
}

fn trace_polygon(streets: &StreetNetwork, steps: &Vec<Step>, clockwise: bool) -> Result<Polygon> {
    let shift_dir = if clockwise { -1.0 } else { 1.0 };
    let mut pts = Vec::new();

    // steps will begin and end with an edge
    for pair in steps.windows(2) {
        match (pair[0], pair[1]) {
            (Step::Edge(r), Step::Node(i)) => {
                let road = &streets.roads[&r];
                if road.dst_i == i {
                    pts.extend(
                        road.center_line
                            .shift_either_direction(shift_dir * road.half_width())?
                            .into_points(),
                    );
                } else {
                    pts.extend(
                        road.center_line
                            .reversed()
                            .shift_either_direction(shift_dir * road.half_width())?
                            .into_points(),
                    );
                }
            }
            // Skip... unless for the last case?
            (Step::Node(_), Step::Edge(_)) => {}
            _ => unreachable!(),
        }
    }

    pts.push(pts[0]);
    Ok(Ring::deduping_new(pts)?.into_polygon())
}

fn classify_block(streets: &StreetNetwork, steps: &Vec<Step>) -> BlockKind {
    let mut has_road = false;
    let mut has_cycle_lane = false;
    let mut has_sidewalk = false;

    for step in steps {
        if let Step::Edge(r) = step {
            let road = &streets.roads[r];
            if road.is_driveable() {
                // TODO Or bus lanes?
                has_road = true;
            } else if road.lane_specs_ltr.len() == 1
                && road.lane_specs_ltr[0].lt == LaneType::Biking
            {
                has_cycle_lane = true;
            } else if road.lane_specs_ltr.len() == 1
                && road.lane_specs_ltr[0].lt == LaneType::Sidewalk
            {
                has_sidewalk = true;
            }
        }
    }

    if has_road && has_sidewalk {
        // TODO But ignore driveways and service roads?
        return BlockKind::RoadAndSidewalk;
    }
    if has_road && has_cycle_lane {
        return BlockKind::RoadAndCycleLane;
    }
    if has_road {
        // TODO Insist on one-ways pointing the opposite direction? What about different types of
        // small connector roads?
        return BlockKind::DualCarriageway;
    }
    // TODO This one is usually missed, because of a small piece of road crossing both
    if !has_road && has_cycle_lane && has_sidewalk {
        return BlockKind::CycleLaneAndSidewalk;
    }
    if !has_road && !has_cycle_lane && has_sidewalk {
        return BlockKind::LandUseBlock;
    }

    BlockKind::Unknown
}

fn classify_bundle(
    streets: &StreetNetwork,
    polygon: &Polygon,
    member_roads: &HashSet<RoadID>,
    member_intersections: &HashSet<IntersectionID>,
) -> BlockKind {
    if member_intersections.is_empty() && member_roads.is_empty() {
        return BlockKind::LandUseBlock;
    }

    // A bad heuristic: sum the intersection and road polygon area, and see which is greater
    if false {
        let mut road_area = 0.0;
        for r in member_roads {
            let road = &streets.roads[r];
            road_area += road.center_line.make_polygons(road.total_width()).area();
        }

        let mut intersection_area = 0.0;
        for i in member_intersections {
            intersection_area += streets.intersections[i].polygon.area();
        }

        if road_area > intersection_area {
            return BlockKind::RoadBundle;
        } else {
            return BlockKind::IntersectionBundle;
        }
    }

    // TODO Check member road names and ignore service roads?

    // See how "square" the block polygon is. Even if it's not axis-aligned, this sometimes works
    let bounds = polygon.get_bounds();
    let ratio = bounds.width() / bounds.height();
    if ratio > 0.5 && ratio < 2.0 {
        return BlockKind::IntersectionBundle;
    } else {
        return BlockKind::RoadBundle;
    }
}

fn serialize_features(features: Vec<Feature>) -> Result<String> {
    let gj = geojson::GeoJson::from(geojson::FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    });
    let output = serde_json::to_string_pretty(&gj)?;
    Ok(output)
}
