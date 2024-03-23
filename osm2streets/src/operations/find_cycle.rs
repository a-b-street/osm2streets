use std::collections::{hash_map::Entry, BinaryHeap, HashMap, HashSet};

use anyhow::Result;
use geojson::Feature;
use geom::Distance;

use crate::{IntersectionID, RoadID, StreetNetwork};

enum Step {
    Node(IntersectionID),
    Edge(RoadID),
}

impl StreetNetwork {
    pub fn find_cycle(&self, start: IntersectionID) -> Result<String> {
        // TODO Or even simpler... always try to go CW or CCW consistently. we dont want to
        // backtrack ever.

        let mut stack = vec![start];
        let mut steps = Vec::new();
        let mut visited = HashSet::new();

        while let Some(current) = stack.pop() {
            if visited.contains(&current) {
                if current == start && steps.len() > 1 {
                    return render(self, steps);
                }

                continue;
            }
            visited.insert(current);
            steps.push(Step::Node(current));

            for road in &self.intersections[&current].roads {
                let next_i = self.roads[road].other_side(current);
                stack.push(next_i);
            }
        }

        bail!("Something broke");
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
