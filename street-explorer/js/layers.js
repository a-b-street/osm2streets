import L from "leaflet";
import "leaflet-osm";

export const makePlainGeoJsonLayer = (
  network,
  dynamicMovementLayer,
  map,
  maybeGroup
) => {
  // TODO Update for new types
  const intersectionColours = {
    Connection: "#666",
    Intersection: "#966",
    Terminus: "#999",
    MapEdge: "#696",
  };
  return new L.geoJSON(JSON.parse(network.toGeojsonPlain()), {
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

      let popup = document.createElement("div");

      // Show JSON properties
      let pre = document.createElement("pre");
      pre.textContent = JSON.stringify(feature.properties, null, 2);
      popup.appendChild(pre);

      popup.appendChild(document.createElement("br"));

      // Link to original OSM nodes
      popup.appendChild(document.createTextNode("OSM nodes: "));
      for (const id of nodes) {
        let a = document.createElement("a");
        a.href = `https://www.openstreetmap.org/node/${id}`;
        a.target = "_blank";
        a.textContent = id;
        popup.appendChild(a);
        popup.appendChild(document.createTextNode(", "));
      }
      // Remove the trailing comma
      if (nodes.length != 0) {
        popup.removeChild(popup.lastChild);
      }

      // Buttons to operate on the road
      if (maybeGroup) {
        let buttons = document.createElement("div");
        buttons.appendChild(
          makeButton("Collapse intersection", () => {
            network.collapseIntersection(feature.properties.id);
            rerenderNetwork(network, dynamicMovementLayer, map, maybeGroup);
          })
        );
        popup.appendChild(buttons);
      }

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

export const makeLanePolygonLayer = (
  network,
  dynamicMovementLayer,
  map,
  maybeGroup
) => {
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

      let popup = document.createElement("div");

      // Show JSON properties
      let pre = document.createElement("pre");
      pre.textContent = JSON.stringify(feature.properties, null, 2);
      popup.appendChild(pre);

      popup.appendChild(document.createElement("br"));

      // Link to original OSM ways
      popup.appendChild(document.createTextNode("OSM ways: "));
      for (const id of ways) {
        let a = document.createElement("a");
        a.href = `https://www.openstreetmap.org/way/${id}`;
        a.target = "_blank";
        a.textContent = id;
        popup.appendChild(a);
        popup.appendChild(document.createTextNode(", "));
      }
      // Remove the trailing comma
      if (ways.length != 0) {
        popup.removeChild(popup.lastChild);
      }

      // Buttons to operate on the road
      if (maybeGroup) {
        let buttons = document.createElement("div");
        buttons.appendChild(
          makeButton("Collapse short road", () => {
            network.collapseShortRoad(feature.properties.road);
            rerenderNetwork(network, dynamicMovementLayer, map, maybeGroup);
          })
        );
        buttons.appendChild(
          makeButton("Zip sidepath", () => {
            network.zipSidepath(feature.properties.road);
            rerenderNetwork(network, dynamicMovementLayer, map, maybeGroup);
          })
        );
        popup.appendChild(buttons);
      }

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
    "vehicle stop line": "white",
    "bike stop line": "green",
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

function rerenderNetwork(network, dynamicMovementLayer, map, group) {
  group.replaceLayer(
    "Geometry",
    makePlainGeoJsonLayer(network, dynamicMovementLayer, map, group)
  );
  group.replaceLayer(
    "Lane polygons",
    makeLanePolygonLayer(network, dynamicMovementLayer, map, group)
  );
  group.replaceLayer(
    "Lane markings",
    makeLaneMarkingsLayer(network.toLaneMarkingsGeojson())
  );
  group.replaceLayer(
    "Intersection markings",
    makeIntersectionMarkingsLayer(network.toIntersectionMarkingsGeojson())
  );
  group.replaceLayer(
    "Debug road ordering",
    () => makeDebugLayer(network.debugClockwiseOrderingGeojson()),
    { lazy: true }
  );
}

function makeButton(label, onclick) {
  const button = document.createElement("button");
  button.type = "button";
  button.innerText = label;
  button.onclick = onclick;
  return button;
}
