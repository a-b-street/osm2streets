<script lang="ts">
  import { layerId } from "../utils";
  import { emptyGeojson, constructMatchExpression } from "svelte-utils";
  import { FillLayer, GeoJSON } from "svelte-maplibre";
  import LayerControls from "../LayerControls.svelte";
  import { network } from "../store";

  let show = true;

  $: gj = $network
    ? JSON.parse($network.toLaneMarkingsGeojson())
    : emptyGeojson();

  let general_road_marking = "white";
</script>

<GeoJSON data={gj}>
  <FillLayer
    {...layerId("lane-markings")}
    layout={{
      visibility: show ? "visible" : "none",
    }}
    paint={{
      "fill-color": constructMatchExpression(
        ["get", "type"],
        {
          "center line": "yellow",
          "lane separator": general_road_marking,
          "lane arrow": general_road_marking,
          "buffer edge": general_road_marking,
          "buffer stripe": general_road_marking,
          "parking hatch": general_road_marking,
          "vehicle stop line": general_road_marking,
          "sidewalk line": "#BBBBBB",
          "bike stop line": "green",
          "path outline": "black",
        },
        "red",
      ),
      "fill-opacity": 0.9,
    }}
  />
</GeoJSON>

<LayerControls {gj} name="Lane markings " bind:show />
