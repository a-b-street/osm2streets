use std::collections::HashSet;

use anyhow::Result;
use geojson::Feature;
use geom::{Polygon, Ring};

use crate::{IntersectionID, RoadID, StreetNetwork};

/// A "tight" cycle of roads and intersections, with a polygon capturing the negative space inside.
pub struct Block {
    pub steps: Vec<Step>,
    pub polygon: Polygon,
}
#[derive(Clone, Copy)]
pub enum Step {
    Node(IntersectionID),
    Edge(RoadID),
}

impl StreetNetwork {
    pub fn find_block(&self, start: IntersectionID) -> Result<Block> {
        let steps_cw = self.walk_around(start, true);
        let steps_ccw = self.walk_around(start, false);
        // Use the shorter one
        let (steps, clockwise) = if steps_cw.len() < steps_ccw.len() && !steps_cw.is_empty() {
            (steps_cw, true)
        } else {
            (steps_ccw, false)
        };
        if steps.is_empty() {
            bail!("Found a dead-end");
        }

        // Trace the polygon
        let shift_dir = if clockwise { -1.0 } else { 1.0 };
        let mut pts = Vec::new();

        // steps will begin and end with an edge
        for pair in steps.windows(2) {
            match (pair[0], pair[1]) {
                (Step::Edge(r), Step::Node(i)) => {
                    let road = &self.roads[&r];
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
        let polygon = Ring::deduping_new(pts)?.into_polygon();

        Ok(Block { steps, polygon })
    }

    pub fn find_all_blocks(&self) -> Result<String> {
        // TODO We should track by side of the road (but then need a way to start there)
        let mut visited_intersections = HashSet::new();
        let mut blocks = Vec::new();

        for i in self.intersections.keys() {
            if visited_intersections.contains(i) {
                continue;
            }
            if let Ok(block) = self.find_block(*i) {
                for step in &block.steps {
                    if let Step::Node(i) = step {
                        visited_intersections.insert(*i);
                    }
                }
                blocks.push(block);
            }
        }

        let mut features = Vec::new();
        for block in blocks {
            let mut f = Feature::from(block.polygon.to_geojson(Some(&self.gps_bounds)));
            f.set_property("type", "block");
            features.push(f);
        }
        serialize_features(features)
    }

    // Returns an empty Vec for failures (hitting a dead-end)
    fn walk_around(&self, start: IntersectionID, clockwise: bool) -> Vec<Step> {
        let mut current_i = start;
        // Start arbitrarily
        let mut current_r = self.intersections[&start].roads[0];

        let mut steps = vec![Step::Edge(current_r)];

        while current_i != start || steps.len() < 2 {
            // Fail for dead-ends (for now, to avoid tracing around the entire clipped map)
            if self.intersections[&current_i].roads.len() == 1 {
                return Vec::new();
            }

            let next_i = &self.intersections[&self.roads[&current_r].other_side(current_i)];
            let idx = next_i.roads.iter().position(|x| *x == current_r).unwrap();
            let next_idx = if clockwise {
                if idx == next_i.roads.len() - 1 {
                    0
                } else {
                    idx + 1
                }
            } else {
                if idx == 0 {
                    next_i.roads.len() - 1
                } else {
                    idx - 1
                }
            };
            let next_r = next_i.roads[next_idx];
            steps.push(Step::Node(next_i.id));
            steps.push(Step::Edge(next_r));
            current_i = next_i.id;
            current_r = next_r;
        }

        steps
    }
}

impl Block {
    pub fn render_polygon(&self, streets: &StreetNetwork) -> Result<String> {
        let mut f = Feature::from(self.polygon.to_geojson(Some(&streets.gps_bounds)));
        f.set_property("type", "block");
        serialize_features(vec![f])
    }

    pub fn render_debug(&self, streets: &StreetNetwork) -> Result<String> {
        let mut features = Vec::new();

        for step in &self.steps {
            match step {
                Step::Node(i) => {
                    let mut f = Feature::from(
                        streets.intersections[&i]
                            .polygon
                            .to_geojson(Some(&streets.gps_bounds)),
                    );
                    f.set_property("type", "intersection");
                    features.push(f);
                }
                Step::Edge(r) => {
                    let road = &streets.roads[&r];
                    let mut f = Feature::from(
                        road.center_line
                            .make_polygons(road.total_width())
                            .to_geojson(Some(&streets.gps_bounds)),
                    );
                    f.set_property("type", "road");
                    features.push(f);
                }
            }
        }

        serialize_features(features)
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
