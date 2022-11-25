use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use anyhow::Result;
use geom::{ArrowCap, Distance, Line, PolyLine, Polygon, Ring};

use crate::{
    DebugStreets, Direction, DrivingSide, Intersection, LaneSpec, LaneType, RoadID, StreetNetwork,
};

impl StreetNetwork {
    /// Saves the plain GeoJSON rendering to a file.
    pub fn save_to_geojson(&self, output_path: String) -> Result<()> {
        let json_output = self.to_geojson()?;
        std::fs::create_dir_all(Path::new(&output_path).parent().unwrap())?;
        let mut file = File::create(output_path)?;
        file.write_all(json_output.as_bytes())?;
        Ok(())
    }

    /// Generates a plain GeoJSON rendering with one polygon per road and intersection.
    pub fn to_geojson(&self) -> Result<String> {
        let mut pairs = Vec::new();

        // Add a polygon per road
        for road in self.roads.values() {
            pairs.push((
                road.trimmed_center_line
                    .make_polygons(road.total_width())
                    .to_geojson(Some(&self.gps_bounds)),
                make_props(&[
                    ("type", "road".into()),
                    (
                        "osm_way_ids",
                        serde_json::Value::Array(
                            road.osm_ids
                                .iter()
                                .map(|id| id.osm_way_id.0.into())
                                .collect(),
                        ),
                    ),
                    ("src_i", road.src_i.0.into()),
                    ("dst_i", road.dst_i.0.into()),
                ]),
            ));
        }

        // Polygon per intersection
        for intersection in self.intersections.values() {
            pairs.push((
                intersection.polygon.to_geojson(Some(&self.gps_bounds)),
                make_props(&[
                    ("type", "intersection".into()),
                    (
                        "osm_node_ids",
                        serde_json::Value::Array(
                            intersection.osm_ids.iter().map(|id| id.0.into()).collect(),
                        ),
                    ),
                    (
                        "intersection_kind",
                        format!("{:?}", intersection.kind).into(),
                    ),
                    ("control", format!("{:?}", intersection.control).into()),
                    (
                        "movements",
                        serde_json::Value::Array(
                            intersection
                                .movements
                                .iter()
                                .map(|(a, b)| format!("{a} -> {b}").into())
                                .collect(),
                        ),
                    ),
                ]),
            ));
        }

        let obj = geom::geometries_with_properties_to_geojson(pairs);
        let output = serde_json::to_string_pretty(&obj)?;
        Ok(output)
    }

    /// Generates a polygon per lane, with a property indicating type.
    pub fn to_lane_polygons_geojson(&self) -> Result<String> {
        let mut pairs = Vec::new();

        for road in self.roads.values() {
            for (lane, pl) in road
                .lane_specs_ltr
                .iter()
                .zip(road.get_lane_center_lines().into_iter())
            {
                pairs.push((
                    pl.make_polygons(lane.width)
                        .to_geojson(Some(&self.gps_bounds)),
                    make_props(&[
                        ("type", format!("{:?}", lane.lt).into()),
                        ("width", lane.width.inner_meters().into()),
                        ("direction", format!("{:?}", lane.dir).into()),
                        (
                            "turn_restrictions",
                            serde_json::Value::Array(
                                lane.turn_restrictions
                                    .iter()
                                    .cloned()
                                    .map(|x| x.into())
                                    .collect(),
                            ),
                        ),
                        (
                            "osm_way_ids",
                            serde_json::Value::Array(
                                road.osm_ids
                                    .iter()
                                    .map(|id| id.osm_way_id.0.into())
                                    .collect(),
                            ),
                        ),
                    ]),
                ));
            }
        }

        let obj = geom::geometries_with_properties_to_geojson(pairs);
        let output = serde_json::to_string_pretty(&obj)?;
        Ok(output)
    }

