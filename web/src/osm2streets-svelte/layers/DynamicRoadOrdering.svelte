<script lang="ts">
  import { Popup } from "maplibre-gl";
  import { hoveredIntersection, map, network } from "../store";

  // TODO UX bug: if the mouse winds up on any popup, $hoveredIntersection
  // becomes null, and everything flickers. Can we display them on top visually,
  // but not block mouseover?
  let popups: Popup[] = [];
  let show = false;

  $: {
    // Clear anything existing
    for (let p of popups) {
      p.remove();
    }
    popups = [];

    if (show && $hoveredIntersection) {
      let gj = JSON.parse(
        $network!.debugClockwiseOrderingForIntersectionGeojson(
          $hoveredIntersection.properties.id
        )
      );
      for (let f of gj.features) {
        popups.push(
          new Popup({
            closeButton: false,
            closeOnClick: false,
            focusAfterOpen: false,
          })
            .setLngLat(f.geometry.coordinates)
            .setHTML(f.properties.label)
            .addTo($map!)
        );
      }
    }
  }
</script>

<div>
  <label>
    <input type="checkbox" bind:checked={show} />
    Clockwise ordering of roads
  </label>
</div>
