import { makeDropHandler, makeLinkHandler, handleDragOver } from "./files.js";
import { makeOpenTest, loadTests } from "./tests.js";

const useMap = (map) => {
    const container = map.getContainer();
    container.ondrop = makeDropHandler(map);
    container.ondragover = handleDragOver;

    map.loadLink = makeLinkHandler(map);
    map.openTest = makeOpenTest(map)
    console.info("New map created! File drops enabled.", container);


    // Here we read the test name from the URL.
    const q = new URLSearchParams(window.location.search);
    if (q.has('test')) {
        const test = q.get('test');
        console.info("Loading test " + test + " from URL.");
        map.openTest(test);
    }

    loadTests();
}

// Smuggle a reference to the created map, so I can work with it in JS land.
console.debug("Listening in on map creations...");
const LM = L.Map;
window.maps = [];
L.Map = function(x, opts = {
    maxZoom: 21,
}, ...args) {
    const m = new LM(x, opts, ...args)
    window.maps.push(m);
    setTimeout(() => useMap(m), 0);
    return m;
}

// Settings for tile layers.
const LL = L.TileLayer;
L.TileLayer = function(x, opts = {
    maxNativeZoom: 18,
    maxZoom: 21,
}, ...args) {
    return new LL(x, opts, ...args);
}

