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
    // Current tests (for your convenience):
    // arizona_highways
    // aurora_sausage_link
    // borough_sausage_links
    // bristol_contraflow_cycleway
    // bristol_sausage_links
    // i5_exit_ramp
    // jelly_bean_roundabout
    // kingsway_junction
    // lib.rs
    // montlake_roundabout
    // perth_stretched_lights
    // seattle_slip_lane
    // seattle_triangle
    // service_road_loop
    // taipei
    // tempe_light_rail
    // tempe_split
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

