import { downloadGeneratedFile } from "./files.js";
import {
  makeLaneMarkingsLayer,
  lanePolygonStyle,
  makePlainGeoJsonLayer,
} from "./layers.js";
import { setupLeafletMap } from "./leaflet.js";
import init, { JsStreetNetwork } from "./osm2streets-js/osm2streets_js.js";

await init();

export class LaneEditor {
  constructor(mapContainer) {
    this.map = setupLeafletMap(mapContainer);
    this.network = null;
    this.layers = [];

    // Wire up the import button
    const importButton = document.getElementById("import-view");
    importButton.onclick = async () => {
      if (this.map.getZoom() < 15) {
        window.alert("Zoom in more to import");
        return;
      }

      await this.importView(importButton);
    };
  }

  async importView(importButton) {
    // Grab OSM XML from Overpass
    // (Sadly toBBoxString doesn't seem to match the order for Overpass)
    const b = app.map.getBounds();
    const bbox = `${b.getSouth()},${b.getWest()},${b.getNorth()},${b.getEast()}`;
    const query = `(nwr(${bbox}); node(w)->.x; <;); out meta;`;
    const url = `https://overpass-api.de/api/interpreter?data=${query}`;
    console.log(`Fetching from overpass: ${url}`);

    importButton.innerText = "Downloading from Overpass...";
    // Prevent this function from happening twice in a row. It could also maybe
    // be nice to allow cancellation.
    importButton.disabled = true;

    try {
      const resp = await fetch(url);
      const osmXML = await resp.text();

      importButton.innerText = "Importing OSM data...";

      this.network = new JsStreetNetwork(osmXML, {
        debug_each_step: false,
        dual_carriageway_experiment: false,
        cycletrack_snapping_experiment: false,
        inferred_sidewalks: false,
        osm2lanes: false,
      });
      this.rerenderAll();
      const bounds = this.layers[0].getBounds();
      this.map.fitBounds(bounds, { animate: false });
    } catch (err) {
      window.alert(`Import failed: ${err}`);
    } finally {
      // Make the button clickable again
      importButton.innerText = "Import current view";
      importButton.disabled = false;
    }
  }

  rerenderAll() {
    for (const layer of this.layers) {
      layer.remove();
    }
    this.layers = [];

    // Just show intersections from the plain layer
    this.layers.push(
      L.geoJSON(JSON.parse(this.network.toGeojsonPlain()), {
        style: function (feature) {
          if (feature.properties.type == "intersection") {
            return {
              color: "black",
              fillOpacity: 0.7,
            };
          }
          return { fill: false, stroke: false };
        },
      })
    );
    this.layers.push(
      L.geoJSON(JSON.parse(this.network.toLanePolygonsGeojson()), {
        style: lanePolygonStyle,
      })
    );
    this.layers.push(
      makeLaneMarkingsLayer(this.network.toLaneMarkingsGeojson())
    );

    for (const layer of this.layers) {
      layer.addTo(this.map);
    }
  }
}
