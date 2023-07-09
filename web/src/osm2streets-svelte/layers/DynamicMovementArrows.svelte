<script lang="ts">
  import type { GeoJSON } from "geojson";
  import Layer from "../Layer.svelte";
  import LayerControls from "../LayerControls.svelte";
  import { hoveredLane, network } from "../store";

  let gj: GeoJSON | undefined = undefined;
  let show = true;
  $: if ($network && $hoveredLane) {
    let props = $hoveredLane.properties;
    gj = JSON.parse(
      $network!.debugMovementsFromLaneGeojson(props.road, props.index)
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

<Layer source="movements" {gj} {layerStyle} {show} />
<LayerControls {gj} name="Movement arrows" bind:show downloadable={false} />
