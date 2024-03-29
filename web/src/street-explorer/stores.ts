import type { FeatureCollection } from "geojson";
import { writable, type Writable } from "svelte/store";
import { emptyGeojson } from "../common/utils";
import { network } from "../common";

export const blockGj: Writable<FeatureCollection> = writable(emptyGeojson());

// TODO Need to unsubscribe
// Unset when the network changes
network.subscribe((value) => {
  blockGj.set(emptyGeojson());
});
