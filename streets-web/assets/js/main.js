import { makeDropHandler, handleDragOver } from "./files.js";

const cb = (map) => {
    const container = map.getContainer();
    container.ondrop = makeDropHandler(map);
    container.ondragover = handleDragOver;
    console.info("Ready for drops on the map!", container);
}

// Smuggle a reference to the created map, so I can work with it in JS land.
const LM = L.Map;
window.maps = [];
L.Map = function(x, opts = {
    maxZoom: 21,
}, ...args) {
    const m = new LM(x, opts, ...args)
    window.maps.push(m);
    setTimeout(() => cb(m), 0);
    return m;
}
