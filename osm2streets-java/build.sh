#!/bin/bash
set -ex

javac -d build -h src src/*.java -target 11 -source 11
cargo build
cd build
jar cfe StreetNetwork.jar org.osm2streets.StreetNetwork org/osm2streets/*.class -C ../../target/debug/ libosm2streets_java.so
