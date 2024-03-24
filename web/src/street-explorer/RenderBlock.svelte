<script lang="ts">
  import { caseHelper, layerId, emptyGeojson } from "../common/utils";
  import { Popup, FillLayer, GeoJSON } from "svelte-maplibre";
  import { blockGj } from "./stores";
  import { network } from "../common";

  function clear() {
    blockGj.set(emptyGeojson());
  }

  function findAll() {
    blockGj.set(JSON.parse($network!.findAllBlocks()));
  }
</script>

<GeoJSON data={$blockGj}>
  <FillLayer
    {...layerId("block")}
    paint={{
      "fill-color": caseHelper(
        "kind",
        {
          RoadAndSidewalk: "green",
          DualCarriageway: "purple",
          Unknown: "red",
        },
        "red",
      ),
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
  <button on:click={findAll}>Find all</button>
</div>
