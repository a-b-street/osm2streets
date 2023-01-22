import L from "leaflet";
import "@geoman-io/leaflet-geoman-free";
import "@geoman-io/leaflet-geoman-free/dist/leaflet-geoman.css";

import { downloadGeneratedFile, loadFile } from "./files.js";
import { loadTests } from "./tests.js";
import {
  makeDebugLayer,
  makeLaneMarkingsLayer,
  makeLanePolygonLayer,
  makeIntersectionMarkingsLayer,
  makeOsmLayer,
  makePlainGeoJsonLayer,
  makeBoundaryLayer,
} from "./layers.js";
import {
  LayerGroup,
  makeLayerControl,
  SequentialLayerGroup,
} from "./controls.js";
import { setupLeafletMap } from "./leaflet.js";
import init, { JsStreetNetwork } from "osm2streets-js";

await init();

export class StreetExplorer {
  constructor(mapContainer) {
    this.map = setupLeafletMap(mapContainer);
    this.currentTest = null;
    this.layers = makeLayerControl(this).addTo(this.map);
    this.settingsControl = null;
    this.dynamicMovementLayer = null;

    // Add all tests to the sidebar
    loadTests();
  }

  static async create(mapContainer) {
    const app = new StreetExplorer(mapContainer);

    // Possibly load a test from the URL
    const q = new URLSearchParams(window.location.search);
    if (q.has("test")) {
      const test = q.get("test");
      console.info(`Loading test ${test} from URL.`);
      await app.setCurrentTest((app) => TestCase.loadFromServer(app, test));
    }

    // Wire up the import button
    const importButton = document.getElementById("import-view");
    if (importButton) {
      importButton.onclick = async () => {
        if (app.map.getZoom() < 15) {
          window.alert("Zoom in more to import");
          return;
        }

        await app.setCurrentTest((app) => {
          const boundary = mapBoundsToGeojson(app.map);
          return TestCase.importBoundary(app, importButton, boundary);
        });
      };
    }

    // Set up controls for importing via rectangle and polygon boundaries.
    // TODO This is flaky. What do we do when geoman's magic init doesn't happen by now?
    if (app.map.pm && app.currentTest == null) {
      app.map.pm.addControls({
        position: "bottomright",
        editControls: false,
        drawMarker: false,
        drawCircleMarker: false,
        drawPolyline: false,
        drawCircle: false,
        drawText: false,
      });
      // TODO Disable snapping with OTHER layers, but do snap to drawn points.
      app.map.on("pm:create", async (e) => {
        // Reset the geoman drawing layer
        app.map.eachLayer((layer) => {
          // This is apparently how geoman layers are identified
          if (layer._path != null) {
            layer.remove();
          }
        });

        await app.setCurrentTest((app) => {
          const boundary = geomanToGeojson(e.layer.getLatLngs()[0]);
          return TestCase.importBoundary(app, importButton, boundary);
        });
      });
    }

    return app;
  }

  getImportSettings() {
    try {
      const data = new FormData(document.getElementById("import-settings"));
      return Object.fromEntries(data);
    } catch (e) {
      console.warn("failed to get import settings from the DOM:", e);
      return {};
    }
  }

  async setCurrentTest(testMaker) {
    // Clear all layers
    this.layers.removeGroups((name) => true);

    this.currentTest = await testMaker(this);
    if (this.currentTest) {
      this.map.fitBounds(this.currentTest.bounds, { animate: false });
      document.getElementById("test-list").value =
        this.currentTest.name || "dynamic";
      this.currentTest.renderControls(document.getElementById("view-controls"));
    }
  }
}

class TestCase {
  constructor(app, name, osmXML, bounds, boundary) {
    this.app = app;
    this.name = name;
    this.osmXML = osmXML;
    this.bounds = bounds;
    this.boundary = boundary;
  }

