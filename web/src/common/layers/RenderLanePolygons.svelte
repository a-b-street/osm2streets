<script lang="ts">
  import LayerControls from "../LayerControls.svelte";
  import { hoveredLane, network } from "../store";
  import { layerId } from "../utils";
  import { constructMatchExpression, emptyGeojson } from "svelte-utils";
  import { hoverStateFilter, FillLayer, GeoJSON } from "svelte-maplibre";

  export let hoverCursor: string | undefined = undefined;

  let show = true;

  $: gj = $network
    ? JSON.parse($network.toLanePolygonsGeojson())
    : emptyGeojson();
</script>

<GeoJSON data={gj} generateId>
  <FillLayer
    {...layerId("lane-polygons")}
    layout={{
      visibility: show ? "visible" : "none",
    }}
    manageHoverState
    bind:hovered={$hoveredLane}
    {hoverCursor}
    on:click
    paint={{
      "fill-color": constructMatchExpression(
        ["get", "type"],
        // TODO Could we express the Rust enum in TS and be type-safe here?
        {
          Driving: "black",
          "Parking(Parallel)": "#333333",
          "Parking(Diagonal)": "#333333",
          "Parking(Perpendicular)": "#333333",
          Sidewalk: "#CCCCCC",
          Shoulder: "#CCCCCC",
          Biking: "#0F7D4B",
          Bus: "#BE4A4C",
          SharedLeftTurn: "black",
          Construction: "#FF6D00",
          LightRail: "#844204",
          Footway: "#DDDDE8",
          SharedUse: "#DED68A",
          // These are the only types used currently
          "Buffer(Planters)": "#555555",
          "Buffer(Curb)": "white",
        },
        "red",
      ),
      "fill-opacity": hoverStateFilter(0.9, 0.4),
    }}
  >
    <slot />
  </FillLayer>
</GeoJSON>

<LayerControls {gj} name="Lane polygons" bind:show />
