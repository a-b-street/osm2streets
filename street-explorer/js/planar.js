import booleanPointInPolygon from "@turf/boolean-point-in-polygon";
import pointToLineDistance from "@turf/point-to-line-distance";

export function addPlanar(group, network, map) {
  let networkJSON = JSON.parse(network.toPlanarGeojsonNetwork());

  group.addLayer(
    "Planar graph (network)",
    new L.geoJSON(networkJSON, {
      style: function (feature) {
        return feature.properties;
      },
      onEachFeature: function (feature, layer) {
        let isCircle = feature.geometry.type == "Polygon";
        layer.on({
          mouseover: function (ev) {
            ev.target.setStyle({
              fillOpacity: 0.5,
              opacity: 0.5,
            });
          },
          mouseout: function (ev) {
            layer.setStyle({
              fillOpacity: 0.9,
              opacity: 0.9,
            });
          },
        });
      },
    })
  );

  group.addLazyLayer(
    "Planar graph (faces)",
    () =>
      new L.geoJSON(JSON.parse(network.toPlanarGeojsonFaces()), {
        style: function (feature) {
          return feature.properties;
        },
        onEachFeature: function (feature, layer) {
          layer.on({
            mouseover: function (ev) {
              ev.target.setStyle({
                fillOpacity: 0.1
              });
            },
            mouseout: function (ev) {
              layer.setStyle({
                fillOpacity: 0.5
              });
            },

          });
        },
      })
  );

  let div = document.getElementById("crowdedStuff");
  let cursorPos = document.getElementById("cursorPos");
  map.on({
    mousemove: (e) => {
      let pt = [e.latlng.lng, e.latlng.lat];
      let points = 0;
      let lines = 0;
      cursorPos.innerText = "hover on a node";
      for (let feature of networkJSON.features) {
        if (feature.geometry.type == "Polygon") {
          if (booleanPointInPolygon(pt, feature)) {
            points++;
            cursorPos.innerText = feature.properties.id;
          }
        } else {
          if (pointToLineDistance(pt, feature, { units: "meters" }) < 1.0) {
            lines++;
          }
        }
      }
      if (points > 1 || lines > 1) {
        div.innerHTML = `<h1 style="color: red">${points} nodes, ${lines} edges by cursor</h1>`;
      } else {
        div.innerHTML = `${points} nodes, ${lines} edges by cursor`;
      }
    },
  });
}
