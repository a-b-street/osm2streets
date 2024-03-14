<script lang="ts">
  import LayerControls from "../LayerControls.svelte";
  import { hoveredLane, network } from "../store";
  import { layerId, emptyGeojson } from "../utils";
  import { FillLayer, GeoJSON } from "svelte-maplibre";

  let show = true;

  $: gj =
    $network && $hoveredLane
      ? JSON.parse(
          $network!.debugMovementsFromLaneGeojson(
            $hoveredLane.properties.road,
            $hoveredLane.properties.index,
          ),
        )
      : emptyGeojson();
</script>

<GeoJSON data={gj}>
  <FillLayer
    {...layerId("movements")}
    layout={{
      visibility: show ? "visible" : "none",
    }}
    paint={{
      "fill-color": "blue",
      "fill-opacity": 0.5,
    }}
  />
</GeoJSON>

<LayerControls {gj} name="Movement arrows" bind:show downloadable={false} />
