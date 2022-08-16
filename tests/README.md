# osm2streets test cases

`tests/> cargo test --release`

## What there is

This is a collection of test cases for osm2streets. Each test case has a directory with:

- `input.osm`, from the export tab of <https://www.openstreetmap.org> or saved from JOSM
- `test.json`, defining the `driving_side` and describing the situation
- `geometry.json` [e.g.](https://github.com/a-b-street/osm2streets/blob/main/tests/src/seattle_triangle/geometry.json), a GeoJSON of the `StreetNetwork` road and intersection polygons
- `road_network.dot` [e.g.](https://doctorbud.com/graphviz-viewer/?url=https:%2F%2Fraw.githubusercontent.com%2Fa-b-street%2Fosm2streets%2Fmain%2Ftests%2Fsrc%2Fseattle_triangle%2Froad_network.dot), a Graphviz of the `RoadNetwork` intersections and roadways connections

The outputs here are *current implementations*, not *gold standards* (though we could add
some of them). Use them to observe your changes as you work. (more coming soon...)

## Running the tests

[Install Rust](https://www.rust-lang.org/tools/install), then:

```shell
git clone https://github.com/a-b-street/osm2streets
cd osm2streets/tests
cargo test --release
```

You can also omit `--release` for faster compilation, but slower running. Each
test case is expensive enough to justify release mode.

View `geometry.json` files with <https://geojson.io>, QGIS, or similar.

View `road_network.dot` files with <https://doctorbud.com/graphviz-viewer/>, or
`dot -Tpng -Kneato -Goverlap=scale -O road_network.dot` or similar.

TODO [osm2streets#22 make a slippy map](https://github.com/a-b-street/osm2streets/issues/22)

## Working on street_network code and preventing regressions

When you work on the [street_network
code](https://github.com/a-b-street/osm2streets/tree/main/street_network) or
related dependencies, there might be an effect on the test case output. When
there's a difference, you can generate the new GeoJSON file, manually view the
old and new version, and decide if the changes are acceptable or not.

1.  Run `cargo test --release` to test your current changes
2.  Manually verify any diffs. Commit the ones that are intentional.
3.  Push your changes

## Adding new test cases

If you identify an interesting situation in OSM that isn't similar to an
existing test case, please add it!

1.  Go to <https://a-b-street.github.io/osm2streets/>
2.  Navigate to the area of interest
3.  Press `Download osm.xml`
4.  Create a new directory in `src`. The naming scheme is not very organized yet.
5.  Put your OSM XML file in that directory as `input.osm`
6.  Copy a `test.json` file from another directory and modify accordingly. `driving_side` is, of course, important to get correct. The `notes` are free-form, but please at least include a useful link to OSM to view the area there.
7.  Run `cargo test --release`. It will fail with something like `src/montlake_roundabout/geometry.json has changed. Manually view the diff with geojson.io. If it's OK, commit the new output to git, and this test will pass.`
8.  Add the new `geometry.json` file to git after viewing it.
9.  You can re-run `cargo test --release` to verify things now pass.
10. Push!

How large should the input OSM area be? Enough to cover whatever you want to
test, but otherwise minimal to not bloat the size of this repository. See
existing test cases for examples. Note that `osm2streets` will clip roads that
extend out of the bounding box and generate special "border" intersections
along the edges.
