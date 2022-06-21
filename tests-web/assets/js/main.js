import { makeDropHandler, makeLinkHandler, handleDragOver } from "./files.js";
import { makeOpenTest } from "./tests.js";

const useMap = (map) => {
    const container = map.getContainer();
    container.ondrop = makeDropHandler(map);
    container.ondragover = handleDragOver;

    map.loadLink = makeLinkHandler(map);

    console.info("New map created! File drops enabled.", container);

    console.info("opening a test, just for fun...");
    makeOpenTest(map)("aurora_sausage_link");
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