  static async loadFromServer(app, name) {
    const prefix = `tests/${name}/`;
    const osmInput = await loadFile(prefix + "input.osm");
    const geometry = await loadFile(prefix + "geometry.json");
    const boundary = JSON.parse(await loadFile(prefix + "boundary.json"));

    const geometryLayer = makePlainGeoJsonLayer(geometry);
    const bounds = geometryLayer.getBounds();

    var group = new LayerGroup("built-in test case", app.map);
    group.addLayer("Boundary", makeBoundaryLayer(boundary));
    group.addLayer("OSM", makeOsmLayer(osmInput), { enabled: false });
    group.addLayer("Geometry", geometryLayer);
    app.layers.addGroup(group);

    return new TestCase(app, name, osmInput, bounds, boundary);
  }

  static async importBoundary(app, importButton, boundaryGeojson) {
    // Construct a query to extract all XML data in the polygon clip. See
    // https://wiki.openstreetmap.org/wiki/Overpass_API/Overpass_QL
    var filter = 'poly:"';
    for (const [lng, lat] of boundaryGeojson.features[0].geometry
      .coordinates[0]) {
      filter += `${lat} ${lng} `;
    }
    filter = filter.slice(0, -1) + '"';
    const query = `(nwr(${filter}); node(w)->.x; <;); out meta;`;
    const url = `https://overpass-api.de/api/interpreter?data=${query}`;
    console.log(`Fetching from overpass: ${url}`);

    importButton.innerText = "Downloading from Overpass...";
    // Prevent this function from happening twice in a row. It could also maybe
    // be nice to allow cancellation.
    importButton.disabled = true;

    try {
      const resp = await fetch(url);
      const osmInput = await resp.text();

      importButton.innerText = "Importing OSM data...";

      importOSM("Imported area", app, osmInput, true, boundaryGeojson);
      const bounds = app.layers
        .getLayer("Imported area", "Geometry")
        .getData()
        .getBounds();

      // Remove the test case from the URL, if needed
      const fixURL = new URL(window.location);
      fixURL.searchParams.delete("test");
      window.history.pushState({}, "", fixURL);

      return new TestCase(app, null, osmInput, bounds, boundaryGeojson);
    } catch (err) {
      window.alert(`Import failed: ${err}`);
      // There won't be a currentTest
      return null;
    } finally {
      // Make the button clickable again
      importButton.innerText = "Import current view";
      importButton.disabled = false;
    }
  }

  renderControls(container) {
    container.innerHTML = "";
    if (this.name) {
      const button1 = container.appendChild(document.createElement("button"));
      button1.type = "button";
      button1.innerHTML = "Generate Details";
      button1.onclick = () => {
        // First remove all existing groups except for the original one
        this.app.layers.removeGroups((name) => name != "built-in test case");
        // Then disable the original group. Seeing dueling geometry isn't a good default.
        this.app.layers.getGroup("built-in test case").setEnabled(false);

        importOSM("Details", this.app, this.osmXML, false, this.boundary);
      };

      const button2 = container.appendChild(document.createElement("button"));
      button2.type = "button";
      button2.innerHTML = "Update OSM data";
      const boundary = this.boundary;
      button2.onclick = async () => {
        await this.app.setCurrentTest((app) => {
          return TestCase.importBoundary(app, button2, boundary);
        });
      };
    }

    const button1 = container.appendChild(document.createElement("button"));
    button1.type = "button";
    button1.innerHTML = "Download osm.xml";
    button1.onclick = () =>
      downloadGeneratedFile(`${this.name || "new"}.osm.xml`, this.osmXML);

    const button2 = container.appendChild(document.createElement("button"));
    button2.type = "button";
    button2.innerHTML = "Reset view";
    button2.onclick = () => {
      this.app.map.fitBounds(this.bounds, { animate: false });
    };
  }
}

