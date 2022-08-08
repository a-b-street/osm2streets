use std::fs::File;
use std::io::Write;
use std::path::Path;

use abstutil::Timer;
use anyhow::Result;
use geom::Distance;

use crate::{DebugStreets, StreetNetwork};

impl StreetNetwork {
    /// Saves the plain GeoJSON rendering to a file.
    pub fn save_to_geojson(&self, output_path: String, timer: &mut Timer) -> Result<()> {
        let json_output = self.to_geojson(timer)?;
        std::fs::create_dir_all(Path::new(&output_path).parent().unwrap())?;
        let mut file = File::create(output_path)?;
        file.write_all(json_output.as_bytes())?;
        Ok(())
    }

    /// Generates a plain GeoJSON rendering of roads and intersections.
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

    /// Generates a more detailed GeoJSON rendering of roads and intersections.
    pub fn to_detailed_geojson(&self, timer: &mut Timer) -> Result<String> {
        // TODO InitialMap is going away very soon, but we still need it
        let initial_map = crate::initial::InitialMap::new(self, timer);

        let mut pairs = Vec::new();

        for (id, road) in &initial_map.roads {
            // Paved road area
            pairs.push((
                road.trimmed_center_pts
                    .make_polygons(2.0 * road.half_width)
                    .to_geojson(Some(&self.gps_bounds)),
                make_props(&[("type", "road polygon".into())]),
            ));

            // Lane separators
            let mut width_so_far = Distance::ZERO;
            for lane in &self.roads[id].lane_specs_ltr {
                // Draw the left
                if let Ok(pl) = road
                    .trimmed_center_pts
                    .shift_from_center(2.0 * road.half_width, width_so_far)
                {
                    pairs.push((
                        pl.to_geojson(Some(&self.gps_bounds)),
                        make_props(&[("type", "lane separator".into())]),
                    ));
                }
                width_so_far += lane.width;
            }
            // The rightmost
            if let Ok(pl) = road
                .trimmed_center_pts
                .shift_from_center(2.0 * road.half_width, width_so_far)
            {
                pairs.push((
                    pl.to_geojson(Some(&self.gps_bounds)),
                    make_props(&[("type", "lane separator".into())]),
                ));
            }
        }

        for (_id, intersection) in &initial_map.intersections {
            pairs.push((
                intersection.polygon.to_geojson(Some(&self.gps_bounds)),
                make_props(&[("type", "intersection polygon".into())]),
            ));
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