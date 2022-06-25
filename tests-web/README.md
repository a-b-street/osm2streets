# Street Explorer: the osm2streets test case browser

## Running

[Install Rust](https://www.rust-lang.org/tools/install), then:

```
tests-web/> cargo install trunk
tests-web/> trunk serve
```

## Architecture

No JS build system is needed yet, we're writing vanilla JS that can be served as is.

Currently, `trunk` serves the static site that loads test case files that are compiled in as assets.

Ideally: A static site that can be deployed or served with any http server, like `python -m http` or `trunk serve`,
which reads from the tests dir directly. See [#29](https://github.com/a-b-street/osm2streets/issues/29) about serving tests.
