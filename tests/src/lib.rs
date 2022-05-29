#[cfg(test)]
mod tests {
    use abstio::MapName;
    use abstutil::Timer;
    use anyhow::{bail, Result};
    use geom::Distance;
    use raw_map::{DrivingSide, RawMap};
    use serde::Deserialize;

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
        let prior_output = std::fs::read_to_string(format!("{path}/raw_map.json"))
            .unwrap_or_else(|_| String::new());

        let mut raw_map = import_rawmap(format!("{path}/input.osm"), cfg.driving_side, timer);
        let consolidate_all_intersections = false;
        // Our clipped areas are very small; this would remove part of the intended input
        let remove_disconnected = false;
        raw_map.run_all_simplifications(consolidate_all_intersections, remove_disconnected, timer);
        raw_map.save_to_geojson(format!("{path}/raw_map.json"), timer)?;

        let current_output = std::fs::read_to_string(format!("{path}/raw_map.json"))?;
        if prior_output != current_output {
            std::fs::write("old_raw_map.json", prior_output)?;
            bail!("{}/raw_map.json has changed. Manually view the diff with geojson.io. If it's OK, commit the new output to git, and this test will pass.", path);
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
            // All of these are boilerplate defaults, except for driving_side
            convert_osm::Options {
                map_config: map_model::MapConfig {
                    driving_side,
                    bikes_can_use_bus_lanes: true,
                    inferred_sidewalks: true,
                    street_parking_spot_length: Distance::meters(8.0),
                    turn_on_red: true,
                },
                onstreet_parking: convert_osm::OnstreetParking::JustOSM,
                public_offstreet_parking: convert_osm::PublicOffstreetParking::None,
                private_offstreet_parking: convert_osm::PrivateOffstreetParking::FixedPerBldg(1),
                include_railroads: true,
                extra_buildings: None,
                skip_local_roads: false,
                filter_crosswalks: false,
                gtfs_url: None,
                elevation: false,
            },
            timer,
        )
    }
}
