# Street Explorer: the osm2streets test case browser

## Running

[Install Rust](https://www.rust-lang.org/tools/install), then:

```
tests-web/> cargo install trunk
tests-web/> trunk serve
```

## Architecture

We're not using any JS framework yet; vanilla JS suffices. We are using `trunk`
for 3 purposes -- triggering the WASM build of the Rust dependencies in
`osm2streets-js`, bundling JS, CSS, and test file assets, and as a local dev
server.  There are some issues documented in the code about `trunk`. See
[#29](https://github.com/a-b-street/osm2streets/issues/29) for more ideas about
serving tests.
