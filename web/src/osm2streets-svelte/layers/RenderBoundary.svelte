<script lang="ts">
  import type { GeoJSON } from "geojson";
  import Layer from "../Layer.svelte";
  import LayerControls from "../LayerControls.svelte";
  import { boundaryGj, map } from "../store";
  import { bbox } from "../utils";

  let gj: GeoJSON | undefined = undefined;
  let show = true;
  $: if ($boundaryGj) {
    gj = structuredClone($boundaryGj);

    // Initially zoom to fit the imported boundary
    $map.fitBounds(bbox(gj), { animate: false, padding: 10 });
  } else {
    gj = undefined;
  }

  let layerStyle = {
    type: "line",
    paint: {
      "line-color": "blue",
      "line-width": 4,
    },
  };
</script>

<Layer source="boundary" {gj} {layerStyle} {show} />
<LayerControls {gj} name="Boundary" bind:show />
