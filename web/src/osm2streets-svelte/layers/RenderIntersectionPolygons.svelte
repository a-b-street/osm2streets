<script lang="ts">
  import type { GeoJSON } from "geojson";
  import Layer from "../Layer.svelte";
  import LayerControls from "../LayerControls.svelte";
  import { clickedIntersection, hoveredIntersection, network } from "../store";
  import { caseHelper, featureStateToggle } from "../utils";

  let gj: GeoJSON | undefined = undefined;
  let show = true;
  $: if ($network) {
    gj = JSON.parse($network.toGeojsonPlain());
  } else {
    gj = undefined;
  }

  let layerStyle = {
    type: "fill",
    filter: ["==", ["get", "type"], "intersection"],
    paint: {
      "fill-color": caseHelper(
        "intersection_kind",
        {
          Connection: "#666",
          Intersection: "#966",
          Terminus: "#999",
          MapEdge: "#696",
        },
        "#666"
      ),
      "fill-opacity": featureStateToggle("hover", 0.9, 0.4),
    },
  };
</script>

<Layer
  source="intersection-polygons"
  {gj}
  {layerStyle}
  interactive
  bind:hoveredFeature={$hoveredIntersection}
  bind:clickedFeature={$clickedIntersection}
  {show}
/>
<LayerControls {gj} name="Intersection polygons" bind:show />
