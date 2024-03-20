use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::Write;
use std::path::Path;

use anyhow::Result;
use geojson::Feature;
use geom::{ArrowCap, Distance, Line, PolyLine, Polygon, Ring};
use serde_json::Value;

use crate::road::RoadEdge;
use crate::{
    BufferType, DebugStreets, Direction, DrivingSide, Intersection, IntersectionID, LaneID,
    LaneSpec, LaneType, Movement, Road, RoadID, StreetNetwork,
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
    /// Saves the plain GeoJSON rendering to a file.
    pub fn save_to_geojson(&self, output_path: String) -> Result<()> {
        let json_output = self.to_geojson(&Filter::All)?;
        std::fs::create_dir_all(Path::new(&output_path).parent().unwrap())?;
        let mut file = File::create(output_path)?;
        file.write_all(json_output.as_bytes())?;
        Ok(())
    }

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

    /// Generate polygons representing lane markings, with a property indicating type.
    pub fn to_lane_markings_geojson(&self, filter: &Filter) -> Result<String> {
        let gps_bounds = Some(&self.gps_bounds);
        let mut features = Vec::new();

        for road in filter.roads(self) {
            // Always oriented in the direction of the road
            let mut lane_centers = road.get_lane_center_lines();

            for (idx, pair) in road.lane_specs_ltr.windows(2).enumerate() {
                // Generate a "center line" between lanes of different directions
                if pair[0].dir != pair[1].dir {
                    let between = lane_centers[idx].shift_right(pair[0].width / 2.0)?;
                    // TODO Ideally we would return a full LineString, and the caller would choose
                    // how to style these as thickened dashed lines.
                    // TODO We could also at least return a MultiPolygon here
                    for poly in between.dashed_lines(
                        Distance::meters(0.25),
                        Distance::meters(2.0),
                        Distance::meters(1.0),
                    ) {
                        let mut f = Feature::from(poly.to_geojson(gps_bounds));
                        f.set_property("type", "center line");
                        f.set_property("layer", road.layer);
                        features.push(f);
                    }
                    continue;
                }

                // Generate a "lane separator" between driving lanes only
                if pair[0].lt == LaneType::Driving && pair[1].lt == LaneType::Driving {
                    let between = lane_centers[idx].shift_right(pair[0].width / 2.0)?;
                    for poly in between.dashed_lines(
                        Distance::meters(0.25),
                        Distance::meters(1.0),
                        Distance::meters(1.5),
                    ) {
                        let mut f = Feature::from(poly.to_geojson(gps_bounds));
                        f.set_property("type", "lane separator");
                        f.set_property("layer", road.layer);
                        features.push(f);
                    }
                }
            }

            // Stop line distances are relative to the direction of the road, not the lane!
            for (lane, center) in road.lane_specs_ltr.iter().zip(lane_centers.iter()) {
                for (polygon, kind) in draw_stop_lines(lane, center, road) {
                    let mut f = Feature::from(polygon.to_geojson(gps_bounds));
                    f.set_property("type", kind);
                    features.push(f);
                }
            }

            // Below renderings need lane centers to point in the direction of the lane
            for (lane, center) in road.lane_specs_ltr.iter().zip(lane_centers.iter_mut()) {
                if lane.dir == Direction::Backward {
                    *center = center.reversed();
                }
            }

            // Draw arrows along any travel lane
            for (lane, center) in road.lane_specs_ltr.iter().zip(lane_centers.iter()) {
                if !lane.lt.is_for_moving_vehicles() {
                    continue;
                }

                let step_size = Distance::meters(20.0);
                let buffer_ends = Distance::meters(5.0);
                let arrow_len = Distance::meters(1.75);
                let thickness = Distance::meters(0.25);
                for (pt, angle) in center.step_along(step_size, buffer_ends) {
                    let arrow = PolyLine::must_new(vec![
                        pt.project_away(arrow_len / 2.0, angle.opposite()),
                        pt.project_away(arrow_len / 2.0, angle),
                    ])
                    .make_arrow(thickness * 2.0, ArrowCap::Triangle)
                    .get_outer_ring()
                    .to_outline(thickness / 2.0);
                    let mut f = Feature::from(arrow.to_geojson(gps_bounds));
                    f.set_property("type", "lane arrow");
                    f.set_property("layer", road.layer);
                    features.push(f);
                }
            }

            // Add stripes to show most buffers.
            for (lane, center) in road.lane_specs_ltr.iter().zip(lane_centers.iter()) {
                // TODO Revisit rendering for different buffer types
                if !matches!(lane.lt, LaneType::Buffer(_)) {
                    continue;
                }
                if lane.lt == LaneType::Buffer(BufferType::Curb) {
                    continue;
                }

                // Mark the sides of the lane clearly
                let thickness = Distance::meters(0.25);
                for dir in [-1.0, 1.0] {
                    let mut f = Feature::from(
                        center
                            .shift_either_direction(dir * (lane.width - thickness) / 2.0)
                            .unwrap()
                            .make_polygons(thickness)
                            .to_geojson(gps_bounds),
                    );
                    f.set_property("type", "buffer edge");
                    f.set_property("layer", road.layer);
                    features.push(f);
                }

                // Diagonal stripes along the lane
                let step_size = Distance::meters(3.0);
                let buffer_ends = Distance::meters(5.0);
                for (center, angle) in center.step_along(step_size, buffer_ends) {
                    // Extend the stripes into the side lines
                    let left =
                        center.project_away(lane.width / 2.0 + thickness, angle.rotate_degs(45.0));
                    let right = center.project_away(
                        lane.width / 2.0 + thickness,
                        angle.rotate_degs(45.0).opposite(),
                    );
                    let mut f = Feature::from(
                        Line::must_new(left, right)
                            .make_polygons(thickness)
                            .to_geojson(gps_bounds),
                    );
                    f.set_property("type", "buffer stripe");
                    f.set_property("layer", road.layer);
                    features.push(f);
                }
            }

            for (lane, center) in road.lane_specs_ltr.iter().zip(lane_centers.iter()) {
                if lane.lt != LaneType::Parking {
                    continue;
                }
                for polygon in draw_parking_lines(lane, center, self) {
                    let mut f = Feature::from(polygon.to_geojson(gps_bounds));
                    f.set_property("type", "parking hatch");
                    features.push(f);
                }
            }

            for (lane, center) in road.lane_specs_ltr.iter().zip(lane_centers.iter()) {
                if lane.lt != LaneType::Sidewalk {
                    continue;
                }
                for polygon in draw_sidewalk_lines(lane, center) {
                    let mut f = Feature::from(polygon.to_geojson(gps_bounds));
                    f.set_property("type", "sidewalk line");
                    features.push(f);
                }
            }

            for (lane, center) in road.lane_specs_ltr.iter().zip(lane_centers.iter()) {
                if lane.lt != LaneType::SharedUse && lane.lt != LaneType::Footway {
                    continue;
                }
                for polygon in draw_path_outlines(lane, center) {
                    let mut f = Feature::from(polygon.to_geojson(gps_bounds));
                    f.set_property("type", "path outline");
                    features.push(f);
                }
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

    pub fn to_intersection_markings_geojson(&self, filter: &Filter) -> Result<String> {
        let mut features = Vec::new();
        for intersection in filter.intersections(self) {
            for polygon in make_sidewalk_corners(self, intersection) {
                let mut f = Feature::from(polygon.to_geojson(Some(&self.gps_bounds)));
                f.set_property("type", "sidewalk corner");
                features.push(f);
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

// TODO Where should this live?
/// For an intersection, show all corners where sidewalks meet.
fn make_sidewalk_corners(streets: &StreetNetwork, intersection: &Intersection) -> Vec<Polygon> {
    // Look at every adjacent pair of edges
    let mut edges = RoadEdge::calculate(
        streets.roads_per_intersection(intersection.id),
        intersection.id,
    );
    edges.push(edges[0].clone());
    let mut results = Vec::new();
    for pair in edges.windows(2) {
        let one = &pair[0];
        let two = &pair[1];

        // Only want corners between two roads
        if one.road == two.road {
            continue;
        }

        // Only want two sidewalks or shoulders
        if !(one.lane.lt == LaneType::Sidewalk || one.lane.lt == LaneType::Shoulder) {
            continue;
        }
        if !(two.lane.lt == LaneType::Sidewalk || two.lane.lt == LaneType::Shoulder) {
            continue;
        }

        // Only want roads with more than just a sidewalk/shoulder lane
        if streets.roads[&one.road].lane_specs_ltr.len() == 1
            || streets.roads[&two.road].lane_specs_ltr.len() == 1
        {
            continue;
        }

        // These points should be right on the intersection polygon
        let outer_corner1 = one.pl.last_pt();
        let outer_corner2 = two.pl.last_pt();

        let mut pts_along_intersection = Vec::new();
        if outer_corner1 == outer_corner2 {
            pts_along_intersection.push(outer_corner1);
        } else {
            if let Some(pl) = intersection
                .polygon
                .get_outer_ring()
                .get_shorter_slice_btwn(outer_corner1, outer_corner2)
            {
                pts_along_intersection = pl.into_points();
            } else {
                // Something went wrong; bail out
                continue;
            }
        }

        // Now find the inner sides of each sidewalk.
        let inner_pl1 = one.pl.must_shift_right(one.lane.width);
        let inner_pl2 = two.pl.must_shift_left(two.lane.width);

        // Imagine the inner lines extended into the intersection. If the point where they meet is
        // still inside the intersection, let's use it.
        let mut meet_pt = None;
        if let Some(pt) = inner_pl1
            .last_line()
            .infinite()
            .intersection(&inner_pl2.last_line().infinite())
        {
            if intersection.polygon.contains_pt(pt) {
                meet_pt = Some(pt);
            }
        }

        // Assemble everything into a ring.

        // This points from one to two, tracing along the intersection.
        let mut pts = pts_along_intersection;
        // Now add two's inner corner
        pts.push(inner_pl2.last_pt());
        // If we have a point where the two infinite lines meet, use it
        pts.extend(meet_pt);
        // one's inner corner
        pts.push(inner_pl1.last_pt());
        // Close the ring
        pts.push(pts[0]);

        if let Ok(ring) = Ring::deduping_new(pts) {
            results.push(ring.into_polygon());
        }
    }
    results
}

fn draw_stop_lines(
    lane: &LaneSpec,
    center: &PolyLine,
    road: &Road,
) -> Vec<(Polygon, &'static str)> {
    let mut results = Vec::new();

    if !matches!(
        lane.lt,
        LaneType::Driving | LaneType::Bus | LaneType::Biking
    ) {
        return results;
    }
    let thickness = Distance::meters(0.5);

    let stop_line = if lane.dir == Direction::Forward {
        &road.stop_line_end
    } else {
        &road.stop_line_start
    };

    // The vehicle line
    if let Some(dist) = stop_line.vehicle_distance {
        if let Ok((pt, angle)) = center.dist_along(dist) {
            results.push((
                Line::must_new(
                    pt.project_away(lane.width / 2.0, angle.rotate_degs(90.0)),
                    pt.project_away(lane.width / 2.0, angle.rotate_degs(-90.0)),
                )
                .make_polygons(thickness),
                "vehicle stop line",
            ));
        }
    }

    if let Some(dist) = stop_line.bike_distance {
        if let Ok((pt, angle)) = center.dist_along(dist) {
            results.push((
                Line::must_new(
                    pt.project_away(lane.width / 2.0, angle.rotate_degs(90.0)),
                    pt.project_away(lane.width / 2.0, angle.rotate_degs(-90.0)),
                )
                .make_polygons(thickness),
                "bike stop line",
            ));
        }
    }

    // TODO Change the rendering based on interruption too

    results
}

fn draw_parking_lines(lane: &LaneSpec, center: &PolyLine, streets: &StreetNetwork) -> Vec<Polygon> {
    let mut result = Vec::new();

    // No spots next to intersections
    let spots = (center.length() / streets.config.street_parking_spot_length).floor() - 2.0;
    let num_spots = if spots >= 1.0 {
        spots as usize
    } else {
        return result;
    };

    let leg_length = Distance::meters(1.0);
    for idx in 0..=num_spots {
        let (pt, lane_angle) =
            center.must_dist_along(streets.config.street_parking_spot_length * (1.0 + idx as f64));
        let perp_angle = if streets.config.driving_side == DrivingSide::Right {
            lane_angle.rotate_degs(270.0)
        } else {
            lane_angle.rotate_degs(90.0)
        };
        // Find the outside of the lane. Actually, shift inside a little bit, since the line
        // will have thickness, but shouldn't really intersect the adjacent line
        // when drawn.
        let t_pt = pt.project_away(lane.width * 0.4, perp_angle);
        // The perp leg
        let p1 = t_pt.project_away(leg_length, perp_angle.opposite());
        result.push(Line::must_new(t_pt, p1).make_polygons(Distance::meters(0.25)));
        // Upper leg
        let p2 = t_pt.project_away(leg_length, lane_angle);
        result.push(Line::must_new(t_pt, p2).make_polygons(Distance::meters(0.25)));
        // Lower leg
        let p3 = t_pt.project_away(leg_length, lane_angle.opposite());
        result.push(Line::must_new(t_pt, p3).make_polygons(Distance::meters(0.25)));
    }

    result
}

fn draw_sidewalk_lines(lane: &LaneSpec, center: &PolyLine) -> Vec<Polygon> {
    center
        .step_along(lane.width, lane.width)
        .into_iter()
        .map(|(pt, angle)| {
            // Project away an arbitrary amount
            let pt2 = pt.project_away(Distance::meters(1.0), angle);
            perp_line(Line::must_new(pt, pt2), lane.width).make_polygons(Distance::meters(0.25))
        })
        .collect()
}

fn draw_path_outlines(lane: &LaneSpec, center: &PolyLine) -> Vec<Polygon> {
    let mut result = Vec::new();
    // Dashed lines on both sides
    for dir in [-1.0, 1.0] {
        let pl = center
            .shift_either_direction(dir * lane.width / 2.0)
            .unwrap();
        result.extend(pl.exact_dashed_polygons(
            Distance::meters(0.25),
            Distance::meters(1.0),
            Distance::meters(1.5),
        ));
    }
    result
}

// this always does it at pt1
fn perp_line(l: Line, length: Distance) -> Line {
    let pt1 = l.shift_right(length / 2.0).pt1();
    let pt2 = l.shift_left(length / 2.0).pt1();
    Line::must_new(pt1, pt2)
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
