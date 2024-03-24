<script lang="ts">
  import { layerId, emptyGeojson } from "../common/utils";
  import { Popup, FillLayer, GeoJSON } from "svelte-maplibre";
  import { blockGj } from "./stores";

  function clear() {
    blockGj.set(emptyGeojson());
  }
</script>

<GeoJSON data={$blockGj}>
  <FillLayer
    {...layerId("block")}
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
  Block <button on:click={clear}>Clear</button>
</div>
