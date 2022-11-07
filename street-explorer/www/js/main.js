import {
  downloadGeneratedFile,
  handleDragOver,
  loadFile,
  makeDropHandler,
  makeLinkHandler,
} from "./files.js";
import { loadTests } from "./tests.js";
import {
  makeDebugLayer,
  makeDotLayer,
  makeLaneMarkingsLayer,
  makeLanePolygonLayer,
  makeOsmLayer,
  makePlainGeoJsonLayer,
} from "./layers.js";
import {
  LayerGroup,
  makeLayerControl,
  SequentialLayerGroup,
} from "./controls.js";
import init, { JsStreetNetwork } from "./osm2streets-js/osm2streets_js.js";

await init();

export class StreetExplorer {
  constructor(mapContainer) {
    this.map = setupLeafletMap(mapContainer);
    this.currentTest = null;
    this.layers = makeLayerControl(this).addTo(this.map);
    this.settingsControl = null;

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

        await app.setCurrentTest((app) =>
          TestCase.importCurrentView(app, importButton)
        );
      };
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
  constructor(app, name, osmXML, drivingSide, bounds) {
    this.app = app;
    this.name = name;
    this.osmXML = osmXML;
    this.drivingSide = drivingSide;
    this.bounds = bounds;
  }

  static async loadFromServer(app, name) {
    const prefix = `tests/${name}/`;
    const osmInput = await loadFile(prefix + "input.osm");
    const geometry = await loadFile(prefix + "geometry.json");
    const network = await loadFile(prefix + "road_network.dot");

    const geometryLayer = makePlainGeoJsonLayer(geometry);
    const bounds = geometryLayer.getBounds();

    var group = new LayerGroup("built-in test case", app.map);
    group.addLayer("OSM", makeOsmLayer(osmInput), { enabled: false });
    group.addLayer("Network", await makeDotLayer(network, { bounds }));
    group.addLayer("Geometry", geometryLayer);
    app.layers.addGroup(group);

    const drivingSide = JSON.parse(await loadFile(prefix + "test.json"))[
      "driving_side"
    ];

    return new TestCase(app, name, osmInput, drivingSide, bounds);
  }

  static async importCurrentView(app, importButton) {
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
      const osmInput = await resp.text();

      importButton.innerText = "Importing OSM data...";

      const drivingSide =
        app.getImportSettings().drivingSideForNewImports || "Right";

      importOSM("Imported area", app, osmInput, drivingSide, true);
      const bounds = app.layers
        .getLayer("Imported area", "Geometry")
        .getData()
        .getBounds();

      // Remove the test case from the URL, if needed
      const fixURL = new URL(window.location);
      fixURL.searchParams.delete("test");
      window.history.pushState({}, "", fixURL);

      return new TestCase(app, null, osmInput, drivingSide, bounds);
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
      const button = container.appendChild(document.createElement("button"));
      button.type = "button";
      button.innerHTML = "Generate Details";
      button.onclick = () => {
        // First remove all existing groups except for the original one
        this.app.layers.removeGroups((name) => name != "built-in test case");
        // Then disable the original group. Seeing dueling geometry isn't a good default.
        this.app.layers.getGroup("built-in test case").setEnabled(false);

        importOSM("Details", this.app, this.osmXML, this.drivingSide, false);
      };
    }

    const button = container.appendChild(document.createElement("button"));
    button.type = "button";
    button.innerHTML = "Download osm.xml";
    button.onclick = () =>
      downloadGeneratedFile(`${this.name || "new"}.osm.xml`, this.osmXML);
  }
}

function importOSM(groupName, app, osmXML, drivingSide, addOSMLayer) {
  try {
    const importSettings = app.getImportSettings();
    const network = new JsStreetNetwork(osmXML, {
      driving_side: drivingSide,
      debug_each_step: !!importSettings.debugEachStep,
      dual_carriageway_experiment: !!importSettings.dualCarriagewayExperiment,
      cycletrack_snapping_experiment:
        !!importSettings.cycletrackSnappingExperiment,
      inferred_sidewalks: importSettings.sidewalks === "infer",
      osm2lanes: !!importSettings.osm2lanes,
    });
    var group = new LayerGroup(groupName, app.map);
    if (addOSMLayer) {
      group.addLayer("OSM", makeOsmLayer(osmXML), { enabled: false });
    }
    group.addLayer("Geometry", makePlainGeoJsonLayer(network.toGeojsonPlain()));
    group.addLayer(
      "Lane polygons",
      makeLanePolygonLayer(network.toLanePolygonsGeojson())
    );
    group.addLayer(
      "Lane markings",
      makeLaneMarkingsLayer(network.toLaneMarkingsGeojson())
    );
    group.addLazyLayer("Debug road ordering", () =>
      makeDebugLayer(network.debugClockwiseOrderingGeojson())
    );
    // TODO Graphviz hits `ReferenceError: can't access lexical declaration 'graph' before initialization`
    group.addLayer("Snap", new L.geoJSON(JSON.parse(network.snap()), {
      style: function (feature) {
        return feature.properties;
      }
    }));

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
        makeLanePolygonLayer(step.getNetwork().toLanePolygonsGeojson())
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

function setupLeafletMap(mapContainer) {
  const osm = L.tileLayer(
    "https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png",
    {
      maxNativeZoom: 18,
      maxZoom: 21,
      attribution: "© OpenStreetMap",
    }
  );
  const arcgis = L.tileLayer(
    "https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}",
    {
      attribution: "© ArcGIS",
      maxNativeZoom: 18,
      maxZoom: 21,
    }
  );

  const map = L.map(mapContainer, {
    layers: [osm],
    maxZoom: 21,
    // Make it smoother to zoom farther into the map
    zoomSnap: 0.5,
    zoomDelta: 0.5,
    wheelPxPerZoomLevel: 120,
  }).setView([40.0, 10.0], 4);

  new GeoSearch.GeoSearchControl({
    provider: new GeoSearch.OpenStreetMapProvider(),
    showMarker: false,
    autoClose: true,
  }).addTo(map);

  new L.hash(map);

  L.control
    .layers({ OpenStreetMap: osm, ArcGIS: arcgis }, {}, { collapsed: false })
    .addTo(map);

  return map;
}

// TODO Unused. Preserve logic for dragging individual files as layers.
const useMap = (map) => {
  const container = map.getContainer();
  container.ondrop = makeDropHandler(map);
  container.ondragover = handleDragOver;

  map.loadLink = makeLinkHandler(map);
  map.openTest = makeOpenTest(map);
  console.info("New map created! File drops enabled.", container);
};
