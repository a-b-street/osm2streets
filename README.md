# osm2streets

OpenStreetMap has many details about streets, but applications rendering or
simulating lane-level detail face many challenges: determining lane properties
along one street, calculating geometry of streets and junctions, handling
motorway entrances, dual carriageways, dog-leg intersections, placement tags,
and parallel sidewalks and cycleways. The goal of osm2streets is to transform
OSM into a cleaned-up street network graph with geometry.

## Project status

This repository is conspicuously missing code. The osm2streets effort is
underway, but splitting the code from the A/B Street codebase isn't easy. (It
is the ultimate goal, though.) Currently the logical parts of osm2streets are
scattered around:

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

Also deliberately absent is any sort of spec describing the output of
osm2streets, or how things should be layered. The piece that draws detailed
lane markings, for instance, maybe belongs as an optional piece on top.

## Next steps

Issues are likely to be more up-to-date. The short-term steps to make
osm2streets a proper project:

- finish making the `RawMap` abstraction "own" the geometry calculation
- set up unit tests for quickly verifying transformations
- move all the relevant code into this repo

Then some new "features" beyond what A/B Street handles today:

- placement tags
- motorway entrance/exit geometry, based on Ben's JOSM work
- merging some cases of dual carriageways and "sausage link" intersections

Longer-term ambitions:

- include vehicle and pedestrian movements in the output
- handle pedestrian areas and highway areas, when they're mapped

## Applications

- A/B Street is already "using" osm2streets (aka, the current implementation is
- embedded there)
- A web viewer with detailed geometry
  - This doesn't necessarily need to pre-generate any tiles at all. Stream in
    OSM data from Overpass, pipe through osm2streets (via WASM) and generate
    geometry, draw polygons!
- Plugins for iD and JOSM to display streets in detail, and more importantly,
  preview what edits would look like

Everything is (and will be) written in Rust, which can run natively, in the
browser with WASM, on the JVM, etc.

## Contributing

This is an early stage project, so there's lots of flexibility! Ideally once we
get the test scaffolding set up, there's lots of room to parallelize and make
quick progress in many directions. File an issue and let's start discussion!

## Further reading

- An early article just about [intersection geometry](https://a-b-street.github.io/docs/tech/map/geometry/index.html)
- The [followup talk at FOSSGIS](https://dabreegster.github.io/talks/map_model_v2/slides.html)
