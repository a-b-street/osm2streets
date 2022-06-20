export const addGeojsonLayer = (map, data) => L.geoJSON(data).addTo(map);
export const zoomToLayer = (map, layer) => map.flyToBounds(layer.getBounds());
