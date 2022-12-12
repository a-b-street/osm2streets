#[cfg(test)]
mod tests {
    use abstutil::Timer;
    use anyhow::{bail, Result};
    use experimental::RoadNetwork;
    use geom::LonLat;
    use osm2streets::{MapConfig, Transformation};

    include!(concat!(env!("OUT_DIR"), "/tests.rs"));

    fn test(path: &str) -> Result<()> {
        abstutil::logger::setup();

        let mut timer = Timer::new("test osm2streets");

        println!("Working on {path}");
        // Read the output file before modifying it. If it doesn't exist, then we're creating a new
        // test case.
        let prior_json = std::fs::read_to_string(format!("{path}/geometry.json"))
            .unwrap_or_else(|_| String::new());
        let prior_dot = std::fs::read_to_string(format!("{path}/road_network.dot"))
            .unwrap_or_else(|_| String::new());

        let clip_pts = Some(LonLat::read_geojson_polygon(&format!(
            "{path}/boundary.json"
        ))?);
        let (mut street_network, _) = streets_reader::osm_to_street_network(
            &std::fs::read_to_string(format!("{path}/input.osm"))?,
            clip_pts,
            MapConfig::default(),
            &mut timer,
        )?;
        street_network
            .apply_transformations(Transformation::standard_for_clipped_areas(), &mut timer);
        street_network.save_to_geojson(format!("{path}/geometry.json"))?;

        let road_network: RoadNetwork = street_network.into();
        std::fs::write(format!("{path}/road_network.dot"), road_network.to_dot())?;

        let current_dot = std::fs::read_to_string(format!("{path}/road_network.dot"))?;
        if prior_dot != current_dot {
            std::fs::write(format!("{path}/road_network.orig.dot"), prior_dot)?;
            bail!("./{}/road_network.dot is different! If it is OK, commit it.
./{0}/road_network.orig.dot is previous result. Compare it on https://doctorbud.com/graphviz-viewer/", path);
        }

        let current_json = std::fs::read_to_string(format!("{path}/geometry.json"))?;
        if prior_json != current_json {
            std::fs::write(format!("{path}/geometry.orig.json"), prior_json)?;
            bail!(
                "./{}/geometry.json is different! If it is OK, commit it.
./{0}/geometry.orig.json is previous result. Compare it on https://geojson.io",
                path
            );
        }
        Ok(())
    }
}
