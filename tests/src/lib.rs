#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Once;

    use abstutil::Timer;
    use anyhow::{bail, Result};
    use env_logger::{Builder, Env};
    use geom::LonLat;

    use osm2streets::{Filter, MapConfig, Transformation};

    static SETUP_LOGGER: Once = Once::new();

    include!(concat!(env!("OUT_DIR"), "/tests.rs"));

    fn test(path: &str) -> Result<()> {
        SETUP_LOGGER
            .call_once(|| Builder::from_env(Env::default().default_filter_or("info")).init());

        let mut timer = Timer::new("test osm2streets");

        println!("Working on {path}");

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

        // Read the output file before modifying it. If it doesn't exist, then we're creating a new
        // test case.
        let prior_geometry = std::fs::read_to_string(format!("{path}/geometry.json"))
            .unwrap_or_else(|_| String::new());
        std::fs::write(
            format!("{path}/geometry.json"),
            street_network.to_geojson(&Filter::All)?,
        )?;
        let current_geometry = std::fs::read_to_string(format!("{path}/geometry.json"))?;
        if prior_geometry != current_geometry {
            std::fs::write(format!("{path}/geometry.orig.json"), prior_geometry)?;
            bail!(
                "./{path}/geometry.json is different! If it is OK, commit it. Compare to
                ./{path}/geometry.orig.json or use two versions of Street Explorer"
            );
        }

        // Manually enable to do diff-testing on blocks.
        if true {
            let prior_blocks = std::fs::read_to_string(format!("{path}/blocks.json"))
                .unwrap_or_else(|_| String::new());
            std::fs::write(
                format!("{path}/blocks.json"),
                street_network.find_all_blocks(false)?,
            )?;
            let current_blocks = std::fs::read_to_string(format!("{path}/blocks.json"))?;
            if prior_blocks != current_blocks {
                std::fs::write(format!("{path}/blocks.orig.json"), prior_blocks)?;
                bail!(
                    "./{path}/blocks.json is different! If it is OK, commit it. Compare to
                ./{path}/blocks.orig.json or use two versions of Street Explorer"
                );
            }
        }

        Ok(())
    }
}
