use std::collections::{hash_map::Entry, BinaryHeap, HashMap};

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
        let mut backrefs: HashMap<IntersectionID, RoadID> = HashMap::new();
        // This is a max-heap, so negate all distances. Tie breaker is intersection ID, arbitrary
        // but deterministic.
        let mut queue: BinaryHeap<(Distance, IntersectionID)> = BinaryHeap::new();
        queue.push((Distance::ZERO, start));

        while !queue.is_empty() {
            let (dist_so_far, current) = queue.pop().unwrap();

            if current == start && dist_so_far != Distance::ZERO {
                let mut steps = vec![Step::Node(current)];
                let mut current = current;
                loop {
                    if current == start && steps.len() > 1 {
                        /*steps.pop();
                        steps.reverse();*/
                        return render(self, steps);
                    }
                    let road = backrefs[&current];
                    current = self.roads[&road].other_side(current);
                    steps.push(Step::Edge(road));
                    steps.push(Step::Node(current));
                }
            }

            for road in &self.intersections[&current].roads {
                let next_i = self.roads[road].other_side(current);
                if let Entry::Vacant(e) = backrefs.entry(next_i) {
                    e.insert(*road);
                    // Remember to keep things negative
                    let dist = dist_so_far - self.roads[road].center_line.length();
                    queue.push((dist, next_i));
                }
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
