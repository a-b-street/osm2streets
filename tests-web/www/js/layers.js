export const makeJsonLayer = (text) => {
  return new L.geoJSON(JSON.parse(text), { style: styleGeoJson });
};

const intersectionColours = {
  undefined: '#666', // for default tarmac
  Connection: '#666',
  MultiConnection: '#669',
  Merge: '#969',
  Crossing: '#966',
  Terminus: '#999',
  MapEdge: '#696',
}

const styleGeoJson = (feature) => {
  if (feature.geometry.type === 'Polygon') {
    return {
      color: intersectionColours[feature.properties?.complexity],
        weight: 1,
        fillOpacity: 0.7,
      };
  }
  return { color: '#f55' };
}

export const makeDetailedGeoJsonLayer = (text) => {
  return new L.geoJSON(JSON.parse(text), {
    style: function (feature) {
      switch (feature.properties.type) {
        case "road polygon":
          return {
            fill: true,
            fillColor: "black",
            fillOpacity: 0.9,
            stroke: false,
          };
        case "intersection polygon":
          return {
            fill: true,
            fillColor: "grey",
            fillOpacity: 0.9,
            stroke: false,
          };
        case "lane separator":
          return { color: "white", dashArray: "4" };
      }
    },
  });
};

export const makeOsmLayer = (text) => {
  return new L.OSM.DataLayer(
    new DOMParser().parseFromString(text, "application/xml"),
    { style: { color: "#5f5" } }
  );
};

export const makeDotLayer = async (text, { bounds }) => {
  return new Promise((resolve, reject) => {
    const graph = d3
      .select("#road-network")
      .graphviz({
        zoom: false,
      })
      .on("end", () => {
        const svg = graph._selection.node().firstElementChild; // assume first child for now
        if (!svg) console.error("no svg element came about from the render");
        resolve(
          new L.svgOverlay(svg, bounds, {
            opacity: 0.3,
            interactive: true,
          })
        );
      })
      .dot(text)
      .render();
  });
};

export const layerMakers = {
  json: makePlainGeoJsonLayer,
  osm: makeOsmLayer,
  dot: makeDotLayer,
};
