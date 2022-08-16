#[cfg(test)]
mod tests {
    use abstutil::Timer;
    use anyhow::{bail, Result};
    use serde::Deserialize;
    use std::fs::File;
    use street_network::{DrivingSide, Transformation};
    use streets::RoadNetwork;

    include!(concat!(env!("OUT_DIR"), "/tests.rs"));

    fn test(path: &str) -> Result<()> {
        // TODO We need to call abstutil::logger::setup() once globally to get all logs

        let mut timer = Timer::new("test osm2streets");

        println!("Working on {path}");
        let cfg: TestCase = serde_json::from_reader(File::open(format!("{path}/test.json"))?)?;
        // Read the output file before modifying it. If it doesn't exist, then we're creating a new
        // test case.
        let prior_json = std::fs::read_to_string(format!("{path}/geometry.json"))
            .unwrap_or_else(|_| String::new());
        let prior_dot = std::fs::read_to_string(format!("{path}/road_network.dot"))
            .unwrap_or_else(|_| String::new());

        let clip_pts = None;
        let mut street_network = import_streets::osm_to_street_network(
            &std::fs::read_to_string(format!("{path}/input.osm"))?,
            clip_pts,
            import_streets::Options::default_for_side(cfg.driving_side),
            &mut timer,
        )?;
        street_network
            .apply_transformations(Transformation::standard_for_clipped_areas(), &mut timer);
        street_network.save_to_geojson(format!("{path}/geometry.json"), &mut timer)?;

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

    #[derive(Deserialize)]
    struct TestCase {
        driving_side: DrivingSide,
        // There's also a notes field that's ignored
    }
}
