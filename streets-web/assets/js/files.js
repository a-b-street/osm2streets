import { addGeojsonLayer, zoomToLayer } from './layers.js';

export const handleDragOver = (dragEvent) => {
    dragEvent.preventDefault(); // tells the browser that we're handling this drop.
};


export const makeDropHandler = (map) => (dropEvent) => {
    // We are handling this event. prevent it from being openend.
    dropEvent.preventDefault();

    console.debug({ dropped: dropEvent.dataTransfer });
    forEachFile(dropEvent.dataTransfer, (f, i) => {
        // depending on type, add a layer to the map or open a graphvis overlay.
        console.info('Got file', f);
        f.text()
            .then(t => addGeojsonLayer(map, JSON.parse(t)))
            .then(l => zoomToLayer(map, l));
    })

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
