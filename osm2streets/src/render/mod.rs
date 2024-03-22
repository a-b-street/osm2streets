mod intersection_markings;
mod lane_markings;
mod marking;
mod output;
mod paint;

use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use geojson::Feature;
use geom::{ArrowCap, Distance, Line, Polygon};
use serde_json::Value;

use crate::{
    DebugStreets, Direction, DrivingSide, Intersection, IntersectionID, LaneID, Movement, Road,
    RoadID, StreetNetwork,
};

/// Specifies what roads and intersections to render.
pub enum Filter {
    All,
    Filtered(BTreeSet<RoadID>, BTreeSet<IntersectionID>),
}

impl Filter {
    fn roads<'a>(&'a self, streets: &'a StreetNetwork) -> Box<dyn Iterator<Item = &Road> + 'a> {
        match self {
            Filter::All => Box::new(streets.roads.values()),
            Filter::Filtered(ref roads, _) => Box::new(roads.iter().map(|r| &streets.roads[r])),
        }
    }

    fn intersections<'a>(
        &'a self,
        streets: &'a StreetNetwork,
    ) -> Box<dyn Iterator<Item = &Intersection> + 'a> {
        match self {
            Filter::All => Box::new(streets.intersections.values()),
            Filter::Filtered(_, ref intersections) => {
                Box::new(intersections.iter().map(|i| &streets.intersections[i]))
            }
        }
    }
}

impl StreetNetwork {
    /// Generates a plain GeoJSON rendering with one polygon per road and intersection.
    pub fn to_geojson(&self, filter: &Filter) -> Result<String> {
        let mut features = Vec::new();

        // Add a polygon per road
        for road in filter.roads(self) {
            let mut f = Feature::from(
                road.center_line
                    .make_polygons(road.total_width())
                    .to_geojson(Some(&self.gps_bounds)),
            );
            f.set_property("id", road.id.0);
            f.set_property("type", "road");
            f.set_property(
                "osm_way_ids",
                Value::Array(road.osm_ids.iter().map(|id| id.0.into()).collect()),
            );
            f.set_property("src_i", road.src_i.0);
            f.set_property("dst_i", road.dst_i.0);
            f.set_property("layer", road.layer);
            features.push(f);
        }

        // Polygon per intersection
        for intersection in filter.intersections(self) {
            let mut f = Feature::from(intersection.polygon.to_geojson(Some(&self.gps_bounds)));
            f.set_property("id", intersection.id.0);
            f.set_property("type", "intersection");
            f.set_property(
                "osm_node_ids",
                Value::Array(intersection.osm_ids.iter().map(|id| id.0.into()).collect()),
            );
            f.set_property("intersection_kind", format!("{:?}", intersection.kind));
            f.set_property("control", format!("{:?}", intersection.control));
            f.set_property(
                "movements",
                Value::Array(
                    intersection
                        .movements
                        .iter()
                        .map(|(a, b)| format!("{a} -> {b}").into())
                        .collect(),
                ),
            );
            features.push(f);
        }

        // Plumb along the country code, so this value shows up in unit tests
        let mut foreign_members = serde_json::Map::new();
        foreign_members.insert(
            "country_code".to_string(),
            self.config.country_code.clone().into(),
        );
        let gj = geojson::GeoJson::from(geojson::FeatureCollection {
            bbox: None,
            features,
            foreign_members: Some(foreign_members),
        });
        let output = serde_json::to_string_pretty(&gj)?;
        Ok(output)
    }

    /// Generates a polygon per lane, with a property indicating type.
    pub fn to_lane_polygons_geojson(&self, filter: &Filter) -> Result<String> {
        let mut features = Vec::new();

        for road in filter.roads(self) {
            for (idx, (lane, pl)) in road
                .lane_specs_ltr
                .iter()
                .zip(road.get_lane_center_lines().into_iter())
                .enumerate()
            {
                let mut f = Feature::from(
                    pl.make_polygons(lane.width)
                        .to_geojson(Some(&self.gps_bounds)),
                );
                f.set_property("type", format!("{:?}", lane.lt));
                f.set_property("road", road.id.0);
                f.set_property("layer", road.layer);
                f.set_property("speed_limit", format!("{:?}", road.speed_limit));
                f.set_property("index", idx);
                f.set_property("width", lane.width.inner_meters());
                f.set_property("direction", format!("{:?}", lane.dir));
                f.set_property(
                    "allowed_turns",
                    Value::Array(
                        lane.allowed_turns
                            .iter()
                            .map(|d| d.tag_value().into())
                            .collect(),
                    ),
                );
                f.set_property(
                    "osm_way_ids",
                    Value::Array(road.osm_ids.iter().map(|id| id.0.into()).collect()),
                );
                if let Some(ref muv) = lane.lane {
                    f.set_property("muv", serde_json::to_value(muv)?);
                }
                features.push(f);
            }
        }

        serialize_features(features)
    }

