# osm2streets Svelte web app

This directory contains web apps for interactively exploring and using osm2streets. The current deployed version is at <https://a-b-street.github.io/osm2streets/>. The apps include:

- Street Explorer: import OSM data from Overpass or test cases, visualize and debug the output
- Lane editor: modify OSM way tags and visualize the results, to make editing lane tags easier

## Installation

To run locally you'll need:
[npm](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm) and
[wasm-pack](https://github.com/rustwasm/wasm-pack).

- `npm install` to install dependencies
- `npm run wasm` to rebuild the `osm2streets-js` Rust library
- `npm run dev` to work locally
  - If you're modifying the Rust code, a handy command is `npm run wasm && npm run dev`
- `npm run fmt` to auto-format code
- `npm run check` to see TypeScript errors

## Architecture

The tech stack is:

- [Vite](https://vitejs.dev) as the build tool
- [Svelte](https://svelte.dev) as the UI framework
- [MapLibre GL](https://maplibre.org) as the map

The code is organized as a common library and apps built on top:

- `src/osm2streets-svelte`: a common library (eventually on NPM)
- `src/street-explorer` and `src/lane-editor`: two end-user apps

To avoid NPM headaches, everything is one NPM package (and not yet published).
