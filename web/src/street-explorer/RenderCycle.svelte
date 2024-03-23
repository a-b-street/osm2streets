<script lang="ts">
  import { layerId, emptyGeojson } from "../common/utils";
  import { Popup, FillLayer, GeoJSON } from "svelte-maplibre";
  import { cycleGj } from "./stores";

  function clear() {
    cycleGj.set(emptyGeojson());
  }
</script>

<GeoJSON data={$cycleGj}>
  <FillLayer
    {...layerId("cycle")}
    paint={{
      "fill-color": "purple",
      "fill-opacity": 0.8,
    }}
  >
    <Popup openOn="hover" let:data>
      <pre>{JSON.stringify(data.properties, null, "  ")}</pre>
    </Popup>
  </FillLayer>
</GeoJSON>

<div>
  Cycle <button on:click={clear}>Clear</button>
</div>
