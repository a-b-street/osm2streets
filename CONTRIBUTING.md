# Working on osm2streets

Come hang out at <https://github.com/a-b-street/osm2streets> to ask questions and improve these docs.

Read [how it works](docs/how_it_works.md)

## Pre-requisites

- latest stable Rust
- [wasm-pack](https://github.com/rustwasm/wasm-pack)

## Tests

See the `tests` crate [(docs)](tests/README.md) for a collection of interesting example scenarios.

See `street-explorer` crate [(docs)](street-explorer/README.md) for a *Street Explorer* test browsing interface.

## Developer notes

To release a new version of <https://www.npmjs.com/package/osm2streets-js>, bump the version number in `osm2streets-js/Cargo.toml` and run `wasm-pack publish`. By current permissions, only Dustin can run this, but we could add others.
