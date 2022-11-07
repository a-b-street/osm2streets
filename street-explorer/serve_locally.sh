#!/bin/bash
set -e # exit on failure
set -x # print commands as they are run

wasm-pack build --dev --target web ../osm2streets-js
python3 -m http.server --directory www/
