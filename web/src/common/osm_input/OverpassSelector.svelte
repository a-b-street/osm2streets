<script lang="ts">
  import type { Feature, Polygon } from "geojson";
  import { createEventDispatcher } from "svelte";
  import type { Map, LngLat } from "maplibre-gl";
  import type { OsmSelection } from "./types";
  import { PolygonControls, PolygonTool } from "maplibre-draw-polygon";

  export let map: Map | null;

  let polygonTool: PolygonTool | null = null;

  let dispatch = createEventDispatcher<{
    loading: string;
    load: OsmSelection;
    error: string;
  }>();

  function startPolygonTool() {
    if (!map) {
      return;
    }
    polygonTool = new PolygonTool(map);
    polygonTool.startNew();
    polygonTool.addEventListenerSuccess(async (f) => {
      polygonTool = null;
      await importPolygon(f);
    });
    polygonTool.addEventListenerFailure(() => {
      polygonTool = null;
    });
  }

  // Also exported for callers to trigger manually
  export async function importPolygon(boundaryGj: Feature<Polygon>) {
    try {
      dispatch("loading", "Loading from Overpass");
      let resp = await fetch(overpassQueryForPolygon(boundaryGj));
      let osmXml = await resp.text();

      dispatch("load", {
        testCase: "none",
        boundaryGj,
        osmXml,
      });
    } catch (err: any) {
      dispatch("error", err.toString());
    }
  }

  // Construct a query to extract all XML data in the polygon clip. See
  // https://wiki.openstreetmap.org/wiki/Overpass_API/Overpass_QL
  function overpassQueryForPolygon(feature: Feature<Polygon>): string {
    let filter = 'poly:"';
    for (let [lng, lat] of feature.geometry.coordinates[0]) {
      filter += `${lat} ${lng} `;
    }
    filter = filter.slice(0, -1) + '"';
    let query = `(nwr(${filter}); node(w)->.x; <;); out meta;`;
    return `https://overpass-api.de/api/interpreter?data=${query}`;
  }

  function latLngToGeojson(pt: LngLat): [number, number] {
    return [pt.lng, pt.lat];
  }

  // Turn the current viewport into a rectangular boundary
  function mapBoundsToGeojson(): Feature<Polygon> {
    let b = map!.getBounds();
    return {
      type: "Feature",
      properties: {},
      geometry: {
        coordinates: [
          [
            latLngToGeojson(b.getSouthWest()),
            latLngToGeojson(b.getNorthWest()),
            latLngToGeojson(b.getNorthEast()),
            latLngToGeojson(b.getSouthEast()),
            latLngToGeojson(b.getSouthWest()),
          ],
        ],
        type: "Polygon",
      },
    };
  }

  async function importCurrentView() {
    if (!map) {
      return;
    }
    if (map.getZoom() < 15) {
      dispatch("error", "Zoom in more to import");
      return;
    }
    await importPolygon(mapBoundsToGeojson());
  }
</script>

{#if polygonTool}
  <PolygonControls {polygonTool} />
{:else}
  <button type="button" on:click={importCurrentView}>
    Import current view
  </button>

  <i>or...</i>

  <button type="button" on:click={startPolygonTool}>
    Draw an area to import on the map
  </button>
{/if}
