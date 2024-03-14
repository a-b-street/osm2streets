<script lang="ts">
  import LayerControls from "../LayerControls.svelte";
  import { boundaryGj, map } from "../store";
  import { layerId, bbox, emptyGeojson } from "../utils";
  import { LineLayer, GeoJSON } from "svelte-maplibre";

  let show = true;

  $: gj = $boundaryGj ?? emptyGeojson();

  $: if ($boundaryGj) {
    // Initially zoom to fit the imported boundary
    $map?.fitBounds(bbox($boundaryGj), { animate: false, padding: 10 });
  }
</script>

<GeoJSON data={gj}>
  <LineLayer
    {...layerId("boundary")}
    layout={{
      visibility: show ? "visible" : "none",
    }}
    paint={{
      "line-color": "blue",
      "line-width": 4,
    }}
  />
</GeoJSON>

<LayerControls {gj} name="Boundary" bind:show />
