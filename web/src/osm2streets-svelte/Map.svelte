<script lang="ts">
  import { Map, NavigationControl, ScaleControl } from "maplibre-gl";
  import { onDestroy, onMount } from "svelte";
  import "maplibre-gl/dist/maplibre-gl.css";
  import { map as mapStore } from "./store";

  let map: Map;
  let mapContainer: HTMLDivElement;
  let loaded = false;

  onMount(() => {
    map = new Map({
      container: mapContainer,
      style:
        "https://api.maptiler.com/maps/streets/style.json?key=MZEJTanw3WpxRvt7qDfo",
      hash: true,
    });
    map.addControl(new ScaleControl({}));
    map.addControl(new NavigationControl({}), "bottom-right");

    map.on("load", () => {
      loaded = true;
      // Debugging
      //window.map = map;
      mapStore.set(map);
    });

    const resizeObserver = new ResizeObserver(() => {
      map.resize();
    });
    resizeObserver.observe(mapContainer);
  });

  onDestroy(() => {
    map.remove();
  });
</script>

<div class="map" bind:this={mapContainer}>
  {#if loaded}
    <slot />
  {/if}
</div>

<style>
  .map {
    position: relative;
    flex-grow: 1;
    /* TODO: Hack, can't figure out why height broken */
    min-height: 100vh;
  }
</style>
