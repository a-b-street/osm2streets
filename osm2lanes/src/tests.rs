use std::sync::Once;

use abstutil::Tags;
use env_logger::{Builder, Env};

use crate::{get_lane_specs_ltr, Direction, DrivingSide, MapConfig};

static SETUP_LOGGER: Once = Once::new();

#[test]
fn test_osm_to_specs() {
    SETUP_LOGGER.call_once(|| Builder::from_env(Env::default().default_filter_or("info")).init());

    let cases = [
        (
            "https://www.openstreetmap.org/way/428294122",
            vec![
                "lanes=2",
                "oneway=yes",
                "sidewalk=both",
                "cycleway:left=lane",
            ],
            DrivingSide::Right,
            "sbdds",
            "v^^^^",
        ),
        /* (
            "https://www.openstreetmap.org/way/8591383",
            vec![
                "lanes=1",
                "oneway=yes",
                "sidewalk=both",
                "cycleway:left=track",
                "oneway:bicycle=no",
            ],
            DrivingSide::Right,
            "sbbds",
            "vv^^^",
        ), */
        (
            // A slight variation of the above, using cycleway:left:oneway=no, which should be
            // equivalent
            "https://www.openstreetmap.org/way/8591383",
            vec![
                "lanes=1",
                "oneway=yes",
                "sidewalk=both",
                "cycleway:left=track",
                "cycleway:left:oneway=no",
            ],
            DrivingSide::Right,
            "sbbds",
            "vv^^^",
        ),
        (
            "https://www.openstreetmap.org/way/353690151",
            vec![
                "lanes=4",
                "sidewalk=both",
                "parking:lane:both=parallel",
                "cycleway:right=track",
                "cycleway:right:oneway=no",
            ],
            DrivingSide::Right,
            "spddddpbbs",
            "vvvv^^^v^^",
        ),
        (
            "https://www.openstreetmap.org/way/389654080",
            vec![
                "lanes=2",
                "sidewalk=both",
                "parking:lane:left=parallel",
                "parking:lane:right=no_stopping",
                "centre_turn_lane=yes",
                "cycleway:right=track",
                "cycleway:right:oneway=no",
            ],
            DrivingSide::Right,
            "spdCdbbs",
            "vvv^^v^^",
        ),
        /* (
            "https://www.openstreetmap.org/way/369623526",
            vec![
                "lanes=1",
                "oneway=yes",
                "sidewalk=both",
                "parking:lane:right=diagonal",
                "cycleway:left=opposite_track",
                "oneway:bicycle=no",
            ],
            DrivingSide::Right,
            "sbbdps",
            "vv^^^^",
        ), */
        (
            "https://www.openstreetmap.org/way/534549104",
            vec![
                "lanes=2",
                "oneway=yes",
                "sidewalk=both",
                "cycleway:right=track",
                "cycleway:right:oneway=no",
                "oneway:bicycle=no",
            ],
            DrivingSide::Right,
            "sddbbs",
            "v^^v^^",
        ),
        (
            "https://www.openstreetmap.org/way/777565028",
            vec!["highway=residential", "oneway=no", "sidewalk=both"],
            DrivingSide::Left,
            "sdds",
            "^^vv",
        ),
        (
            "https://www.openstreetmap.org/way/224637155",
            vec!["lanes=2", "oneway=yes", "sidewalk=left"],
            DrivingSide::Left,
            "sdd",
            "^^^",
        ),
        (
            "https://www.openstreetmap.org/way/4188078",
            vec![
                "lanes=2",
                "cycleway:left=lane",
                "oneway=yes",
                "sidewalk=left",
            ],
            DrivingSide::Left,
            "sbdd",
            "^^^^",
        ),
        (
            "https://www.openstreetmap.org/way/49207928",
            vec!["cycleway:right=lane", "sidewalk=both"],
            DrivingSide::Left,
            "sddbs",
            "^^vvv",
        ),
        // How should an odd number of lanes forward/backwards be split without any clues?
        (
            "https://www.openstreetmap.org/way/898731283",
            vec!["lanes=3", "sidewalk=both"],
            DrivingSide::Left,
            "sddds",
            "^^^vv",
        ),
        (
            // I didn't look for a real example of this
            "https://www.openstreetmap.org/way/898731283",
            vec!["lanes=5", "sidewalk=none"],
            DrivingSide::Right,
            "ddddd",
            "vv^^^",
        ),
        (
            "https://www.openstreetmap.org/way/335668924",
            vec!["lanes=1", "sidewalk=none"],
            DrivingSide::Right,
            "d",
            "^",
        ),
        (
            "https://www.openstreetmap.org/way/632329263",
            vec![
                "bus:lanes:backward=designated|yes",
                "lanes=3",
                "lanes:backward=2",
                "lanes:bus:backward=1",
                "lanes:forward=1",
                "psv=yes",
            ],
            DrivingSide::Left,
            "ddB",
            "^vv",
        ),
        (
            "https://www.openstreetmap.org/way/4013378",
            vec!["busway:left=lane", "cycleway:left=lane", "oneway=yes"],
            DrivingSide::Left,
            "bBd",
            "^^^",
        ),
        (
            "https://www.openstreetmap.org/way/312855494",
            vec!["busway:right=lane"],
            DrivingSide::Left,
            "ddB",
            "^vv",
        ),
        (
            "https://www.openstreetmap.org/way/228767989",
            vec!["busway:both=lane", "sidewalk=both"],
            DrivingSide::Left,
            "sBddBs",
            "^^^vvv",
        ),
        (
            "https://www.openstreetmap.org/way/905830125",
            vec!["highway=cycleway", "oneway=yes"],
            DrivingSide::Left,
            "b",
            "^",
        ),
        (
            "https://www.openstreetmap.org/way/414489468",
            vec![
                "highway=cycleway",
                "oneway=no",
                "segregated=yes",
                "sidewalk=right",
            ],
            DrivingSide::Left,
            "bbs",
            "^vv",
        ),
        (
            "https://www.openstreetmap.org/way/705809125",
            vec![
                "highway=cycleway",
                "oneway=yes",
                "segregated=yes",
                "sidewalk=left",
            ],
            DrivingSide::Left,
            "sb",
            "^^",
        ),
        /* (
            "https://www.openstreetmap.org/way/539534598",
            vec!["highway=cycleway", "oneway=no", "segregated=no"],
            DrivingSide::Left,
            "F",
            "^",
        ), */
        (
            "https://www.openstreetmap.org/way/280732115",
            vec!["highway=cycleway", "foot=yes", "segregated=no"],
            DrivingSide::Left,
            "F",
            "^",
        ),
        (
            // Sidewalks tagged on highway=footway are invalid
            "https://www.openstreetmap.org/way/523882355",
            vec![
                "bicycle=yes",
                "foot=yes",
                "highway=footway",
                "sidewalk=both",
            ],
            DrivingSide::Right,
            "F",
            "^",
        ),
        (
            // Lots of detail about sidewalk:left, but no "sidewalk=left" or similar
            "https://www.openstreetmap.org/way/148338681",
            vec![
                "highway=residential",
                "lanes=1",
                "oneway=yes",
                "sidewalk:left:width=0.9",
            ],
            DrivingSide::Right,
            "d",
            "^",
        ),
        (
            "https://www.openstreetmap.org/way/23806634",
            vec![
                "highway=secondary_link",
                "lanes=2",
                "oneway=yes",
                "turn:lanes=reverse;left|left",
            ],
            DrivingSide::Right,
            "dd",
            "^^",
        ),
        (
            "https://www.openstreetmap.org/way/528310266",
            vec!["highway=motorway", "lanes=5", "oneway=yes", "bicycle=no"],
            DrivingSide::Right,
            "ddddd",
            "^^^^^",
        ),
    ];
    let cases_count = cases.len();

    let mut failing = 0;
    for (url, mut input, driving_side, expected_lt, expected_dir) in cases {
        let mut cfg = MapConfig::default();
        cfg.driving_side = driving_side;
        if input.iter().all(|x| !x.starts_with("highway=")) {
            input.push("highway=residential");
        }
        let actual = get_lane_specs_ltr(&tags(input.clone()), &cfg);
        let actual_lt: String = actual.iter().map(|s| s.lt.to_char()).collect();
        let actual_dir: String = actual
            .iter()
            .map(|s| if s.dir == Direction::Fwd { '^' } else { 'v' })
            .collect();
        if actual_lt != expected_lt || actual_dir != expected_dir {
            failing += 1;
            println!("For input (example from {}):", url);
            for kv in input {
                println!("    {}", kv);
            }
            println!("Got:");
            println!("    {}", actual_lt);
            println!("    {}", actual_dir);
            println!("Expected:");
            println!("    {}", expected_lt);
            println!("    {}", expected_dir);
            println!();
        }
    }
    assert!(
        failing == 0,
        "{}/{} spec tests failing",
        failing,
        cases_count
    );
}

fn tags(kv: Vec<&str>) -> Tags {
    let mut tags = Tags::empty();
    for pair in kv {
        let parts = pair.split('=').collect::<Vec<_>>();
        tags.insert(parts[0], parts[1]);
    }
    tags
}
