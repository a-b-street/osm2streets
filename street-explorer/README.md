# Street Explorer: the osm2streets test case browser

## Running

[Install wasm-pack](https://rustwasm.github.io/wasm-pack/installer/), then:

```
street-explorer/> ./serve_locally.sh
```

You can edit HTML, CSS, and JS and just refresh the page immediately. If you
modify the Rust code, you must re-run `serve_locally.sh`, which will recompile.

## Architecture

We're not using any JS or build framework. We use `wasm-pack` to build a WASM +
JS API to the Rust code. For serving all of the CSS, JS, WASM, and test data
assets, we just use a plain HTTP file server and make use of symlinks.
