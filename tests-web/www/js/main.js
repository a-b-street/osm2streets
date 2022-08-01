import {
  loadFile,
  makeDropHandler,
  makeLinkHandler,
  handleDragOver,
  downloadGeneratedFile,
} from "./files.js";
import { makeImportCurrentView } from "./import.js";
import { loadTests } from "./tests.js";
import {
  makeOsmLayer,
  makePlainGeoJsonLayer,
  makeDetailedGeoJsonLayer,
  makeDotLayer,
} from "./layers.js";
import { JsStreetNetwork } from "./osm2streets-js/osm2streets_js.js";

export class StreetExplorer {
  constructor(mapContainer) {
    this.map = setupLeafletMap(mapContainer);
    this.layerControl = L.control.layers({}, {}).addTo(this.map);
    this.currentTest = null;

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
    this.map.fitBounds(this.currentTest.bounds, { animate: false });
    this.currentTest.renderControls(document.getElementById("view-controls"));
  }
}

// design litmus test: can multiple of these exist at the app at the same time?
class TestCase {
  // null for freshly imported places
  constructor(app, name, osmXML, drivingSide, layers, bounds) {
    this.app = app;
    this.name = name;
    this.osmXML = osmXML;
    this.drivingSide = drivingSide;
    // TODO this is probably more organized at some point, not just a list
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

  static async importCurrentView(bounds) {}

  renderControls(container) {
    container.innerHTML = "";
    if (this.name) {
      const title = container.appendChild(document.createElement("b"));
      title.innerText = `Currently showing ${this.name}`;

      const button1 = container.appendChild(document.createElement("button"));
      button1.type = "button";
      button1.innerHTML = "Download osm.xml";
      button1.onclick = () =>
        downloadGeneratedFile(`${this.name}.osm.xml`, this.osmXML);

      const button2 = container.appendChild(document.createElement("button"));
      button2.type = "button";
      button2.innerHTML = "Reimport";
      button2.onclick = () => {
        button2.disabled = true;
        this.reimport();
      };
    } else {
      container.innerHTML = `<b>Custom imported view</b>`;
    }
  }

  reimport() {
    try {
      const network = new JsStreetNetwork(this.osmXML, {
        driving_side: this.drivingSide,
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
    } catch (err) {
      window.alert(`Reimport failed: ${err}`);
    }
  }
}

function setupLeafletMap(mapContainer) {
  const map = L.map(mapContainer, { maxZoom: 21 }).setView([40.0, 10.0], 4);
  L.tileLayer("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", {
    maxNativeZoom: 18,
    maxZoom: 21,
    attribution: "© OpenStreetMap",
  }).addTo(map);
  // Geocoder, satellite layers, etc
  return map;
}

// TODO Port stuff
const useMap = (map) => {
  const container = map.getContainer();
  container.ondrop = makeDropHandler(map);
  container.ondragover = handleDragOver;

  map.loadLink = makeLinkHandler(map);
  map.openTest = makeOpenTest(map);
  console.info("New map created! File drops enabled.", container);

  makeImportCurrentView(map, document.getElementById("import-view"));
};
