#!/bin/bash

set -e
javac StreetNetwork.java
cargo build
java -Djava.library.path=../target/x86_64-unknown-linux-gnu/debug/ StreetNetwork