    /// Generate polygons representing lane markings, with a property indicating type.
    pub fn to_lane_markings_geojson(&self) -> Result<String> {
        let gps_bounds = Some(&self.gps_bounds);

        let mut pairs = Vec::new();

        for road in self.roads.values() {
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
                        pairs.push((
                            poly.to_geojson(gps_bounds),
                            make_props(&[("type", "center line".into())]),
                        ));
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
                        pairs.push((
                            poly.to_geojson(gps_bounds),
                            make_props(&[("type", "lane separator".into())]),
                        ));
                    }
                }
            }

            // Below renderings need lane centers to point in the direction of the lane
            for (lane, center) in road.lane_specs_ltr.iter().zip(lane_centers.iter_mut()) {
                if lane.dir == Direction::Back {
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
                    pairs.push((
                        arrow.to_geojson(gps_bounds),
                        make_props(&[("type", "lane arrow".into())]),
                    ));
                }
            }

            // Add stripes to show buffers. Ignore the type of the buffer for now -- we need to
            // decide all the types and how to render them.
            for (lane, center) in road.lane_specs_ltr.iter().zip(lane_centers.iter()) {
                if !matches!(lane.lt, LaneType::Buffer(_)) {
                    continue;
                }

                // Mark the sides of the lane clearly
                let thickness = Distance::meters(0.25);
                pairs.push((
                    center
                        .must_shift_right((lane.width - thickness) / 2.0)
                        .make_polygons(thickness)
                        .to_geojson(gps_bounds),
                    make_props(&[("type", "buffer edge".into())]),
                ));
                pairs.push((
                    center
                        .must_shift_left((lane.width - thickness) / 2.0)
                        .make_polygons(thickness)
                        .to_geojson(gps_bounds),
                    make_props(&[("type", "buffer edge".into())]),
                ));

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
                    pairs.push((
                        Line::must_new(left, right)
                            .make_polygons(thickness)
                            .to_geojson(gps_bounds),
                        make_props(&[("type", "buffer stripe".into())]),
                    ));
                }
            }
        }

        let obj = geom::geometries_with_properties_to_geojson(pairs);
        let output = serde_json::to_string_pretty(&obj)?;
        Ok(output)
    }

    /// For an intersection, show the clockwise ordering of roads around it
    pub fn debug_clockwise_ordering_geojson(&self) -> Result<String> {
        let mut pairs = Vec::new();

        for (i, intersection) in &self.intersections {
            for (idx, r) in intersection.roads.iter().enumerate() {
                let road = &self.roads[r];
                let pt = if road.src_i == *i {
                    road.trimmed_center_line.first_pt()
                } else {
                    road.trimmed_center_line.last_pt()
                };
                pairs.push((
                    pt.to_geojson(Some(&self.gps_bounds)),
                    make_props(&[(
                        "label",
                        format!("{} / {}", idx + 1, intersection.roads.len()).into(),
                    )]),
                ));
            }
        }

        let obj = geom::geometries_with_properties_to_geojson(pairs);
        let output = serde_json::to_string_pretty(&obj)?;
        Ok(output)
    }

    /// For an intersection, show all the movements.
    pub fn debug_movements_geojson(&self) -> Result<String> {
        // Each movement is represented as an arrow from the end of one road to the beginning of
        // another. To stop arrows overlapping, arrows to/from bidirectional roads are offset from
        // the center to the appropriate driving side.
        let arrow_fwd_offset_dist = if self.config.driving_side == DrivingSide::Right {
            Distance::meters(-1.3)
        } else {
            Distance::meters(1.3)
        };

        let mut pairs = Vec::new();

        for (i, intersection) in &self.intersections {
            // Find the points where the arrows should (leave, enter) the roads.
            let road_points: BTreeMap<_, _> = intersection
                .roads
                .iter()
                .map(|r| {
                    let road = &self.roads[r];
                    let first_road_segment = if road.src_i == *i {
                        road.trimmed_center_line.first_line()
                    } else {
                        road.trimmed_center_line.last_line().reversed()
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
            for (a, b) in &intersection.movements {
                if a != b {
                    if let Ok(line) = Line::new(road_points[a].0, road_points[b].1) {
                        pairs.push((
                            line.to_polyline()
                                .make_arrow(Distance::meters(0.5), ArrowCap::Triangle)
                                .to_geojson(Some(&self.gps_bounds)),
                            make_props(&[]),
                        ))
                    }
                }
            }
        }

        let obj = geom::geometries_with_properties_to_geojson(pairs);
        let output = serde_json::to_string_pretty(&obj)?;
        Ok(output)
    }

    pub fn to_intersection_markings_geojson(&self) -> Result<String> {
        let mut pairs = Vec::new();

        for intersection in self.intersections.values() {
            for polygon in make_sidewalk_corners(self, intersection) {
                pairs.push((
                    polygon.to_geojson(Some(&self.gps_bounds)),
                    make_props(&[("type", "sidewalk corner".into())]),
                ));
            }
        }
        let obj = geom::geometries_with_properties_to_geojson(pairs);
        let output = serde_json::to_string_pretty(&obj)?;
        Ok(output)
    }
}

impl DebugStreets {
    /// None if there's nothing labelled
    pub fn to_debug_geojson(&self) -> Option<String> {
        let mut pairs = Vec::new();
        for (pt, label) in &self.points {
            pairs.push((
                pt.to_geojson(Some(&self.streets.gps_bounds)),
                make_props(&[("label", label.to_string().into())]),
            ));
        }
        for (pl, label) in &self.polylines {
            pairs.push((
                pl.to_geojson(Some(&self.streets.gps_bounds)),
                make_props(&[("label", label.to_string().into())]),
            ));
        }
        if pairs.is_empty() {
            return None;
        }
        let obj = geom::geometries_with_properties_to_geojson(pairs);
        Some(serde_json::to_string_pretty(&obj).unwrap())
    }
}

fn make_props(list: &[(&str, serde_json::Value)]) -> serde_json::Map<String, serde_json::Value> {
    let mut props = serde_json::Map::new();
    for (x, y) in list {
        props.insert(x.to_string(), y.clone());
    }
    props
}

// TODO Where should this live?
/// For an intersection, show all corners where sidewalks meet.
fn make_sidewalk_corners(streets: &StreetNetwork, intersection: &Intersection) -> Vec<Polygon> {
    #[derive(Clone)]
    struct Edge {
        road: RoadID,
        // Pointed into the intersection
        pl: PolyLine,
        lane: LaneSpec,
    }

    // Get the left and right edge of each road, pointed into the intersection. All sorted
    // clockwise
    // TODO Use the road view idea instead. Or just refactor this.
    let mut edges = Vec::new();
    for road in streets.roads_per_intersection(intersection.id) {
        let mut left = Edge {
            road: road.id,
            pl: road
                .trimmed_center_line
                .must_shift_left(road.total_width() / 2.0),
            lane: road.lane_specs_ltr[0].clone(),
        };
        let mut right = Edge {
            road: road.id,
            pl: road
                .trimmed_center_line
                .must_shift_right(road.total_width() / 2.0),
            lane: road.lane_specs_ltr.last().unwrap().clone(),
        };
        if road.dst_i == intersection.id {
            edges.push(right);
            edges.push(left);
        } else {
            left.pl = left.pl.reversed();
            right.pl = right.pl.reversed();
            edges.push(left);
            edges.push(right);
        }
    }

    // Look at every adjacent pair
    let mut results = Vec::new();
    edges.push(edges[0].clone());
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
