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
  makeDetailedGeoJsonLayer,
  makeDotLayer,
} from "./layers.js";
import init, { JsStreetNetwork } from "./osm2streets-js/osm2streets_js.js";

await init();

export class StreetExplorer {
  constructor(mapContainer) {
    this.map = setupLeafletMap(mapContainer);
    this.layerControl = L.control.layers({}, {}).addTo(this.map);
    this.currentTest = null;
    this.importSettings = {
      debugEachStep: false,
    };

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

  addLayer(name, layer) {
    layer.addTo(this.map);
    this.layerControl.addOverlay(layer, name);
    return layer;
  }

  removeLayer(layer) {
    this.map.removeLayer(layer);
    this.layerControl.removeLayer(layer);
  }

  async setCurrentTest(testMaker) {
    this.currentTest?.cleanup();
    this.currentTest = await testMaker(this);
    if (this.currentTest) {
      this.map.fitBounds(this.currentTest.bounds, { animate: false });
      this.currentTest.renderControls(document.getElementById("view-controls"));
    }
  }
}

class TestCase {
  constructor(app, name, osmXML, drivingSide, layers, bounds) {
    this.app = app;
    this.name = name;
    this.osmXML = osmXML;
    this.drivingSide = drivingSide;
    this.layers = layers;
    this.bounds = bounds;
  }

  cleanup() {
    for (const layer of this.layers) {
      this.app.removeLayer(layer);
    }
  }

  static async loadFromServer(app, name) {
    const prefix = `tests/${name}/`;
    const osmInput = await loadFile(prefix + "input.osm");
    const rawMap = loadFile(prefix + "raw_map.json");
    const network = loadFile(prefix + "road_network.dot");

    const rawMapLayer = makePlainGeoJsonLayer(await rawMap);
    const bounds = rawMapLayer.getBounds();

    var layers = [];
    layers.push(app.addLayer("Geometry", rawMapLayer));
    layers.push(app.addLayer("OSM", makeOsmLayer(osmInput)));
    layers.push(
      app.addLayer("Network", await makeDotLayer(await network, { bounds }))
    );

    const drivingSide = JSON.parse(await loadFile(prefix + "test.json"))[
      "driving_side"
    ];

    return new TestCase(app, name, osmInput, drivingSide, layers, bounds);
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

      const network = new JsStreetNetwork(osmInput, {
        driving_side: drivingSide,
        debug_each_step: app.importSettings.debugEachStep,
      });
      const rawMapLayer = makePlainGeoJsonLayer(network.toGeojsonPlain());
      const bounds = rawMapLayer.getBounds();

      var layers = [];
      layers.push(app.addLayer("Geometry", rawMapLayer));
      layers.push(
        app.addLayer(
          "Detailed geometry",
          makeDetailedGeoJsonLayer(network.toGeojsonDetailed())
        )
      );
      layers.push(app.addLayer("OSM", makeOsmLayer(osmInput)));
      // TODO ReferenceError: can't access lexical declaration 'graph' before initialization
      /*layers.push(
        app.addLayer(
          "Network",
          await makeDotLayer(network.toGraphviz(), { bounds })
        )
      );*/

      return new TestCase(app, null, osmInput, drivingSide, layers, bounds);
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
      button.onclick = async () => {
        // It doesn't make sense to ever reimport twice; that would only add redundant layers
        button.disabled = true;
        await this.reimport();
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

  async reimport() {
    try {
      const network = new JsStreetNetwork(this.osmXML, {
        driving_side: this.drivingSide,
        debug_each_step: this.app.importSettings.debugEachStep,
      });
      this.layers.push(
        this.app.addLayer(
          "Geometry (reimport)",
          makePlainGeoJsonLayer(network.toGeojsonPlain())
        )
      );
      this.layers.push(
        this.app.addLayer(
          "Detailed geometry (reimport)",
          makeDetailedGeoJsonLayer(network.toGeojsonDetailed())
        )
      );
      /*this.layers.push(
        this.app.addLayer(
          "Network (reimport)",
          await makeDotLayer(network.toGraphviz(), { bounds: this.bounds })
        )
      );*/
    } catch (err) {
      window.alert(`Reimport failed: ${err}`);
    }
  }
}

function setupLeafletMap(mapContainer) {
  // Make it smoother to zoom farther into the map
  const map = L.map(mapContainer, {
    maxZoom: 21,
    zoomSnap: 0.5,
    zoomDelta: 0.5,
    wheelPxPerZoomLevel: 120,
  }).setView([40.0, 10.0], 4);
  L.tileLayer("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", {
    maxNativeZoom: 18,
    maxZoom: 21,
    attribution: "© OpenStreetMap",
  }).addTo(map);
  new GeoSearch.GeoSearchControl({
    provider: new GeoSearch.OpenStreetMapProvider(),
    showMarker: false,
    autoClose: true,
  }).addTo(map);
  // TODO Add satellite layer
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

// TODO A Leaflet control isn't the right abstraction. We want a popup modal
// that totally blocks the map and other things. Things like disabling
// settingsButton to stop multiple of these controls is a total hack.
const SettingsControl = L.Control.extend({
  // TODO Centered would be great. https://github.com/Leaflet/Leaflet/issues/8358
  options: {
    position: "topleft",
  },
  onAdd: function (map) {
    var checkbox = L.DomUtil.create("input");
    checkbox.id = "debugEachStep";
    checkbox.type = "checkbox";
    if (this.options.app.importSettings.debugEachStep) {
      checkbox.checked = true;
    }
    checkbox.onclick = () => {
      this.options.app.importSettings.debugEachStep = checkbox.checked;
    };

    var label = L.DomUtil.create("label");
    label.for = "debugEachStep";
    label.innerText = "Debug each transformation step";

    const button = L.DomUtil.create("button");
    button.type = "button";
    button.innerHTML = "Confirm";
    button.onclick = () => {
      this.remove();
      document.getElementById("settingsButton").disabled = false;
    };

    var group = L.DomUtil.create("div");
    group.style = "background: black; padding: 10px;";
    group.appendChild(checkbox);
    group.appendChild(label);
    group.appendChild(button);

    return group;
  },
});

const makeSettingsControl = function (app) {
  return new SettingsControl({ app: app });
};
