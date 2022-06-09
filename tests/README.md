# osm2streets test cases

`tests/> cargo test --release`

## What there is

This is a collection of test cases for osm2streets. Each test case has a directory with:

- `input.osm`, from the export tab of <https://www.openstreetmap.org> or saved from JOSM
- `test.json`, defining the `driving_side` and describing the situation
- `road_network.dot` [e.g.](https://github.com/a-b-street/osm2streets/blob/main/tests/src/seattle_triangle/raw_map.json), a Graphvis of the `RoadNetwork` intersections and roadways connections
- `raw_map.json` [e.g.](https://doctorbud.com/graphviz-viewer/?url=https:%2F%2Fraw.githubusercontent.com%2Fa-b-street%2Fosm2streets%2Fmain%2Ftests%2Fsrc%2Fseattle_triangle%2Froad_network.dot), a GeoJSON of the `RawMap` road and intersection polygons

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

View `raw_map.json` files with <https://geojson.io>, QGIS, or similar.

View `road_network.dot` files with <https://doctorbud.com/graphviz-viewer/>, or
`dot -Tpng -Kneato -Goverlap=scale -O road_network.dot` or similar.

TODO [osm2streets#22 make a slippy map](https://github.com/a-b-street/osm2streets/issues/22)

## Working on RawMap code and preventing regressions

When you work on the [RawMap
code](https://github.com/a-b-street/abstreet/tree/master/raw_map) or related
dependencies, there might be an effect on the test case output. When there's a
difference, you can generate the new GeoJSON file, manually view the old and
new version, and decide if the changes are acceptable or not. This process is
currently complicated because the code is in another repository from this one.

So while locally working on your changes, you must temporarily point
`osm2streets` to your local version of the `abstreet` repository.

1.  Edit `Cargo.toml`
2.  Replace every instance of `git = "https://github.com/dabreegster/abstreet"`
    with `path = "/path/to/your/abstreet/repo"`
3.  Run `cargo test --release` to include your current changes

Before committing changes in `osm2streets`, of course you should revert those
changes to `Cargo.toml`.

After you verify the changes and merge them into the upstream `abstreet`
repository, then back in `osm2streets` you can officially make the updates:

1.  Run `cargo update -p raw_map`. This will update `Cargo.lock`. (And note
    this will update all the other A/B Street dependencies, since they share a
    repository.)
2.  Run `cargo test --release` and commit any diffs (which you previously verified are intentional)
3.  Push your changes to `osm2streets`

This is tedious; we aim to move the `RawMap` implementation and other
dependencies into this repository.

## Adding new test cases

If you identify an interesting situation in OSM that isn't similar to an
existing test case, please add it!

1.  Go to <https://www.openstreetmap.org>
2.  Navigate to the area of interest
3.  Use the "export" tab to download a `.osm` file. You can adjust the bounding box manually.
4.  Create a new directory in `src`. The naming scheme is not very organized yet.
5.  Put your OSM XML file in that directory as `input.osm`
6.  Copy a `test.json` file from another directory and modify accordingly. `driving_side` is, of course, important to get correct. The `notes` are free-form, but please at least include a useful link to OSM to view the area there.
7.  Run `cargo test --release`. It will fail with something like `src/montlake_roundabout/raw_map.json has changed. Manually view the diff with geojson.io. If it's OK, commit the new output to git, and this test will pass.`
8.  Add the new `raw_map.json` file to git after viewing it.
9.  You can re-run `cargo test --release` to verify things now pass.
10. Push!

How large should the input OSM area be? Enough to cover whatever you want to
test, but otherwise minimal to not bloat the size of this repository. See
existing test cases for examples. Note that `osm2streets` will clip roads that
extend out of the bounding box and generate special "border" intersections
along the edges.
