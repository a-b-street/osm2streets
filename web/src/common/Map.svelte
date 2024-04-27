<script lang="ts">
  import { MapLibre, ScaleControl, NavigationControl } from "svelte-maplibre";
  import type { Map, StyleSpecification } from "maplibre-gl";
  import { map as mapStore, basemap, maptilerApiKey } from "./store";
  import { PolygonToolLayer } from "maplibre-draw-polygon";
  import { Geocoder } from "svelte-utils";

  let map: Map;
  let loaded = false;

  function getStyle(basemap: string): string | StyleSpecification {
    if (basemap == "blank") {
      return {
        version: 8,
        sources: {},
        layers: [],
      };
    }

    return `https://api.maptiler.com/maps/${basemap}/style.json?key=${maptilerApiKey}`;
  }

  $: if (loaded) {
    mapStore.set(map);
  }
</script>

<div>
  <MapLibre
    style={getStyle($basemap)}
    hash
    bind:map
    bind:loaded
    on:error={(e) => {
      // @ts-expect-error Not exported
      console.log(e.detail.error);
    }}
  >
    {#if $mapStore}
      <ScaleControl />
      <NavigationControl position="bottom-right" visualizePitch />
      <PolygonToolLayer />
      <Geocoder {map} apiKey={maptilerApiKey} />
      <slot />
    {/if}
  </MapLibre>
</div>

<style>
  div {
    position: relative;
    height: 100vh;
  }
</style>