    /// For an intersection, show the clockwise ordering of roads around it
    pub fn debug_clockwise_ordering_geojson(&self, filter: &Filter) -> Result<String> {
        let mut features = Vec::new();

        for intersection in filter.intersections(self) {
            for (idx, r) in intersection.roads.iter().enumerate() {
                let road = &self.roads[r];
                let pt = if road.src_i == intersection.id {
                    road.center_line.first_pt()
                } else {
                    road.center_line.last_pt()
                };
                let mut f = Feature::from(pt.to_geojson(Some(&self.gps_bounds)));
                f.set_property(
                    "label",
                    format!("{} / {}", idx + 1, intersection.roads.len()),
                );
                features.push(f);
            }
        }

        serialize_features(features)
    }

    pub fn debug_movements_from_lane_geojson(&self, id: LaneID) -> Result<String> {
        let road = &self.roads[&id.road];
        let i = if road.lane_specs_ltr[id.index].dir == Direction::Forward {
            road.dst_i
        } else {
            road.src_i
        };

        let mut features = Vec::new();
        for ((from, _), polygon) in movements_for_intersection(self, i) {
            if from == road.id {
                features.push(Feature::from(polygon.to_geojson(Some(&self.gps_bounds))));
            }
        }
        serialize_features(features)
    }
}

impl DebugStreets {
    /// None if there's nothing labelled
    pub fn to_debug_geojson(&self) -> Option<String> {
        let mut features = Vec::new();
        for (pt, label) in &self.points {
            let mut f = Feature::from(pt.to_geojson(Some(&self.streets.gps_bounds)));
            f.set_property("label", label.to_string());
            features.push(f);
        }
        for (pl, label) in &self.polylines {
            let mut f = Feature::from(pl.to_geojson(Some(&self.streets.gps_bounds)));
            f.set_property("label", label.to_string());
            features.push(f);
        }
        if features.is_empty() {
            return None;
        }
        Some(serialize_features(features).unwrap())
    }
}

fn movements_for_intersection(
    streets: &StreetNetwork,
    i: IntersectionID,
) -> Vec<(Movement, Polygon)> {
    // Each movement is represented as an arrow from the end of one road to the beginning of
    // another. To stop arrows overlapping, arrows to/from bidirectional roads are offset from
    // the center to the appropriate driving side.
    let arrow_fwd_offset_dist = if streets.config.driving_side == DrivingSide::Right {
        Distance::meters(-1.3)
    } else {
        Distance::meters(1.3)
    };

    // Find the points where the arrows should (leave, enter) the roads.
    let road_points: BTreeMap<_, _> = streets.intersections[&i]
        .roads
        .iter()
        .map(|r| {
            let road = &streets.roads[r];
            let first_road_segment = if road.src_i == i {
                road.center_line.first_line()
            } else {
                road.center_line.last_line().reversed()
            };
            // Offset the arrow start/end points if it is bidirectional.
            (
                r,
                if road.oneway_for_driving().is_some() {
                    (first_road_segment.pt1(), first_road_segment.pt1())
                } else {
                    (
                        first_road_segment
                            .shift_either_direction(arrow_fwd_offset_dist)
                            .pt1(),
                        first_road_segment
                            .shift_either_direction(-arrow_fwd_offset_dist)
                            .pt1(),
                    )
                },
            )
        })
        .collect();

    let mut result = Vec::new();
    for (a, b) in &streets.intersections[&i].movements {
        if a != b {
            if let Ok(line) = Line::new(road_points[a].0, road_points[b].1) {
                result.push((
                    (*a, *b),
                    line.to_polyline()
                        .make_arrow(Distance::meters(0.5), ArrowCap::Triangle),
                ));
            }
        }
    }
    result
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
