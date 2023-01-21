import { downloadGeneratedFile } from "./files.js";
import {
  makeIntersectionMarkingsLayer,
  makeLaneMarkingsLayer,
  makePlainGeoJsonLayer,
  lanePolygonStyle,
} from "./layers.js";
import { setupLeafletMap } from "./leaflet.js";
import init, { JsStreetNetwork } from "osm2streets-js";

await init();

export class LaneEditor {
  constructor(mapContainer) {
    this.map = setupLeafletMap(mapContainer);
    this.map.on({
      click: () => {
        this.resetEditView();
      },
    });
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
      downloadGeneratedFile("lane_edits.osc", this.toOsc(null));
    };
    document.getElementById("upload").onclick = async () => {
      try {
        let url = await this.uploadChangeset();
        console.log(`Success! Check out ${url}`);
        window.alert(`Success! Check out ${url}`);
      } catch (err) {
        window.alert(`Uploading changeset failed: ${err}`);
      }
      // TODO Clear state
    };
  }

  resetEditView() {
    this.currentWay = null;
    if (this.currentWaysLayer) {
      this.currentWaysLayer.remove();
    }
    this.currentWaysLayer = null;
    document.getElementById("tags").innerHTML = "";
  }

  async importView(importButton) {
    this.resetEditView();
    this.editedWays = new Set();
    document.getElementById("edits-list").innerText = "0 edits";

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

      const clipPts = "";
      this.network = new JsStreetNetwork(osmXML, clipPts, {
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
              // Prevent map.click from being triggered and resetting the edit view
              L.DomEvent.stop(ev);
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
    this.resetEditView();
    this.currentWay = id;
    this.currentWaysLayer = L.geoJSON(
      JSON.parse(this.network.getGeometryForWay(id)),
      {
        style: (feature) => {
          return {
            stroke: true,
            fill: true,
            color: "red",
            weight: 1,
            fillOpacity: 0.3,
          };
        },
        interactive: false,
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

  toOsc(changesetId) {
    var contents = `<osmChange version="0.6" generator="osm2streets">\n`;
    contents += `<create/>\n`;
    contents += `<modify>\n`;
    for (const id of this.editedWays) {
      contents += this.network.wayToXml(id, changesetId);
      contents += "\n";
    }
    contents += `</modify>\n`;
    contents += `</osmChange>`;
    return contents;
  }

  async uploadChangeset() {
    let api = "https://master.apis.dev.openstreetmap.org";

    // Create the changeset
    let changesetBody = `<osm><changeset><tag k="created_by" v="osm2streets StreetExplorer"/>`;
    let comment = document.getElementById("comment").value;
    if (comment) {
      // TODO Encode
      changesetBody += `<tag k="comment" v="${comment}"/>`;
    }
    changesetBody += `</changeset></osm>`;

    let resp1 = await fetch(`${api}/0.6/changeset/create`, {
      method: "PUT",
      headers: {
        "content-type": "application/xml; charset=utf-8",
      },
      body: changesetBody,
    });
    if (!resp1.ok) {
      throw new Error(`Creating changeset failed: ${await resp1.text()}`);
    }
    let changesetId = await resp1.text();

    // Upload the OSC file
    let resp2 = await fetch(`${api}/0.6/changeset/${changesetId}/upload`, {
      method: "POST",
      headers: {
        "content-type": "application/xml; charset=utf-8",
      },
      body: this.toOsc(changesetId),
    });
    if (!resp2.ok) {
      throw new Error(
        `Uploading OSC to changeset failed: ${await resp2.text()}`
      );
    }

    // Close the changeset
    let resp3 = await fetch(`${api}/0.6/changeset/${changesetId}/close`, {
      method: "PUT",
    });
    if (!resp3.ok) {
      throw new Error(`Closing changeset failed: ${await resp3.text()}`);
    }

    return `${api}/changeset/${changesetId}`;
  }
}
