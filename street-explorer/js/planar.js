import booleanPointInPolygon from "@turf/boolean-point-in-polygon";
import pointToLineDistance from "@turf/point-to-line-distance";

export function addPlanar(group, network, map) {
  let networkJSON = JSON.parse(network.toPlanarGeojsonNetwork());
  group.addLazyLayer(
    "Planar graph (network)",
    () =>
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

  let faceJson = JSON.parse(network.toPlanarGeojsonFaces());
  let faceLayer = new L.geoJSON(faceJson, {
    style: function (feature) {
      return feature.properties;
    },
    onEachFeature: function (feature, layer) {
      layer.on({
        mouseover: function (ev) {
          ev.target.setStyle({
            fillOpacity: 0.1,
          });
        },
        mouseout: function (ev) {
          layer.setStyle({
            fillOpacity: 0.5,
          });
        },
      });
    },
  });
  group.addLayer("Planar graph (faces)", faceLayer);

  let div = document.getElementById("crowdedStuff");
  let cursorPos = document.getElementById("cursorPos");
  let someFaceId = null;
  map.on({
    mousemove: (e) => {
      let pt = [e.latlng.lng, e.latlng.lat];
      let nodes = 0;
      let edges = [];
      cursorPos.innerText = "hover on a node";
      for (let feature of networkJSON.features) {
        if (feature.geometry.type == "Polygon") {
          if (booleanPointInPolygon(pt, feature)) {
            nodes++;
            cursorPos.innerText = feature.properties.id;
          }
        } else {
          if (pointToLineDistance(pt, feature, { units: "meters" }) < 1.0) {
            edges.push(
              `${feature.properties.id} (${feature.properties.sources})`
            );
          }
        }
      }
      if (nodes > 1 || edges.length > 1) {
        div.innerHTML = `<h1 style="color: red">${nodes} nodes, ${edges.length} edges by cursor</h1>`;
      } else {
        div.innerHTML = `${nodes} nodes, ${edges.length} edges by cursor (${edges})`;
      }

      if (group.isEnabled("Planar graph (faces)")) {
        let faces = 0;
        someFaceId = null;
        for (let feature of faceJson.features) {
          if (booleanPointInPolygon(pt, feature)) {
            faces++;
            div.innerText = `Face: ${feature.properties.sources}`;
            if (faces == 1) {
              someFaceId = feature.properties.id;
            }
          }
        }
        if (faces > 1) {
          div.innerHTML = `<h1 style="color: red">${faces} faces</h1>`;
        }
      }
    },
  });

  window.addEventListener("keydown", (e) => {
    if (e.key == "Delete") {
      console.log(`Deleted ${someFaceId}`);
      faceLayer.eachLayer((layer) => {
        if (layer.feature.properties.id == someFaceId) {
          faceLayer.removeLayer(layer);
        }
      });

      faceJson.features = faceJson.features.filter(
        (f) => f.properties.id != someFaceId
      );
    } else if (e.key == "Backspace") {
      console.log(`Deleted everything except ${someFaceId}`);
      faceLayer.eachLayer((layer) => {
        if (layer.feature.properties.id != someFaceId) {
          faceLayer.removeLayer(layer);
        }
      });

      faceJson.features = faceJson.features.filter(
        (f) => f.properties.id == someFaceId
      );
    }
  });
}
