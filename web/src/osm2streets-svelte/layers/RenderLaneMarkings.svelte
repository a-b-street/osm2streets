<script lang="ts">
  import type { GeoJSON } from "geojson";
  import Layer from "../Layer.svelte";
  import LayerControls from "../LayerControls.svelte";
  import { network } from "../store";
  import { caseHelper } from "../utils";

  let gj: GeoJSON | undefined = undefined;
  let show = true;
  $: if ($network) {
    gj = JSON.parse($network.toLaneMarkingsGeojson());
  } else {
    gj = undefined;
  }

  let layerStyle = {
    type: "fill",
    paint: {
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
        "red"
      ),
      "fill-opacity": 0.9,
    },
  };
</script>

<Layer source="lane-markings" {gj} {layerStyle} {show} />
<LayerControls {gj} name="Lane markings " bind:show />
