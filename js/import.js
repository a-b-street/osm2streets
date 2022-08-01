import init, { JsStreetNetwork } from "./osm2streets-js/osm2streets_js.js";
import { makePlainGeoJsonLayer, makeDetailedGeoJsonLayer } from "./layers.js";
await init();

export const makeImportCurrentView = (map, btn) => {
  btn.onclick = async () => {
    if (map.getZoom() < 15) {
      window.alert("Zoom in more to import");
      return;
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

    try {
      const resp = await fetch(url);
      const osmXML = await resp.text();

      btn.innerText = "Importing OSM data...";

      const network = new JsStreetNetwork(osmXML, {
        // TODO Ask overpass
        driving_side: "Right",
      });

      // TODO Definitely time to think about cleaning up old layers
      makePlainGeoJsonLayer(network.toGeojsonPlain()).addTo(map);
      makeDetailedGeoJsonLayer(network.toGeojsonDetailed()).addTo(map);
    } catch (err) {
      window.alert(`Import failed: ${err}`);
    }

    // Make the button clickable again
    btn.innerText = "Import current view";
    btn.disabled = false;
  };
};
