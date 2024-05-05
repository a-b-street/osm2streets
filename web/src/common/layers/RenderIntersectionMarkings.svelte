<script lang="ts">
  import LayerControls from "../LayerControls.svelte";
  import { network } from "../store";
  import { layerId, caseHelper } from "../utils";
  import { emptyGeojson } from "svelte-utils";
  import { FillLayer, GeoJSON } from "svelte-maplibre";

  let show = true;

  $: gj = $network
    ? JSON.parse($network.toIntersectionMarkingsGeojson())
    : emptyGeojson();
</script>

<GeoJSON data={gj}>
  <FillLayer
    {...layerId("intersection-markings")}
    layout={{
      visibility: show ? "visible" : "none",
    }}
    paint={{
      "fill-color": caseHelper(
        "type",
        {
          "sidewalk corner": "#CCCCCC",
          "marked crossing line": "white",
          "unmarked crossing outline": "white",
        },
        "red",
      ),
      "fill-opacity": 0.9,
    }}
  />
</GeoJSON>

<LayerControls {gj} name="Intersection markings" bind:show />
