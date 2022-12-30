import L from "leaflet";
import "leaflet-osm";

export const makePlainGeoJsonLayer = (text) => {
  // TODO Update for new types
  const intersectionColours = {
    Connection: "#666",
    Intersection: "#966",
    Terminus: "#999",
    MapEdge: "#696",
  };

  return new L.geoJSON(JSON.parse(text), {
    style: function (feature) {
      if (feature.properties.type == "intersection") {
        return {
          color:
            intersectionColours[feature.properties.intersection_kind] || "#666",
          weight: 1,
          fillOpacity: 0.7,
        };
      }
      return { color: "#f55" };
    },
    onEachFeature: function (feature, layer) {
      if (feature.properties.type != "intersection") {
        return;
      }
      layer.on({
        mouseover: function (ev) {
          const layer = ev.target;
          layer.setStyle({
            fillOpacity: 0.5,
          });
        },
        mouseout: function (ev) {
          layer.setStyle({
            fillOpacity: 0.7,
          });
        },
      });

      const nodes = feature.properties.osm_node_ids;
      delete feature.properties.osm_node_ids;
      var popup = `<pre>${JSON.stringify(feature.properties, null, 2)}</pre>`;
      popup += `<br/>OSM nodes: `;
      for (const id of nodes) {
        popup += `<a href="https://www.openstreetmap.org/node/${id}" target="_blank">${id}</a>, `;
      }
      popup = popup.slice(0, -2);
      layer.bindPopup(popup);
    },
  });
};

export const lanePolygonStyle = (feature) => {
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
    // This is the only type used currently
    "Buffer(Planters)": "#555555",
  };

  if (feature.properties.type == "Footway") {
    return {
      fill: true,
      fillColor: "#DDDDE8",
      stroke: true,
      color: "black",
      dashArray: "5,10",
    };
  }
  if (feature.properties.type == "SharedUse") {
    return {
      fill: true,
      fillColor: "#E5E1BB",
      stroke: true,
      color: "black",
      dashArray: "5,10",
    };
  }

  return {
    fill: true,
    fillColor: colors[feature.properties.type] || "red",
    fillOpacity: 0.9,
    stroke: false,
  };
};

export const makeLanePolygonLayer = (network, dynamicMovementLayer, map) => {
  return new L.geoJSON(JSON.parse(network.toLanePolygonsGeojson()), {
    style: lanePolygonStyle,
    onEachFeature: function (feature, layer) {
      layer.on({
        mouseover: function (ev) {
          const layer = ev.target;
          layer.setStyle({
            fillOpacity: 0.5,
          });

          if (dynamicMovementLayer) {
            map.removeLayer(dynamicMovementLayer);
          }
          const movements = network.debugMovementsFromLaneGeojson(
            feature.properties.road,
            feature.properties.index
          );
          dynamicMovementLayer = L.geoJSON(JSON.parse(movements));
          dynamicMovementLayer.addTo(map);
        },
        mouseout: function (ev) {
          layer.setStyle({
            fillOpacity: 0.9,
          });

          if (dynamicMovementLayer) {
            map.removeLayer(dynamicMovementLayer);
          }
        },
      });

      const ways = feature.properties.osm_way_ids;
      delete feature.properties.osm_way_ids;
      var popup = `<pre>${JSON.stringify(feature.properties, null, 2)}</pre>`;
      popup += `<br/>OSM ways: `;
      for (const id of ways) {
        popup += `<a href="https://www.openstreetmap.org/way/${id}" target="_blank">${id}</a>, `;
      }
      popup = popup.slice(0, -2);
      layer.bindPopup(popup);
    },
  });
};

export const makeLaneMarkingsLayer = (text) => {
  // These could change per locale
  const colors = {
    "center line": "yellow",
    "lane separator": "white",
    "lane arrow": "white",
    "buffer edge": "white",
    "buffer stripe": "white",
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
    interactive: false,
  });
};

export const makeIntersectionMarkingsLayer = (text) => {
  // These could change per locale
  const colors = {
    "sidewalk corner": "#CCCCCC",
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

export const makeDebugLayer = (text) => {
  return new L.geoJSON(JSON.parse(text), {
    onEachFeature: function (feature, layer) {
      if (feature.properties.label) {
        layer.bindTooltip(feature.properties.label, { permanent: true });
      }
    },
  });
};

export const makeBoundaryLayer = (geojson) => {
  return new L.geoJSON(geojson, { interactive: false });
};
