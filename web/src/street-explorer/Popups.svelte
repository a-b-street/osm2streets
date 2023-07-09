<script lang="ts">
  import { Popup } from "maplibre-gl";
  import { onDestroy } from "svelte";
  import {
    clickedIntersection,
    clickedLane,
    map,
  } from "../osm2streets-svelte/store";
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
    // TODO Not sure if we can get the DOM contents of a component by doing this
    if ($clickedLane) {
      let container = document.createElement("div");
      new LanePopup({
        target: container,
        props: { lane: $clickedLane },
      });
      // TODO Not sure what point to base the popup at. Use turf centroid at least, maybe
      let center = $clickedLane.geometry.coordinates[0][0] as [number, number];
      popup.setLngLat(center).setDOMContent(container).addTo($map);
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
      popup.setLngLat(center).setDOMContent(container).addTo($map);
    } else {
      popup.remove();
    }
  }
</script>
