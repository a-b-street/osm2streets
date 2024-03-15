<script lang="ts">
  import LayerControls from "../LayerControls.svelte";
  import { theme, hoveredIntersection, network } from "../store";
  import { caseHelper, layerId, emptyGeojson } from "../utils";
  import { hoverStateFilter, FillLayer, GeoJSON } from "svelte-maplibre";

  export let hoverCursor: string | undefined = undefined;

  let show = true;

  $: gj = $network ? JSON.parse($network.toGeojsonPlain()) : emptyGeojson();
</script>

<GeoJSON data={gj} generateId>
  <FillLayer
    {...layerId("intersection-polygons")}
    layout={{
      visibility: show ? "visible" : "none",
    }}
    manageHoverState
    bind:hovered={$hoveredIntersection}
    {hoverCursor}
    filter={["==", ["get", "type"], "intersection"]}
    paint={{
      "fill-color": {
        debug: caseHelper(
          "intersection_kind",
          {
            Connection: "#666",
            Intersection: "#966",
            Terminus: "#999",
            MapEdge: "#696",
          },
          "#666",
        ),
        realistic: "black",
      }[$theme],
      "fill-opacity": hoverStateFilter(0.9, 0.4),
    }}
  >
    <slot />
  </FillLayer>
</GeoJSON>

<LayerControls {gj} name="Intersection polygons" bind:show />
