# Street Explorer: the osm2streets test case browser

## Running

[Install wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) and
[npm](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm). Run
`npm install` once.

```
# To rebuild the Rust library and WASM bindings
npm run wasm
# To run a local server
npm run dev
# To rebuild the Rust library and launch the local server
npm run wasm && npm run dev
```

You can edit HTML, CSS, and JS and just refresh the page immediately. If you
modify the Rust code, you must do `npm run wasm` again.

## Architecture

We use [Vite](https://vitejs.dev) for managing dependencies, bundling, and as
an HTTP server. `wasm-pack` builds a WASM + JS API to the Rust code.
