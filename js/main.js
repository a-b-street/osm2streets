import {
  loadFile,
  makeDropHandler,
  makeLinkHandler,
  handleDragOver,
  downloadGeneratedFile,
} from "./files.js";
import { loadTests } from "./tests.js";
import {
  makeOsmLayer,
  makePlainGeoJsonLayer,
  makeLanePolygonLayer,
  makeLaneMarkingsLayer,
  makeDotLayer,
  makeDebugLayer,
} from "./layers.js";
import {
  makeSettingsControl,
  makeLayerControl,
  LayerGroup,
  SequentialLayerGroup,
} from "./controls.js";
import init, { JsStreetNetwork } from "./osm2streets-js/osm2streets_js.js";

await init();

export class StreetExplorer {
  constructor(mapContainer) {
    this.map = setupLeafletMap(mapContainer);
    this.currentTest = null;
    this.importSettings = {
	    // TODO For quicker dev
      debugEachStep: true,
      dualCarriagewayExperiment: false,
      cycletrackSnappingExperiment: true,
    };
    this.layers = makeLayerControl(this).addTo(this.map);

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
    importButton.onclick = async () => {
      if (app.map.getZoom() < 15) {
        window.alert("Zoom in more to import");
        return;
      }

      await app.setCurrentTest((app) =>
        TestCase.importCurrentView(app, importButton)
      );
    };

    return app;
  }

  async setCurrentTest(testMaker) {
    // Clear all layers
    this.layers.removeGroups((name) => true);

    this.currentTest = await testMaker(this);
    if (this.currentTest) {
      this.map.fitBounds(this.currentTest.bounds, { animate: false });
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
    const rawMap = await loadFile(prefix + "raw_map.json");
    const network = await loadFile(prefix + "road_network.dot");

    const rawMapLayer = makePlainGeoJsonLayer(rawMap);
    const bounds = rawMapLayer.getBounds();

    var group = new LayerGroup("built-in test case", app.map);
    group.addLayer("Geometry", rawMapLayer);
    group.addLayer("OSM", makeOsmLayer(osmInput), { enabled: false });
    group.addLayer("Network", await makeDotLayer(network, { bounds }));
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

      // TODO Ask overpass
      const drivingSide = "Right";

      importOSM("Imported area", app, osmInput, drivingSide, true);
      const bounds = app.layers
        .getLayer("Imported area", "Geometry")
        .layer.getBounds();

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
      const title = container.appendChild(document.createElement("b"));
      title.innerText = `Currently showing ${this.name}`;

      const button = container.appendChild(document.createElement("button"));
      button.type = "button";
      button.innerHTML = "Reimport";
      button.onclick = () => {
        // First remove all existing groups except for the original one
        this.app.layers.removeGroups((name) => name != "built-in test case");

        importOSM("Reimport", this.app, this.osmXML, this.drivingSide, false);
      };

      const settings = container.appendChild(document.createElement("button"));
      settings.id = "settingsButton";
      settings.type = "button";
      settings.innerHTML = "(Settings)";
      settings.onclick = () => {
        settings.disabled = true;
        makeSettingsControl(app).addTo(app.map);
      };
    } else {
      container.innerHTML = `<b>Custom imported view</b>`;
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
    const network = new JsStreetNetwork(osmXML, {
      driving_side: drivingSide,
      debug_each_step: app.importSettings.debugEachStep,
      dual_carriageway_experiment: app.importSettings.dualCarriagewayExperiment,
      cycletrack_snapping_experiment:
        app.importSettings.cycletrackSnappingExperiment,
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
    // TODO Graphviz hits `ReferenceError: can't access lexical declaration 'graph' before initialization`

    const numDebugSteps = network.getDebugSteps().length;
    group.setEnabled(numDebugSteps == 0);
    app.layers.addGroup(group);

    var debugGroups = [];
    var i = 0;
    for (const step of network.getDebugSteps()) {
      i++;
      var group = new LayerGroup(`Step ${i}: ${step.getLabel()}`, app.map);
      group.addLayer(
        "Geometry",
        makePlainGeoJsonLayer(step.getNetwork().toGeojsonPlain())
      );
      /*group.addLayer(
        "Lane polygons",
        makeLanePolygonLayer(step.getNetwork().toLanePolygonsGeojson())
      );
      group.addLayer(
        "Lane markings",
        makeLaneMarkingsLayer(step.getNetwork().toLaneMarkingsGeojson())
      );*/

      const debugGeojson = step.toDebugGeojson();
      if (debugGeojson) {
        group.addLayer("Debug", makeDebugLayer(debugGeojson));
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
