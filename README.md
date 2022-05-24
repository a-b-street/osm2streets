# osm2streets

OpenStreetMap has many details about streets, but applications rendering or
simulating lane-level detail face many challenges: determining lane properties
along one street, calculating geometry of streets and junctions, handling
motorway entrances, dual carriageways, dog-leg intersections, placement tags,
and parallel sidewalks and cycleways. The goal of osm2streets is to transform
OSM into a cleaned-up street network graph with geometry.

## Project status

This repository is conspicuously missing functionality. The osm2streets effort is
underway, but splitting the code from the A/B Street codebase will take some time.
We will iteratively import and move in different logical parts of the osm2streets scope
from around the place:

- [osm2lanes](https://github.com/a-b-street/osm2lanes) determines the lanes
  along one OSM way
- [convert_osm](https://github.com/a-b-street/abstreet/tree/master/convert_osm)
  reads OSM XML files and produces a `RawMap`
- [raw_map](https://github.com/a-b-street/abstreet/tree/master/raw_map) is the
  current graph + geometry representation, containing all of the transformations
- [map_editor](https://github.com/a-b-street/abstreet/tree/master/apps/map_editor)
  is a UI that can display and interactively transform `RawMaps`
- [map_gui](https://github.com/a-b-street/abstreet/tree/master/map_gui/src/render)
  contains code to draw lane markings

Also deliberately absent is any definitive spec describing the output of
osm2streets, or how things should be layered. The piece that draws detailed
lane markings, for instance, maybe belongs as an optional piece on top.
Looking towards [this proposal](https://github.com/a-b-street/osm2streets/issues/5#issuecomment-1114305718),
we will iterate on types, APIs and documentation, to circle in on a sensible starting place for osm2streets.

## Next steps

Issues are likely to be more up-to-date. The short-term steps to make
osm2streets a proper project:

- Iterate on the api/docs, working towards [abstreet/RawMap](ttps://github.com/a-b-street/abstreet/blob/master/raw_map/src/lib.rs)
- Iterate on `RawMap` working towards modularity and the ideas in the emerging api/docs
  - If `RawMap` get good enough, we we can drop the experimentation on the api,
  - otherwise, one by one, all the pieces eventully get integrated into the api
- [#8](https://github.com/a-b-street/osm2streets/issues/8) set up unit tests for quickly verifying transformations
- [#13](https://github.com/a-b-street/osm2streets/issues/13) create a slippy map to visualise and understand the resulting networks
- (in [abstreet](https://github.com/a-b-street/abstreet)) finish making the `RawMap` abstraction "own" the geometry calculation
- move all the relevant code into this repo piece by piece

Then some new "features" beyond what A/B Street handles today:

- placement tags
- motorway entrance/exit geometry, based on [Ben's JOSM work](https://github.com/BjornRasmussen/Lanes/pull/8)
- merging some cases of dual carriageways and "sausage link" intersections

Longer-term ambitions:

- include vehicle and pedestrian movements in the output
- handle pedestrian areas and highway areas, when they're mapped

## Applications

- A/B Street is already "using" osm2streets (aka, the current implementation is embedded there)
- [#8](https://github.com/a-b-street/osm2streets/issues/8) A slippy map web viewer with detailed geometry.
  - [#12](https://github.com/a-b-street/osm2streets/issues/12) Render data as raster for a tileserver for layers.
  - This doesn't necessarily need to pre-generate any tiles at all. Stream in
    OSM data from Overpass, pipe through osm2streets (via WASM) and generate
    geometry, draw polygons!
- Plugins for iD and JOSM to display streets in detail, and more importantly,
  preview what edits would look like.

Everything is (and will be) written in Rust, which can run natively, in the
browser with WASM, on the JVM, etc.

## Contributing

This is an early stage project, so there's lots of flexibility! Ideally once we
get the test scaffolding set up, there's lots of room to parallelize and make
quick progress in many directions. File an issue and let's start discussion!

## Further reading

- An early article just about [intersection geometry](https://a-b-street.github.io/docs/tech/map/geometry/index.html)
- The [followup talk at FOSSGIS](https://dabreegster.github.io/talks/map_model_v2/slides.html)
