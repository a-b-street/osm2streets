import * as fs from "fs";
import * as path from "path";

// Use this to update the OSM data for all test cases using Overpass
async function main() {
  for (let file of fs.readdirSync("../tests/src")) {
    let filePath = path.join("../tests/src", file);
    if (fs.statSync(filePath).isDirectory()) {
      await update(file);
    }
  }
}

async function update(test) {
  if (test == "frederiksted") {
    console.warn(`Skipping ${test}, since it's a special case using osm.pbf`);
    return;
  }

  console.log(`Updating OSM data for ${test}`);
  let boundary = JSON.parse(
    fs.readFileSync(path.join("../tests/src", test, "boundary.json")),
  ).features[0];
  let url = overpassQueryForPolygon(boundary);
  let resp = await fetch(url);
  let xml = await resp.text();

  fs.writeFileSync(path.join("../tests/src", test, "input.osm"), xml);
}

// Construct a query to extract all XML data in the polygon clip. See
// https://wiki.openstreetmap.org/wiki/Overpass_API/Overpass_QL
function overpassQueryForPolygon(feature) {
  let filter = 'poly:"';
  for (let [lng, lat] of feature.geometry.coordinates[0]) {
    filter += `${lat} ${lng} `;
  }
  filter = filter.slice(0, -1) + '"';
  let query = `(nwr(${filter}); node(w)->.x; <;); out meta;`;
  return `https://overpass-api.de/api/interpreter?data=${query}`;
}

main();
