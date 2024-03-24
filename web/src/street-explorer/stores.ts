import type { FeatureCollection } from "geojson";
import { writable, type Writable } from "svelte/store";
import { emptyGeojson } from "../common/utils";

export const blockGj: Writable<FeatureCollection> = writable(emptyGeojson());
