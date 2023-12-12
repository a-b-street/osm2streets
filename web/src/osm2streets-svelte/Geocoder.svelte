<script lang="ts">
  import { createMapLibreGlMapController } from "@maptiler/geocoding-control/maplibregl";
  import GeocodingControl from "@maptiler/geocoding-control/svelte/GeocodingControl.svelte";
  import type { MapController } from "@maptiler/geocoding-control/types";
  import maplibregl from "maplibre-gl";
  import { onMount } from "svelte";
  import { map } from "./store";

  let mapController: MapController;

  // TODO HMR is broken
  onMount(() => {
    mapController = createMapLibreGlMapController($map, maplibregl);
  });

  // TODO Show markers
  // TODO Set the flyTo duration
</script>

{#if mapController}
  <div>
    <GeocodingControl {mapController} apiKey="MZEJTanw3WpxRvt7qDfo" />
  </div>
{/if}

<style>
  div {
    position: absolute;
    top: 50px;
    right: 10px;
  }
</style>
