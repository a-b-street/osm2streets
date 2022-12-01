import { downloadGeneratedFile } from "./files.js";
import {
  makeIntersectionMarkingsLayer,
  makeLaneMarkingsLayer,
  makePlainGeoJsonLayer,
  lanePolygonStyle,
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
    this.currentWaysLayer = null;
    this.editedWays = new Set();

    const importButton = document.getElementById("import-view");
    importButton.onclick = async () => {
      if (this.map.getZoom() < 15) {
        window.alert("Zoom in more to import");
        return;
      }

      await this.importView(importButton);
    };

    document.getElementById("osc").onclick = () => {
      this.downloadOsc();
    };
  }

  async importView(importButton) {
    // Reset state
    this.currentWay = null;
    if (this.currentWaysLayer) {
      this.currentWaysLayer.remove();
    }
    this.currentWaysLayer = null;
    this.editedWays = new Set();
    document.getElementById("edits-list").innerText = "0 edits";
    document.getElementById("tags").innerHTML = "";

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
    this.layers.push(
      makeIntersectionMarkingsLayer(
        this.network.toIntersectionMarkingsGeojson()
      )
    );

    for (const layer of this.layers) {
      layer.addTo(this.map);
    }
  }

  editWay(id) {
    this.currentWay = id;
    if (this.currentWaysLayer) {
      this.currentWaysLayer.remove();
    }
    this.currentWaysLayer = L.geoJSON(
      JSON.parse(this.network.getGeometryForWay(id)),
      {
        style: (feature) => {
          return { stroke: false, fill: true, color: "red", opacity: 0.5 };
        },
      }
    ).addTo(this.map);

    var html = `<a href="http://openstreetmap.org/way/${id}" target="_blank">Way ${id}</a><br/>`;
    html += `<table><tbody id="tags-table">`;

    const tags = JSON.parse(this.network.getOsmTagsForWay(id));
    // Note IDs initially use indices, but as the user adds and deletes rows, the indices get out of sync. That's not important; as long as the IDs are unique, it's fine.
    var idx = 0;
    for (let key in tags) {
      const value = tags[key];
      html += `<tr id="row-${idx}">`;
      html += `<td><input type="text" value="${key}"></td>`;
      html += `<td><input type="text" value="${value}"></td>`;
      html += `<td><button type="button" id="del-${idx}">Delete</button></td>`;
      html += `</tr>`;
      idx++;
    }
    html += `</tbody></table>`;
    html += `<button type="button" id="add-row">Add new row</button>`;

    html += `<button type="button" id="recalculate">Recalculate</button>`;

    const div = document.getElementById("tags");
    div.innerHTML = html;

    document.getElementById("recalculate").onclick = () => {
      this.recalculateWay();
    };

    document.getElementById("add-row").onclick = () => {
      idx++;

      const row = document.createElement("tr");
      row.id = `row-${idx}`;
      row.innerHTML = `<td><input type="text"></td><td><input type="text"></td><td><button type="button" id="del-${idx}">Delete</button></td>`;
      document.getElementById("tags-table").appendChild(row);

      document.getElementById(`del-${idx}`).onclick = () => {
        document.getElementById(`row-${idx}`).remove();
      };
    };

    for (var i = 0; i < idx; i++) {
      // Ahh Javascript...
      const ii = i;
      document.getElementById(`del-${ii}`).onclick = () => {
        console.log(`lets remove row-${ii}`);
        document.getElementById(`row-${ii}`).remove();
      };
    }
  }

  recalculateWay() {
    const tags = {};
    const table = document.getElementById("tags-table");
    for (var i = 0, row; (row = table.rows[i]); i++) {
      var key = null;
      for (var j = 0, cell; (cell = row.cells[j]); j++) {
        if (cell.firstChild instanceof HTMLInputElement) {
          if (key) {
            // Skip empty keys or values
            if (key && cell.firstChild.value) {
              tags[key] = cell.firstChild.value;
            }
          } else {
            key = cell.firstChild.value;
          }
        }
      }
    }

    console.log(`Recalculate with ${JSON.stringify(tags)}`);
    this.network.overwriteOsmTagsForWay(this.currentWay, JSON.stringify(tags));
    this.rerenderAll();

    this.editedWays.add(this.currentWay);
    document.getElementById(
      "edits-list"
    ).innerText = `${this.editedWays.size} edits`;
  }

  downloadOsc() {
    var contents = `<osmChange version="0.6" generator="osm2streets">\n`;
    contents += `<create/>\n`;
    contents += `<modify>\n`;
    for (const id of this.editedWays) {
      contents += this.network.wayToXml(id);
      contents += "\n";
    }
    contents += `</modify>\n`;
    contents += `</osmChange>`;

    downloadGeneratedFile("lane_edits.osc", contents);
  }
}
