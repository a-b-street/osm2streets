import "leaflet/dist/leaflet.css";
import "leaflet-geosearch/dist/geosearch.css";

import L from "leaflet";
import { GeoSearchControl, OpenStreetMapProvider } from "leaflet-geosearch";
import "leaflet-hash";
import "./deps/SmoothWheelZoom.js";

export function setupLeafletMap(mapContainer) {
  const osm = L.tileLayer(
    "https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png",
    {
      maxNativeZoom: 18,
      maxZoom: 21,
      attribution: "© OpenStreetMap",
    }
  );
  const arcgis = L.tileLayer(
    "https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}",
    {
      attribution: "© ArcGIS",
      maxNativeZoom: 18,
      maxZoom: 21,
    }
  );

  const map = L.map(mapContainer, {
    layers: [osm],
    maxZoom: 21,
    zoomSnap: 0,
    zoomDelta: 0.5,
    scrollWheelZoom: false,
    smoothWheelZoom: true,
    smoothSensitivity: 1,
  }).setView([40.0, 10.0], 4);

  new GeoSearchControl({
    provider: new OpenStreetMapProvider(),
    showMarker: false,
    autoClose: true,
  }).addTo(map);

  new L.hash(map);

  L.control
    .layers({ OpenStreetMap: osm, ArcGIS: arcgis }, {}, { collapsed: false })
    .addTo(map);

  return map;
}
