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

  var hideFaces = new Set();
  group.addLazyLayer(
    "Planar graph (faces)",
    () =>
      new L.geoJSON(JSON.parse(network.toPlanarGeojsonFaces()), {
        style: function (feature) {
          return feature.properties;
        },
        onEachFeature: function (feature, layer) {
          layer.on({
            click: function (ev) {
              // TODO Doesn't stop double-click zooming
              L.DomEvent.preventDefault(ev);
              const layer = ev.target;
              const id = layer.feature.properties.id;
              var fillOpacity = 0.5;
              if (hideFaces.has(id)) {
                hideFaces.delete(id);
                fillOpacity = 0.5;
              } else {
                hideFaces.add(id);
                fillOpacity = 0.1;
              }
              layer.setStyle({ fillOpacity });
            },
          });
        },
      })
  );

  let div = document.getElementById("crowdedStuff");
  map.on({
    mousemove: (e) => {
      let pt = [e.latlng.lng, e.latlng.lat];
      let points = 0;
      let lines = 0;
      for (let feature of networkJSON.features) {
        if (feature.geometry.type == "Polygon") {
          if (booleanPointInPolygon(pt, feature)) {
            points++;
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
