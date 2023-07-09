<script lang="ts">
  import type { GeoJSON } from "geojson";
  import Layer from "../Layer.svelte";
  import LayerControls from "../LayerControls.svelte";
  import { hoveredIntersection, network } from "../store";

  let gj: GeoJSON | undefined = undefined;
  let show = true;
  $: if ($network && $hoveredIntersection) {
    gj = JSON.parse(
      $network!.debugRoadsConnectedToIntersectionGeojson(
        $hoveredIntersection.properties.id
      )
    );
  } else {
    gj = undefined;
  }

  let layerStyle = {
    type: "fill",
    paint: {
      "fill-color": "blue",
      "fill-opacity": 0.5,
    },
  };
</script>

<Layer source="connected-roads" {gj} {layerStyle} {show} />
<LayerControls
  {gj}
  name="Roads connected to intersection"
  bind:show
  downloadable={false}
/>