function importOSM(groupName, app, osmXML, addOSMLayer, boundaryGeojson) {
  try {
    const importSettings = app.getImportSettings();
    const network = new JsStreetNetwork(
      osmXML,
      JSON.stringify(boundaryGeojson),
      {
        debug_each_step: !!importSettings.debugEachStep,
        dual_carriageway_experiment: !!importSettings.dualCarriagewayExperiment,
        cycletrack_snapping_experiment:
          !!importSettings.cycletrackSnappingExperiment,
        inferred_sidewalks: importSettings.sidewalks === "infer",
        osm2lanes: !!importSettings.osm2lanes,
      }
    );
    var group = new LayerGroup(groupName, app.map);
    group.addLayer("Boundary", makeBoundaryLayer(boundaryGeojson));
    if (addOSMLayer) {
      // TODO This is crashing at #18.37/-33.88071/151.21693 for unknown reasons. Don't break everything.
      try {
        group.addLayer("OSM", makeOsmLayer(osmXML), { enabled: false });
      } catch (err) {
        window.alert(`Warning: OSM layer not added: ${err}`);
      }
    }
    group.addLayer("Geometry", makePlainGeoJsonLayer(network.toGeojsonPlain()));
    group.addLayer(
      "Lane polygons",
      makeLanePolygonLayer(network, app.dynamicMovementLayer, app.map)
    );
    group.addLayer(
      "Lane markings",
      makeLaneMarkingsLayer(network.toLaneMarkingsGeojson())
    );
    group.addLayer(
      "Intersection markings",
      makeIntersectionMarkingsLayer(network.toIntersectionMarkingsGeojson())
    );
    group.addLazyLayer("Debug road ordering", () =>
      makeDebugLayer(network.debugClockwiseOrderingGeojson())
    );

    const numDebugSteps = network.getDebugSteps().length;
    // This enables all layers within the group. We don't want to do that for the OSM layer. So only disable if we're debugging.
    if (numDebugSteps > 0) {
      group.setEnabled(false);
    }
    app.layers.addGroup(group);

    var debugGroups = [];
    var i = 0;
    for (const step of network.getDebugSteps()) {
      i++;
      var group = new LayerGroup(`Step ${i}: ${step.getLabel()}`, app.map);
      group.addLazyLayer("Geometry", () =>
        makePlainGeoJsonLayer(step.getNetwork().toGeojsonPlain())
      );
      group.addLazyLayer("Lane polygons", () =>
        makeLanePolygonLayer(
          step.getNetwork(),
          app.dynamicMovementLayer,
          app.map
        )
      );
      group.addLazyLayer("Lane markings", () =>
        makeLaneMarkingsLayer(step.getNetwork().toLaneMarkingsGeojson())
      );
      // TODO Can we disable by default in a group? This one is very noisy, but
      // could be useful to inspect
      /*group.addLazyLayer("Debug road ordering", () =>
        makeDebugLayer(step.getNetwork().debugClockwiseOrderingGeojson())
      );*/

      const debugGeojson = step.toDebugGeojson();
      if (debugGeojson) {
        group.addLazyLayer("Debug", () => makeDebugLayer(debugGeojson));
      }
      debugGroups.push(group);
    }
    if (debugGroups.length != 0) {
      app.layers.addGroup(
        new SequentialLayerGroup("transformation steps", debugGroups)
      );
    }
  } catch (err) {
    window.alert(`Import failed: ${err}`);
  }
}

function latLngToGeojson(pt) {
  return [pt.lng, pt.lat];
}

function geomanToGeojson(points) {
  points.push(points[0]);
  return {
    type: "FeatureCollection",
    features: [
      {
        type: "Feature",
        properties: {},
        geometry: {
          coordinates: [points.map(latLngToGeojson)],
          type: "Polygon",
        },
      },
    ],
  };
}

// Turn the current viewport into a rectangular boundary
function mapBoundsToGeojson(map) {
  const b = map.getBounds();
  return {
    type: "FeatureCollection",
    features: [
      {
        type: "Feature",
        properties: {},
        geometry: {
          coordinates: [
            [
              latLngToGeojson(b.getSouthWest()),
              latLngToGeojson(b.getNorthWest()),
              latLngToGeojson(b.getNorthEast()),
              latLngToGeojson(b.getSouthEast()),
              latLngToGeojson(b.getSouthWest()),
            ],
          ],
          type: "Polygon",
        },
      },
    ],
  };
}
