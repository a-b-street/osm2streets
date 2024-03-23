<script lang="ts">
  import LayerControls from "../LayerControls.svelte";
  import { theme, hoveredIntersection, network } from "../store";
  import { caseHelper, layerId, emptyGeojson } from "../utils";
  import { SymbolLayer, hoverStateFilter, FillLayer, GeoJSON } from "svelte-maplibre";

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
            MapEdge: "#696",
            Terminus: "#999",
            Connection: "#666",
            Fork: "#669",
            Intersection: "#966",
          },
          "red",
        ),
        realistic: "black",
      }[$theme],
      "fill-opacity": hoverStateFilter(0.9, 0.4),
    }}
  >
    <slot />
  </FillLayer>
  <SymbolLayer
          filter={["==", ["get", "type"], "intersection"]}
          layout={{
          "text-field": ["get", "id"]
          }}
  />
</GeoJSON>

<LayerControls {gj} name="Intersection polygons" bind:show />
