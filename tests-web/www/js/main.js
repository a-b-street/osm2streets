import { makeDropHandler, makeLinkHandler, handleDragOver } from "./files.js";
import { makeImportCurrentView } from "./import.js";
import { makeOpenTest, loadTests } from "./tests.js";

const useMap = (map) => {
  const container = map.getContainer();
  container.ondrop = makeDropHandler(map);
  container.ondragover = handleDragOver;

  map.loadLink = makeLinkHandler(map);
  map.openTest = makeOpenTest(map);
  console.info("New map created! File drops enabled.", container);

  // Here we read the test name from the URL.
  const q = new URLSearchParams(window.location.search);
  if (q.has("test")) {
    const test = q.get("test");
    console.info("Loading test " + test + " from URL.");
    map.openTest(test);
  }

  loadTests(map);

  makeImportCurrentView(map, document.getElementById("import-view"));
};

// Initialize the map
const map = L.map("map", { maxZoom: 21 }).setView([40.0, 10.0], 4);
L.tileLayer("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", {
  maxNativeZoom: 18,
  maxZoom: 21,
  attribution: "© OpenStreetMap",
}).addTo(map);
useMap(map);
