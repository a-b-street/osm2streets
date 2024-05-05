import type { FeatureCollection } from "geojson";
import { writable, type Writable } from "svelte/store";
import { emptyGeojson } from "svelte-utils";
import { network } from "../common";

export const blockGj: Writable<FeatureCollection> = writable(emptyGeojson());
// Does blockGj currently represent blocks or bundles?
export const showingBundles = writable(false);

// TODO Need to unsubscribe
// Unset when the network changes
network.subscribe((value) => {
  blockGj.set(emptyGeojson());
});
