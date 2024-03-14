<script lang="ts">
  import { layerId, emptyGeojson, caseHelper } from "../utils";
  import { FillLayer, GeoJSON } from "svelte-maplibre";
  import LayerControls from "../LayerControls.svelte";
  import { network } from "../store";

  let show = true;

  $: gj = $network
    ? JSON.parse($network.toLaneMarkingsGeojson())
    : emptyGeojson();
</script>

<GeoJSON data={gj}>
  <FillLayer
    {...layerId("lane-markings")}
    layout={{
      visibility: show ? "visible" : "none",
    }}
    paint={{
      "fill-color": caseHelper(
        "type",
        {
          "center line": "yellow",
          "lane separator": "white",
          "lane arrow": "white",
          "buffer edge": "white",
          "buffer stripe": "white",
          "vehicle stop line": "white",
          "bike stop line": "green",
        },
        "red",
      ),
      "fill-opacity": 0.9,
    }}
  />
</GeoJSON>

<LayerControls {gj} name="Lane markings " bind:show />
