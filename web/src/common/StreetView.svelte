<script lang="ts">
  import type { MapMouseEvent } from "maplibre-gl";
  import { onDestroy } from "svelte";
  import { map } from "./store";

  export let enabled = false;
  let source = "google";

  function on() {
    $map!.on("click", onClick);
    $map!.getCanvas().style.cursor = "zoom-in";
  }
  function off() {
    $map!.off("click", onClick);
    $map!.getCanvas().style.cursor = "auto";
  }
  onDestroy(off);

  $: if (enabled) {
    on();
  } else {
    off();
  }

  function onClick(e: MapMouseEvent) {
    let lon = e.lngLat.lng;
    let lat = e.lngLat.lat;
    if (source == "google") {
      window.open(
        `http://maps.google.com/maps?q=&layer=c&cbll=${lat},${lon}&cbp=11,0,0,0,0`,
        "_blank",
      );
    } else if (source == "bing") {
      window.open(
        `https://www.bing.com/maps?cp=${lat}~${lon}&style=x`,
        "_blank",
      );
    }
  }

  function onKeyDown(e: KeyboardEvent) {
    if (enabled && e.key == "Escape") {
      enabled = false;
    }
  }
</script>

<svelte:window on:keydown={onKeyDown} />

<label>
  <input type="checkbox" bind:checked={enabled} />
  StreetView
</label>
{#if enabled}
  <div style="background: grey">
    <label>
      <input type="radio" bind:group={source} value="google" />
      Google StreetView
    </label>
    <label>
      <input type="radio" bind:group={source} value="bing" />
      Bing Streetside
    </label>
  </div>
{/if}
