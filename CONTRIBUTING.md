# Working on osm2streets

Come hang out at https://github.com/a-b-street/osm2streets to ask questions and improve these docs.

## Pre-requisites

- latest stable Rust
- [wasm-pack](https://github.com/rustwasm/wasm-pack)

## Tests

See the `tests` crate [(docs)](tests/README.md) for a collection of interesting example scenarios.

See `street-explorer` crate [(docs)](street-explorer/README.md) for a *Street Explorer* test browsing interface.

## Design decisions

The `Road` and `Intersection` structs are rather stateful. `trimmed_road_line` and `polygon` are derived state, produced by a transformation step. Derived state isn't updated automatically, so you have to be internally careful about knowing what transformations have run. Some earlier ones call `estimate_trimmed_geometry` for this reason.
