use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use anyhow::Result;
use geom::{ArrowCap, Distance, Line, PolyLine};

use crate::{
    DebugStreets, Direction, DrivingSide, IntersectionComplexity, LaneType, StreetNetwork,
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
        for (id, road) in &self.roads {
            pairs.push((
                road.trimmed_center_line
                    .make_polygons(2.0 * road.total_width() / 2.0)
                    .to_geojson(Some(&self.gps_bounds)),
                make_props(&[
                    ("type", "road".into()),
                    ("osm_way_id", id.osm_way_id.0.into()),
                    ("src_i", id.i1.0.into()),
                    ("dst_i", id.i2.0.into()),
                ]),
            ));
        }

        // Polygon per intersection
        for (id, intersection) in &self.intersections {
            pairs.push((
                intersection.polygon.to_geojson(Some(&self.gps_bounds)),
                make_props(&[
                    ("type", "intersection".into()),
                    ("osm_node_id", id.0.into()),
                    (
                        "complexity",
                        if intersection.complexity == IntersectionComplexity::MultiConnection {
                            format!(
                                "{:?} {:?}",
                                intersection.complexity, intersection.conflict_level
                            )
                        } else {
                            format!("{:?}", intersection.complexity)
                        }
                        .into(),
                    ),
                    ("control", format!("{:?}", intersection.control).into()),
                    ("osm_link", id.to_string().into()),
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

        for (id, road) in &self.roads {
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
                        ("osm_link", id.osm_way_id.to_string().into()),
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
                let pl = &self.roads[r].trimmed_center_line;
                let pt = if r.i1 == *i {
                    pl.first_pt()
                } else {
                    pl.last_pt()
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
                    let first_road_segment = if road.id.i1 == *i {
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
            for (a, b) in intersection.movements.iter() {
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
