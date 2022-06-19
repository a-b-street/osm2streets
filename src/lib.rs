#[cfg(test)]
mod tests {
    use abstio::MapName;
    use abstutil::Timer;
    use anyhow::{bail, Result};
    use raw_map::{DrivingSide, RawMap};
    use serde::Deserialize;
    use streets::RoadNetwork;

    #[test]
    fn test_osm2streets() {
        abstutil::logger::setup();

        let mut any = false;
        let mut timer = Timer::new("test osm2streets");
        for entry in std::fs::read_dir("src").unwrap() {
            let entry = entry.unwrap();
            if !entry.file_type().unwrap().is_dir() {
                continue;
            }
            let name = entry.path().display().to_string();
            any = true;
            test(name, &mut timer).unwrap();
        }
        assert!(any, "Didn't find any tests");
    }

    fn test(path: String, timer: &mut Timer) -> Result<()> {
        println!("Working on {path}");
        let cfg: TestCase = abstio::maybe_read_json(format!("{path}/test.json"), timer)?;
        // Read the output file before modifying it. If it doesn't exist, then we're creating a new
        // test case.
        let prior_json = std::fs::read_to_string(format!("{path}/raw_map.json"))
            .unwrap_or_else(|_| String::new());
        let prior_dot = std::fs::read_to_string(format!("{path}/road_network.dot"))
            .unwrap_or_else(|_| String::new());

        let mut raw_map = import_rawmap(format!("{path}/input.osm"), cfg.driving_side, timer);
        let consolidate_all_intersections = false;
        // Our clipped areas are very small; this would remove part of the intended input
        let remove_disconnected = false;
        raw_map.run_all_simplifications(consolidate_all_intersections, remove_disconnected, timer);
        raw_map.save_to_geojson(format!("{path}/raw_map.json"), timer)?;

        let road_network: RoadNetwork = raw_map.into();
        std::fs::write(format!("{path}/road_network.dot"), road_network.to_dot())?;

        let current_dot = std::fs::read_to_string(format!("{path}/road_network.dot"))?;
        if current_dot != current_dot {
            std::fs::write(format!("{path}/road_network.orig.dot"), prior_dot)?;
            bail!("./{}/road_network.dot is different! If it is OK, commit it.
./{0}/road_network.orig.dot is previous result. Compare it on https://doctorbud.com/graphviz-viewer/", path);
        }

        let current_json = std::fs::read_to_string(format!("{path}/raw_map.json"))?;
        if prior_json != current_json {
            std::fs::write(format!("{path}/raw_map.orig.json"), prior_json)?;
            bail!(
                "./{}/raw_map.json is different! If it is OK, commit it.
./{0}/raw_map.orig.json is previous result. Compare it on https://geojson.io",
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

    fn import_rawmap(osm_path: String, driving_side: DrivingSide, timer: &mut Timer) -> RawMap {
        let clip = None;
        convert_osm::convert(
            osm_path.clone(),
            MapName::new("zz", "osm2streets_test", &abstutil::basename(&osm_path)),
            clip,
            convert_osm::Options::default_for_side(driving_side),
            timer,
        )
    }
}
