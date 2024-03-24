use anyhow::Result;
use geojson::Feature;
use geom::Ring;

use crate::{IntersectionID, RoadID, StreetNetwork};

#[derive(Clone, Copy)]
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
            trace_polygon(self, steps_cw, true)
        } else {
            trace_polygon(self, steps_ccw, false)
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

fn trace_polygon(streets: &StreetNetwork, steps: Vec<Step>, clockwise: bool) -> Result<String> {
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
    let polygon = Ring::deduping_new(pts)?.into_polygon();

    let mut f = Feature::from(polygon.to_geojson(Some(&streets.gps_bounds)));
    f.set_property("type", "block");
    let gj = geojson::GeoJson::from(geojson::FeatureCollection {
        bbox: None,
        features: vec![f],
        foreign_members: None,
    });
    let output = serde_json::to_string_pretty(&gj)?;
    Ok(output)
}

fn _debug_steps(streets: &StreetNetwork, steps: Vec<Step>) -> Result<String> {
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
