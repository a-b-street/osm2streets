# osm2streets test cases

This is a collection of test cases for osm2streets. Each test case has a directory with:

- `input.osm`, from the export tab of <https://www.openstreetmap.org>
- `test.json`, defining the `driving_side` and describing the situation
- `raw_map.json`, a GeoJSON file showing the resulting road and intersection polygons


Most output files are not what we intend osm2streets to look like. The
intention of this crate is to establish regression tests for tricky situations.
When we improve RawMap transformations, we can run these tests to see what
changes, and manually approve/reject any changed GeoJSON files.

The output GeoJSON files can be viewed with <http://geojson.io>, QGIS, or similar.

## Running the tests

[Install Rust](https://www.rust-lang.org/tools/install), then:

```shell
git clone https://github.com/a-b-street/osm2streets
cd osm2streets
cargo test --release
```

You can also omit `--release` for faster compilation, but slower running. Each
test case is expensive enough to justify release mode.

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
8.  `raw_map.json` will be created in that directory; it should show up as a new untracked file in git. Check the current output with QGIS or <http://geojson.io>.
9.  Add the file in git.
10.  You can re-run `cargo test --release` to verify things now pass.
11. Push!

How large should the input OSM area be? Enough to cover whatever you want to
test, but otherwise minimal to not bloat the size of this repository. See
existing test cases for examples. Note that `osm2streets` will clip roads that
extend out of the bounding box and generate special "border" intersections
along the edges.
