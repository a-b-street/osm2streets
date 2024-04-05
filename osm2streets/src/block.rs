use std::collections::HashSet;

use abstutil::wraparound_get;
use anyhow::Result;
use geojson::Feature;
use geom::{Polygon, Ring};

use crate::{
    Direction, IntersectionID, IntersectionKind, LaneType, RoadID, RoadSideID, SideOfRoad,
    StreetNetwork,
};

/// A "tight" cycle of roads and intersections, with a polygon capturing the negative space inside.
pub struct Block {
    pub kind: BlockKind,
    /// First != last, they're not repeated
    pub boundary: Vec<RoadSideID>,
    pub polygon: Polygon,
    /// Not counting the boundary (described by steps)
    pub member_roads: HashSet<RoadID>,
    pub member_intersections: HashSet<IntersectionID>,
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
    // TODO API is getting messy
    pub fn find_block(&self, start: RoadSideID, sidewalks: bool) -> Result<Block> {
        let (boundary, start_intersection) = walk_around(self, start, sidewalks)?;
        let polygon = trace_polygon(self, &boundary, start_intersection)?;

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
            classify_block(self, &boundary)
        };

        Ok(Block {
            kind,
            boundary,
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
                let road_side = RoadSideID { road: *r, side };
                if visited_roads.contains(&road_side) {
                    continue;
                }
                if let Ok(block) = self.find_block(road_side, sidewalks) {
                    visited_roads.extend(block.boundary.clone());
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

// Also returns the first intersection.
fn walk_around(
    streets: &StreetNetwork,
    start_road_side: RoadSideID,
    sidewalks: bool,
) -> Result<(Vec<RoadSideID>, IntersectionID)> {
    let mut roads = Vec::new();

    // We may start on a loop road on the "inner" direction
    {
        let start_r = &streets.roads[&start_road_side.road];
        if start_r.src_i == start_r.dst_i {
            let i = &streets.intersections[&start_r.src_i];
            if !i.get_road_sides_sorted(streets).contains(&start_road_side) {
                bail!("Starting on inner piece of a loop road");
            }
        }
    }

    // We need to track which side of the road we're at, but also which direction we're facing
    let mut current_road_side = start_road_side;
    // TODO This is convoluted. When we started with a lane, it was that lane's dst_i
    let (orig_from_intersection, start_intersection) = {
        let dir = start_road_side.get_outermost_lane(streets).dir;
        let road = &streets.roads[&start_road_side.road];
        if dir == Direction::Forward {
            (road.src_i, road.dst_i)
        } else {
            (road.src_i, road.dst_i)
        }
    };
    let mut current_intersection = start_intersection;

    loop {
        let i = &streets.intersections[&current_intersection];
        if i.kind == IntersectionKind::MapEdge {
            bail!("hit a MapEdge at {}", i.id);
        }
        let mut sorted_roads = i.get_road_sides_sorted(streets);
        // When we're limiting to sidewalks, get rid of any roads around the intersection that
        // aren't crossings or sidewalks
        if sidewalks {
            sorted_roads.retain(|r| streets.roads[&r.road].is_footway());
        }

        let idx = sorted_roads
            .iter()
            .position(|x| *x == current_road_side)
            .unwrap() as isize;
        // Do we go clockwise or counter-clockwise around the intersection? Well, unless we're
        // at a dead-end, we want to avoid the other side of the same road.
        let mut next = *wraparound_get(&sorted_roads, idx + 1);
        assert_ne!(next, current_road_side);
        if next.road == current_road_side.road {
            next = *wraparound_get(&sorted_roads, idx - 1);
            assert_ne!(next, current_road_side);
            if next.road == current_road_side.road {
                if sorted_roads.len() != 2 {
                    bail!("Looped back on the same road, but not at a dead-end");
                }
            }
        }
        roads.push(current_road_side);
        current_road_side = next;
        current_intersection =
            streets.roads[&current_road_side.road].other_side(current_intersection);

        if current_road_side == start_road_side {
            roads.push(start_road_side);
            break;
        }
    }
    assert_eq!(roads[0], *roads.last().unwrap());
    roads.pop();
    Ok((roads, orig_from_intersection))
}

fn trace_polygon(
    streets: &StreetNetwork,
    road_sides: &Vec<RoadSideID>,
    start_intersection: IntersectionID,
) -> Result<Polygon> {
    let mut pts = Vec::new();

    let mut last_i = start_intersection;
    for road_side in road_sides {
        //info!("{:?}", last_i);
        //info!("{:?}", road_side);
        let road = &streets.roads[&road_side.road];

        // First handle the side
        let shift_dir = if road_side.side == SideOfRoad::Left {
            -1.0
        } else {
            1.0
        };
        let pl = road
            .center_line
            .shift_either_direction(shift_dir * road.half_width())?;

        // Then figure out the direction
        // TODO Track this along the way instead, if possible
        if road.src_i == last_i {
            pts.extend(pl.into_points());
            last_i = road.dst_i;
        } else {
            pts.extend(pl.reversed().into_points());
            last_i = road.src_i;
        }
    }

    pts.push(pts[0]);
    Ok(Ring::deduping_new(pts)?.into_polygon())
}

fn classify_block(streets: &StreetNetwork, boundary: &Vec<RoadSideID>) -> BlockKind {
    let mut has_road = false;
    let mut has_cycle_lane = false;
    let mut has_sidewalk = false;

    for road_side in boundary {
        let lt = road_side.get_outermost_lane(streets).lt;
        if lt == LaneType::Driving || lt == LaneType::Bus {
            has_road = true;
        } else if lt == LaneType::Biking {
            has_cycle_lane = true;
        } else if lt == LaneType::Sidewalk {
            has_sidewalk = true;
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
    // See how many "major" named roads are inside
    if true {
        let mut major_road_names = HashSet::new();

        for r in member_roads {
            let road = &streets.roads[r];
            if let Some(name) = road.name.clone() {
                if road.highway_type != "service" {
                    major_road_names.insert(name);
                }
            }
        }

        return if major_road_names.len() == 0 {
            BlockKind::LandUseBlock
        } else if major_road_names.len() == 1 {
            BlockKind::RoadBundle
        } else {
            BlockKind::IntersectionBundle
        };
    }

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

    // See how "square" the block polygon is.
    // TODO Need to axis-align this first for it to have hope
    if false {
        let bounds = polygon.get_bounds();
        let ratio = bounds.width() / bounds.height();
        if ratio > 0.5 && ratio < 2.0 {
            return BlockKind::IntersectionBundle;
        } else {
            return BlockKind::RoadBundle;
        }
    }

    BlockKind::Unknown
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
