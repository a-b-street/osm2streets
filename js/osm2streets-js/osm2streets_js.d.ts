/* tslint:disable */
/* eslint-disable */
/**
*/
export class JsDebugStreets {
  free(): void;
/**
* @returns {string}
*/
  getLabel(): string;
/**
* @returns {any}
*/
  getNetwork(): any;
/**
* @returns {string | undefined}
*/
  toDebugGeojson(): string | undefined;
}
/**
*/
export class JsStreetNetwork {
  free(): void;
/**
* @param {string} osm_xml_input
* @param {any} input
*/
  constructor(osm_xml_input: string, input: any);
/**
* @returns {string}
*/
  toGeojsonPlain(): string;
/**
* @returns {string}
*/
  toLanePolygonsGeojson(): string;
/**
* @returns {string}
*/
  toLaneMarkingsGeojson(): string;
/**
* @returns {string}
*/
  toIntersectionMarkingsGeojson(): string;
/**
* @returns {string}
*/
  toGraphviz(): string;
/**
* @returns {any[]}
*/
  getDebugSteps(): any[];
/**
* @returns {string}
*/
  debugClockwiseOrderingGeojson(): string;
/**
* @returns {string}
*/
  debugMovementsGeojson(): string;
/**
* @param {bigint} id
* @returns {string}
*/
  getOsmTagsForWay(id: bigint): string;
/**
* Returns a GeoJSON Polygon showing a wide buffer around the way's original geometry
* @param {bigint} id
* @returns {string}
*/
  getGeometryForWay(id: bigint): string;
/**
* Modifies all affected roads and only reruns `Transformation::GenerateIntersectionGeometry`.
* @param {bigint} id
* @param {string} tags
*/
  overwriteOsmTagsForWay(id: bigint, tags: string): void;
/**
* Returns the XML string representing a way. Any OSM tags changed via
* `overwrite_osm_tags_for_way` are reflected.
* @param {bigint} id
* @returns {string}
*/
  wayToXml(id: bigint): string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_jsstreetnetwork_free: (a: number) => void;
  readonly jsstreetnetwork_new: (a: number, b: number, c: number, d: number) => void;
  readonly jsstreetnetwork_toGeojsonPlain: (a: number, b: number) => void;
  readonly jsstreetnetwork_toLanePolygonsGeojson: (a: number, b: number) => void;
  readonly jsstreetnetwork_toLaneMarkingsGeojson: (a: number, b: number) => void;
  readonly jsstreetnetwork_toIntersectionMarkingsGeojson: (a: number, b: number) => void;
  readonly jsstreetnetwork_toGraphviz: (a: number, b: number) => void;
  readonly jsstreetnetwork_getDebugSteps: (a: number, b: number) => void;
  readonly jsstreetnetwork_debugClockwiseOrderingGeojson: (a: number, b: number) => void;
  readonly jsstreetnetwork_debugMovementsGeojson: (a: number, b: number) => void;
  readonly jsstreetnetwork_getOsmTagsForWay: (a: number, b: number, c: number, d: number) => void;
  readonly jsstreetnetwork_getGeometryForWay: (a: number, b: number, c: number, d: number) => void;
  readonly jsstreetnetwork_overwriteOsmTagsForWay: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly jsstreetnetwork_wayToXml: (a: number, b: number, c: number, d: number) => void;
  readonly __wbg_jsdebugstreets_free: (a: number) => void;
  readonly jsdebugstreets_getLabel: (a: number, b: number) => void;
  readonly jsdebugstreets_getNetwork: (a: number) => number;
  readonly jsdebugstreets_toDebugGeojson: (a: number, b: number) => void;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
}

/**
* Synchronously compiles the given `bytes` and instantiates the WebAssembly module.
*
* @param {BufferSource} bytes
*
* @returns {InitOutput}
*/
export function initSync(bytes: BufferSource): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
