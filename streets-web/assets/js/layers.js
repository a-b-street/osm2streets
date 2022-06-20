export const makeJsonLayer = (text) => {
    return new L.geoJSON(JSON.parse(text), { style: { color: '#f55' }});
};

export const makeOsmLayer = (text) => {
    return new L.OSM.DataLayer(new DOMParser().parseFromString(text, 'application/xml'), { style: { color: '#5f5' }});
};

export const makeDotLayer = async (text, { bounds }) => {
    return new Promise((resolve, reject) => {
        const graph = d3.select("#road-network")
            .graphviz({
                zoom: false,
            })
            .on('end', () => {
                const svg = graph._selection.node().firstElementChild; // assume first child for now
                if (!svg) console.error('no svg element came about from the render')
                resolve(new L.svgOverlay(svg, bounds, {
                    opacity: 0.3,
                    interactive: true
                }));
            })
            .dot(text).render();
    })
}

export const layerMakers = { json: makeJsonLayer, osm: makeOsmLayer, dot: makeDotLayer };