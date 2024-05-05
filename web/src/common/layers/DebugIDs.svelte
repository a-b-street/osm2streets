<script lang="ts">
  import LayerControls from "../LayerControls.svelte";
  import { network } from "../store";
  import { layerId } from "../utils";
  import { emptyGeojson } from "svelte-utils";
  import { SymbolLayer, GeoJSON } from "svelte-maplibre";

  let show = false;

  $: gj = $network ? JSON.parse($network.toGeojsonPlain()) : emptyGeojson();
</script>

<GeoJSON data={gj} generateId>
  <SymbolLayer
    {...layerId("debug-ids")}
    layout={{
      "text-field": ["get", "id"],
      visibility: show ? "visible" : "none",
    }}
    paint={{
      "text-halo-color": [
        "case",
        ["==", ["get", "type"], "intersection"],
        "red",
        "cyan",
      ],
      "text-halo-width": 3,
    }}
  />
</GeoJSON>

<LayerControls {gj} name="Debug IDs" bind:show downloadable={false} />
