#!/bin/bash
# This script is used for manually releasing new versions of
# https://www.npmjs.com/package/osm2streets-js and
# https://www.npmjs.com/package/osm2streets-js-node. It's necessary until
# wasm-pack can generate one package that works both in web browsers and Node:
# https://github.com/rustwasm/wasm-pack/issues/313

set -e

read -p "Did you bump the version number in osm2streets-js/Cargo.toml (y/n)? " answer
case ${answer:0:1} in
    y|Y )
        echo Building
    ;;
    * )
        exit
    ;;
esac

# Build the version for web browsers
cd osm2streets-js
rm -rf pkg
wasm-pack build --release --target web
mv pkg web_pkg

# Build the version for NodeJS.
wasm-pack build --release --target nodejs
# Manually change the package name
sed -ie "s/osm2streets-js/osm2streets-js-node/" pkg/package.json
mv pkg nodejs_pkg

echo "Manually check each package, then run npm publish"
