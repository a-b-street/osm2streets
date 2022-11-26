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
    this.currentWay = null;

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

    this.layers.push(
      L.geoJSON(JSON.parse(this.network.toGeojsonPlain()), {
        // Just show intersections from the plain layer
        style: (feature) => {
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
        // Make lanes clickable
        onEachFeature: (feature, layer) => {
          layer.on({
            click: (ev) => {
              if (feature.properties.osm_way_ids.length != 1) {
                window.alert(
                  "This road doesn't match up with one OSM way; you can't edit it"
                );
              } else {
                this.editWay(BigInt(feature.properties.osm_way_ids[0]));
              }
            },
          });
        },
      })
    );
    this.layers.push(
      makeLaneMarkingsLayer(this.network.toLaneMarkingsGeojson())
    );

    for (const layer of this.layers) {
      layer.addTo(this.map);
    }
  }

  editWay(id) {
    this.currentWay = id;

    var html = `<table id="tags-table">`;

    const tags = JSON.parse(this.network.getOsmTagsForWay(id));
    for (let key in tags) {
      const value = tags[key];
      html += `<tr>`;
      html += `<td><input type="text" value="${key}"></td>`;
      html += `<td><input type="text" value="${value}"></td>`;
      html += `<td><button type="button">Delete</button></td>`;
      html += `</tr>`;
    }
    html += `</table>`;
    // TODO Add new tag

    html += `<button type="button" id="recalculate">Recalculate</button>`;

    const div = document.getElementById("tags");
    div.innerHTML = html;

    document.getElementById("recalculate").onclick = () => {
      this.recalculateWay();
    };
  }

  recalculateWay() {
    const tags = {};
    const table = document.getElementById("tags-table");
    for (var i = 0, row; (row = table.rows[i]); i++) {
      console.log(`heres a row`);
      var key = null;
      for (var j = 0, cell; (cell = row.cells[j]); j++) {
        if (cell.firstChild instanceof HTMLInputElement) {
          if (key) {
            tags[key] = cell.firstChild.value;
          } else {
            key = cell.firstChild.value;
          }
        }
      }
    }

    this.network.overwriteOsmTagsForWay(this.currentWay, JSON.stringify(tags));
    this.rerenderAll();
  }
}
