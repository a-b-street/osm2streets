// TODO The hash changes every time the Rust code does, this is very brittle.
// See https://github.com/thedodd/trunk/issues/230 or stop using trunk.
import { import_osm } from "../osm2streets-js-5b7f56a6bec77166.js";

export const makeImportCurrentView = (map, btn) => {
  btn.onclick = async () => {
    if (map.getZoom() < 15) {
      window.alert("Zoom in more to import");
    }

    // Grab OSM XML from Overpass
    // (Sadly toBBoxString doesn't seem to match the order for Overpass)
    const b = map.getBounds();
    const bbox = `${b.getSouth()},${b.getWest()},${b.getNorth()},${b.getEast()}`;
    const query = `(nwr(${bbox}); node(w)->.x; <;); out meta;`;
    const url = `https://overpass-api.de/api/interpreter?data=${query}`;
    console.log(`Fetching from overpass: ${url}`);

    btn.innerText = "Downloading from Overpass...";
    // Prevent this function from happening twice in a row. It could also
    // maybe be nice to allow cancellation.
    btn.disabled = true;

    const resp = await fetch(url);
    // TODO Error handling and such
    const osmXML = await resp.text();

    btn.innerText = "Importing OSM data...";

    const output = import_osm(osmXML, {
      // TODO Ask overpass
      driving_side: "Right",
    });

    // TODO Definitely time to think about cleaning up old layers
    L.geoJSON(JSON.parse(output), { style: { color: "#f55" } }).addTo(map);

    // Make the button clickable again
    btn.innerText = "Import current view";
    btn.disabled = false;
  };
};
