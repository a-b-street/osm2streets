<script lang="ts">
  import { caseHelper, layerId, emptyGeojson } from "../common/utils";
  import { Popup, FillLayer, GeoJSON } from "svelte-maplibre";
  import { blockGj } from "./stores";
  import { network, Legend } from "../common";

  $: active = $blockGj.features.length > 0;

  function clear() {
    blockGj.set(emptyGeojson());
  }

  function findAll() {
    blockGj.set(JSON.parse($network!.findAllBlocks()));
  }

  let colors = {
    RoadAndSidewalk: "green",
    RoadAndCycleLane: "orange",
    CycleLaneAndSidewalk: "yellow",
    DualCarriageway: "purple",
    Unknown: "blue",
  };
</script>

<GeoJSON data={$blockGj}>
  <FillLayer
    {...layerId("block")}
    paint={{
      "fill-color": caseHelper("kind", colors, "red"),
      "fill-opacity": 0.8,
    }}
  >
    <Popup openOn="hover" let:data>
      <p>{data.properties.kind}</p>
    </Popup>
  </FillLayer>
</GeoJSON>

<div>
  Blocks
  <button on:click={clear} disabled={!active}>Clear</button>
  <button on:click={findAll}>Find all</button>
</div>
{#if active}
  <Legend rows={Object.entries(colors)} />
{/if}
