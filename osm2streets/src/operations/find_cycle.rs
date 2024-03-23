use anyhow::Result;
use geojson::Feature;

use crate::{IntersectionID, RoadID, StreetNetwork};

enum Step {
    Node(IntersectionID),
    Edge(RoadID),
}

impl StreetNetwork {
    pub fn find_cycle(&self, start: IntersectionID) -> Result<String> {
        let steps_cw = self.walk_around(start, true);
        let steps_ccw = self.walk_around(start, false);
        // Use the shorter
        if steps_cw.len() < steps_ccw.len() {
            render(self, steps_cw)
        } else {
            render(self, steps_ccw)
        }
    }

    fn walk_around(&self, start: IntersectionID, clockwise: bool) -> Vec<Step> {
        let mut current_i = start;
        // Start arbitrarily
        let mut current_r = self.intersections[&start].roads[0];

        let mut steps = vec![Step::Edge(current_r)];

        while current_i != start || steps.len() < 2 {
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

fn render(streets: &StreetNetwork, steps: Vec<Step>) -> Result<String> {
    let mut features = Vec::new();

    info!("Cycle!");
    for step in steps {
        match step {
            Step::Node(i) => {
                info!("- {i}");
                let mut f = Feature::from(
                    streets.intersections[&i]
                        .polygon
                        .to_geojson(Some(&streets.gps_bounds)),
                );
                f.set_property("type", "intersection");
                features.push(f);
            }
            Step::Edge(r) => {
                info!("- {r}");
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

    let gj = geojson::GeoJson::from(geojson::FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    });
    let output = serde_json::to_string_pretty(&gj)?;
    Ok(output)
}
