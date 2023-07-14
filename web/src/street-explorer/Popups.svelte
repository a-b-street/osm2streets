<script lang="ts">
  import { Popup } from "maplibre-gl";
  import { onDestroy } from "svelte";
  import { clickedIntersection, clickedLane, map } from "../osm2streets-svelte";
  import IntersectionPopup from "./IntersectionPopup.svelte";
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
      new LanePopup({
        target: container,
        props: { lane: $clickedLane },
      });
      // TODO Not sure what point to base the popup at. Use turf centroid at least, maybe
      let center = $clickedLane.geometry.coordinates[0][0] as [number, number];
      popup.setLngLat(center).setDOMContent(container).addTo($map!);
    } else if ($clickedIntersection) {
      let container = document.createElement("div");
      new IntersectionPopup({
        target: container,
        props: { intersection: $clickedIntersection },
      });
      let center = $clickedIntersection.geometry.coordinates[0][0] as [
        number,
        number
      ];
      popup.setLngLat(center).setDOMContent(container).addTo($map!);
    } else {
      popup.remove();
    }
  }
</script>
