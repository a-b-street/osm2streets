use std::fs::File;
use std::io::Write;
use std::path::Path;

use abstutil::Timer;
use anyhow::Result;
use geom::{ArrowCap, Distance, PolyLine};

use crate::{DebugStreets, Direction, LaneType, StreetNetwork};

impl StreetNetwork {
    /// Saves the plain GeoJSON rendering to a file.
    pub fn save_to_geojson(&self, output_path: String, timer: &mut Timer) -> Result<()> {
        let json_output = self.to_geojson(timer)?;
        std::fs::create_dir_all(Path::new(&output_path).parent().unwrap())?;
        let mut file = File::create(output_path)?;
        file.write_all(json_output.as_bytes())?;
        Ok(())
    }

    /// Generates a plain GeoJSON rendering with one polygon per road and intersection.
    pub fn to_geojson(&self, timer: &mut Timer) -> Result<String> {
        // TODO InitialMap is going away very soon, but we still need it
        let initial_map = crate::initial::InitialMap::new(self, timer);

        let mut pairs = Vec::new();

        // Add a line-string and polygon per road
        for (id, road) in &initial_map.roads {
            let properties = make_props(&[
                ("osm_way_id", id.osm_way_id.0.into()),
                ("src_i", id.i1.0.into()),
                ("dst_i", id.i2.0.into()),
            ]);
            pairs.push((
                road.trimmed_center_pts.to_geojson(Some(&self.gps_bounds)),
                properties.clone(),
            ));

            pairs.push((
                road.trimmed_center_pts
                    .make_polygons(2.0 * road.half_width)
                    .to_geojson(Some(&self.gps_bounds)),
                properties,
            ));
        }

        // Polygon per intersection
        for (id, intersection) in &initial_map.intersections {
            pairs.push((
                intersection.polygon.to_geojson(Some(&self.gps_bounds)),
                make_props(&[
                    ("intersection_id", id.0.into()),
                    ("fill", "#729fcf".into()),
                    (
                        "complexity",
                        format!("{:?}", intersection.complexity).into(),
                    ),
                ]),
            ));
        }

        let obj = geom::geometries_with_properties_to_geojson(pairs);
        let output = serde_json::to_string_pretty(&obj)?;
        Ok(output)
    }

    /// Generates a polygon per lane, with a property indicating type.
    pub fn to_lane_polygons_geojson(&self, timer: &mut Timer) -> Result<String> {
        // TODO InitialMap is going away very soon, but we still need it
        let initial_map = crate::initial::InitialMap::new(self, timer);

        let mut pairs = Vec::new();

        for (id, road) in &self.roads {
            for (lane, pl) in road.lane_specs_ltr.iter().zip(
                road.get_lane_center_lines(&initial_map.roads[id].trimmed_center_pts)
                    .into_iter(),
            ) {
                pairs.push((
                    pl.make_polygons(lane.width)
                        .to_geojson(Some(&self.gps_bounds)),
                    make_props(&[("type", format!("{:?}", lane.lt).into())]),
                ));
            }
        }

        let obj = geom::geometries_with_properties_to_geojson(pairs);
        let output = serde_json::to_string_pretty(&obj)?;
        Ok(output)
    }

    /// Generate polygons representing lane markings, with a property indicating type.
    pub fn to_lane_markings_geojson(&self, timer: &mut Timer) -> Result<String> {
        // TODO InitialMap is going away very soon, but we still need it
        let initial_map = crate::initial::InitialMap::new(self, timer);

        let mut pairs = Vec::new();

        for (id, road) in &self.roads {
            let lane_centers =
                road.get_lane_center_lines(&initial_map.roads[id].trimmed_center_pts);

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
                            poly.to_geojson(Some(&self.gps_bounds)),
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
                            poly.to_geojson(Some(&self.gps_bounds)),
                            make_props(&[("type", "lane separator".into())]),
                        ));
                    }
                }
            }

            // Draw arrows along any travel lane
            for (lane, mut center) in road.lane_specs_ltr.iter().zip(lane_centers.into_iter()) {
                if !lane.lt.is_for_moving_vehicles() {
                    continue;
                }
                if lane.dir == Direction::Back {
                    center = center.reversed();
                }

                let step_size = Distance::meters(20.0);
                let buffer_ends = Distance::meters(5.0);
                let arrow_len = Distance::meters(1.75);
                let thickness = Distance::meters(0.25);
                for (pt, angle) in center.step_along(step_size, buffer_ends) {
                    if let Ok(arrow) = PolyLine::must_new(vec![
                        pt.project_away(arrow_len / 2.0, angle.opposite()),
                        pt.project_away(arrow_len / 2.0, angle),
                    ])
                    .make_arrow(thickness * 2.0, ArrowCap::Triangle)
                    .to_outline(thickness / 2.0)
                    {
                        pairs.push((
                            arrow.to_geojson(Some(&self.gps_bounds)),
                            make_props(&[("type", "lane arrow".into())]),
                        ));
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
