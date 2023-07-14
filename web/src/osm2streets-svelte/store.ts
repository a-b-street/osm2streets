import type { Feature, Polygon } from "geojson";
import type { LngLat, Map } from "maplibre-gl";
import type { JsStreetNetwork } from "osm2streets-js";
import { writable, type Writable } from "svelte/store";
import type { FeatureWithProps } from "./utils";

// These are all global singleton values, available anywhere in the code. When
// they're non-null, then they're loaded and ready to use.

export const map: Writable<Map | null> = writable(null);
export const network: Writable<JsStreetNetwork | null> = writable(null);
export const boundaryGj: Writable<Feature<Polygon> | null> = writable(null);

export const hoveredLane: Writable<FeatureWithProps<Polygon> | null> =
  writable(null);
export const hoveredIntersection: Writable<FeatureWithProps<Polygon> | null> =
  writable(null);

export const clickedLane: Writable<FeatureWithProps<Polygon> | null> =
  writable(null);
export const clickedIntersection: Writable<FeatureWithProps<Polygon> | null> =
  writable(null);
// When something is clicked, also remember the position the click happened
export const clickedLanePosition: Writable<LngLat | null> = writable(null);
export const clickedIntersectionPosition: Writable<LngLat | null> =
  writable(null);

// TODO Need to unsubscribe
// Unset when the network changes
network.subscribe((value) => {
  hoveredLane.set(null);
  hoveredIntersection.set(null);
  clickedLane.set(null);
  clickedIntersection.set(null);
  clickedLanePosition.set(null);
  clickedIntersectionPosition.set(null);
});
