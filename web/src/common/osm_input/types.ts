import type { Feature, Polygon } from "geojson";

// This component provides osm.xml data and a boundary GeoJSON, either from
// Overpass or a built-in file.
export interface OsmSelection {
  // "none" for boundaries from Overpass
  // TODO Ideally undefined for that, but then binding to a <select> is hard
  testCase: string;
  boundaryGj: Feature<Polygon>;
  osmXml: string;
}
