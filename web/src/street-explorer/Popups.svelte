<script lang="ts">
  import { Popup } from "maplibre-gl";
  import { onDestroy } from "svelte";
  import { clickedLane, clickedLanePosition, map } from "../osm2streets-svelte";
  import LanePopup from "./LanePopup.svelte";

  let popup = new Popup({
    closeButton: false,
    closeOnClick: false,
    maxWidth: "none",
  });

  onDestroy(() => {
    popup.remove();
  });

  $: {
    if ($clickedLane) {
      // Instantiate the Svelte component manually, so we can then put the DOM
      // it creates into the popup
      let container = document.createElement("div");
      container.style.cssText = "height: 300px; overflow-y: auto;";
      new LanePopup({
        target: container,
        props: { lane: $clickedLane },
      });
      popup
        .setLngLat($clickedLanePosition!)
        .setDOMContent(container)
        .addTo($map!);
    } else {
      popup.remove();
    }
  }
</script>
