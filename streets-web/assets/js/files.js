import { openDot } from './dot.js';

const fileOpeners = {
    json(text) { return new L.geoJSON(JSON.parse(text)); },
    osm(text) { return new L.OSM.DataLayer(new DOMParser().parseFromString(text, 'application/xml')); },
    async dot(text, { bounds }) {
        return new Promise((resolve, reject) => {
            const graph = d3.select("#road-network")
                .graphviz({
                    zoom: false,
                })
                .on('end', () => {
                    const svg = graph._selection.node().firstElementChild; // assume first child for now
                    if (!svg) console.error('no svg element came about from the render')
                    resolve(new L.svgOverlay(svg, bounds, {
                        opacity: 0.7,
                        interactive: true
                    }));
                })
                .dot(text).render();
        })
    }
}

const makeOpenFile = (map) => async (text, name) => {
    const dotPos = name?.lastIndexOf('.') ?? -1;
    const layer = await fileOpeners[name.substring(dotPos + 1)]?.(text, { bounds: map.getBounds()});
    if (layer) {
        map.addLayer(layer);
        map.flyToBounds(layer.getBounds());
    }
}

export const makeLinkHandler = (map) => {
    const openFile = makeOpenFile(map);
    return (link) => {
        return fetch(link).then(body => openFile(body.text(), body.url));
    }
};

export const handleDragOver = (dragEvent) => {
    dragEvent.preventDefault(); // tells the browser that we're handling this drag/drop.
};

export const makeDropHandler = (map) => {
    const openFile = makeOpenFile(map);
    return (dropEvent) => {
        // We are handling this event. prevent it from being openend.
        dropEvent.preventDefault();

        console.debug({ dropped: dropEvent.dataTransfer });
        forEachFile(dropEvent.dataTransfer, (file, i) => {
            file.text().then(t => openFile(t, file.name));
        })

    }
};

const forEachFile = (dt, f) => {
    let c = 0;
    if (dt.items) {
        // Use DataTransferItemList interface to access the file(s)
        for (let i = 0; i < dt.items.length; i++) {
            // Call f for each file that we find.
            if (dt.items[i].kind === 'file') {
                f(dt.items[i].getAsFile(), c++);
            }
        }
    } else {
        // Use DataTransfer interface to access the file(s)
        for (let i = 0; i < dt.files.length; i++) {
            f(dt.files[i], c++);
        }
    }
}
