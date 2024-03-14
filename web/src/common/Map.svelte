<script lang="ts">
  import { MapLibre, ScaleControl, NavigationControl } from "svelte-maplibre";
  import type { Map } from "maplibre-gl";
  import { map as mapStore } from "./store";

  let map: Map;
  let loaded = false;

  $: if (loaded) {
    mapStore.set(map);
  }
</script>

<div>
  <MapLibre
    style="https://api.maptiler.com/maps/streets/style.json?key=MZEJTanw3WpxRvt7qDfo"
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
