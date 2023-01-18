#!/bin/bash
set -ex

./build.sh
java -jar build/StreetNetwork.jar
