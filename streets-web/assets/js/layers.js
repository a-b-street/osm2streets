export const addGeojsonLayer = (map, json) => L.geoJSON(json).addTo(map);
export const addOsmLayer = (map, xml) => L.OSM.DataLayer(xml).addTo(map);

export const zoomToLayer = (map, layer) => map.flyToBounds(layer.getBounds());
