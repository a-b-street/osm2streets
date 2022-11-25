use std::collections::BTreeSet;

use abstutil::Tags;
use geom::Distance;

use crate::{get_lane_specs_ltr, Direction, DrivingSide, MapConfig};

// osm2lanes has a more extensive unit test suite, so why does this one exist? This also checks the
// translation from osm2lanes output into osm2streets. This is particularly useful during migration
// to osm2lanes (https://github.com/a-b-street/osm2lanes/issues/248).
#[test]
fn test_osm_to_specs() {
    abstutil::logger::setup();

    let mut ok = true;
    for (url, mut input, driving_side, expected_lt, expected_dir) in vec![
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
        (
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
        ),
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
            "spddddbbps",
            "vvvv^^v^^^",
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
        (
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
        ),
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
            "SdddddS",
            "vvv^^^^",
        ),
        (
            "https://www.openstreetmap.org/way/335668924",
            vec!["lanes=1", "sidewalk=none"],
            DrivingSide::Right,
            "SddS",
            "vv^^",
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
            "sddBs",
            "^^vvv",
        ),
    ] {
        let cfg = MapConfig {
            driving_side,
            bikes_can_use_bus_lanes: true,
            inferred_sidewalks: true,
            street_parking_spot_length: Distance::meters(8.0),
            turn_on_red: true,
            // Flip this temporarily to work on the new integration
            osm2lanes: false,
            merge_osm_ways: BTreeSet::new(),
        };
        input.push("highway=residential");
        let actual = get_lane_specs_ltr(&tags(input.clone()), &cfg);
        let actual_lt: String = actual.iter().map(|s| s.lt.to_char()).collect();
        let actual_dir: String = actual
            .iter()
            .map(|s| if s.dir == Direction::Fwd { '^' } else { 'v' })
            .collect();
        if actual_lt != expected_lt || actual_dir != expected_dir {
            ok = false;
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
    assert!(ok);
}

fn tags(kv: Vec<&str>) -> Tags {
    let mut tags = Tags::empty();
    for pair in kv {
        let parts = pair.split('=').collect::<Vec<_>>();
        tags.insert(parts[0], parts[1]);
    }
    tags
}
