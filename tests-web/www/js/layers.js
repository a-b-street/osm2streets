export const makePlainGeoJsonLayer = (text) => {
  const intersectionColours = {
    undefined: "#666", // for default tarmac
    Connection: "#666",
    MultiConnection: "#669",
    Merge: "#969",
    Crossing: "#966",
    Terminus: "#999",
    MapEdge: "#696",
  };

  return new L.geoJSON(JSON.parse(text), {
    style: function (feature) {
      if (feature.geometry.type === "Polygon") {
        return {
          color: intersectionColours[feature.properties?.complexity],
          weight: 1,
          fillOpacity: 0.7,
        };
      }
      return { color: "#f55" };
    },
  });
};

export const makeLanePolygonLayer = (text) => {
  // These could change per locale
  const colors = {
    Driving: "black",
    Parking: "#333333",
    Sidewalk: "#CCCCCC",
    Shoulder: "#CCCCCC",
    Biking: "#0F7D4B",
    Bus: "#BE4A4C",
    SharedLeftTurn: "black",
    Construction: "#FF6D00",
    LightRail: "#844204",
    // Skip Buffer -- those need different icons depending on the type
  };

  return new L.geoJSON(JSON.parse(text), {
    style: function (feature) {
      return {
        fill: true,
        fillColor: colors[feature.properties.type] || "red",
        fillOpacity: 0.9,
        stroke: false,
      };
    },
  });
};

export const makeLaneMarkingsLayer = (text) => {
  // These could change per locale
  const colors = {
    "center line": "yellow",
    "lane separator": "white",
  };

  return new L.geoJSON(JSON.parse(text), {
    style: function (feature) {
      return {
        fill: true,
        fillColor: colors[feature.properties.type],
        fillOpacity: 0.9,
        stroke: false,
      };
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

export const makeDebugLayer = (text) => {
  return new L.geoJSON(JSON.parse(text), {
    onEachFeature: function (feature, layer) {
      layer.bindTooltip(feature.properties.label, { permanent: true });
    },
  });
};

export const layerMakers = {
  json: makePlainGeoJsonLayer,
  osm: makeOsmLayer,
  dot: makeDotLayer,
};