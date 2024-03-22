#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Once;

    use abstutil::Timer;
    use anyhow::{bail, Result};
    use env_logger::{Builder, Env};
    use geom::LonLat;

    use experimental::RoadNetwork;
    use osm2streets::{Filter, MapConfig, Transformation};

    static SETUP_LOGGER: Once = Once::new();

    include!(concat!(env!("OUT_DIR"), "/tests.rs"));

    fn test(path: &str) -> Result<()> {
        SETUP_LOGGER
            .call_once(|| Builder::from_env(Env::default().default_filter_or("info")).init());

        let mut timer = Timer::new("test osm2streets");

        println!("Working on {path}");
        // Read the output file before modifying it. If it doesn't exist, then we're creating a new
        // test case.
        let prior_json = std::fs::read_to_string(format!("{path}/geometry.json"))
            .unwrap_or_else(|_| String::new());
        let prior_dot = std::fs::read_to_string(format!("{path}/road_network.dot"))
            .unwrap_or_else(|_| String::new());

        let clip_pts = if Path::new(format!("{path}/boundary.json").as_str()).exists() {
            Some(LonLat::read_geojson_polygon(&format!(
                "{path}/boundary.json"
            ))?)
        } else {
            None
        };

        let (mut street_network, _) = if Path::new(format!("{path}/input.osm").as_str()).exists() {
            streets_reader::osm_to_street_network(
                &std::fs::read(format!("{path}/input.osm"))?,
                clip_pts,
                MapConfig::default(),
                &mut timer,
            )?
        } else {
            streets_reader::osm_to_street_network(
                &std::fs::read(format!("{path}/input.osm.pbf"))?,
                clip_pts,
                MapConfig::default(),
                &mut timer,
            )?
        };
        street_network.check_invariants();
        street_network.apply_transformations_with_invariant_checks(
            Transformation::standard_for_clipped_areas(),
            &mut timer,
        );
        std::fs::write(
            format!("{path}/geometry.json"),
            street_network.to_geojson(&Filter::All)?,
        )?;
        let road_network: RoadNetwork = street_network.into();

        std::fs::write(format!("{path}/road_network.dot"), road_network.to_dot())?;

        let current_dot = std::fs::read_to_string(format!("{path}/road_network.dot"))?;
        if prior_dot != current_dot {
            std::fs::write(format!("{path}/road_network.orig.dot"), prior_dot)?;
            bail!("./{}/road_network.dot is different! If it is OK, commit it. \
            ./{0}/road_network.orig.dot is previous result. Compare it on https://doctorbud.com/graphviz-viewer/", path);
        }

        let current_json = std::fs::read_to_string(format!("{path}/geometry.json"))?;
        if prior_json != current_json {
            std::fs::write(format!("{path}/geometry.orig.json"), prior_json)?;
            bail!(
                "./{}/geometry.json is different! If it is OK, commit it. \
                ./{0}/geometry.orig.json is previous result. Compare it on https://geojson.io",
                path
            );
        }
        Ok(())
    }
}
