#!/bin/bash

wasm-pack build --dev --target web ../osm2streets-js
python3 -m http.server --directory www/
