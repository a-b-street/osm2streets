<script lang="ts">
  import LayerControls from "../LayerControls.svelte";
  import { hoveredIntersection, network } from "../store";
  import { layerId, emptyGeojson } from "../utils";
  import { FillLayer, GeoJSON } from "svelte-maplibre";

  let show = true;

  $: gj =
    $network && $hoveredIntersection
      ? JSON.parse(
          $network!.debugRoadsConnectedToIntersectionGeojson(
            $hoveredIntersection.properties.id,
          ),
        )
      : emptyGeojson();
</script>

<GeoJSON data={gj}>
  <FillLayer
    {...layerId("connected-roads")}
    layout={{
      visibility: show ? "visible" : "none",
    }}
    paint={{
      "fill-color": "blue",
      "fill-opacity": 0.5,
    }}
  />
</GeoJSON>

<LayerControls
  {gj}
  name="Roads connected to intersection"
  bind:show
  downloadable={false}
/>
