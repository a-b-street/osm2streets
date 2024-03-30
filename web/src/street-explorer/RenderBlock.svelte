<script lang="ts">
  import { caseHelper, layerId, emptyGeojson } from "../common/utils";
  import {
    hoverStateFilter,
    Popup,
    LineLayer,
    FillLayer,
    GeoJSON,
  } from "svelte-maplibre";
  import { showingBundles, blockGj } from "./stores";
  import { network, Legend } from "../common";

  $: active = $blockGj.features.length > 0;

  function clear() {
    blockGj.set(emptyGeojson());
  }

  function findAll(sidewalks: boolean) {
    blockGj.set(JSON.parse($network!.findAllBlocks(sidewalks)));
    showingBundles.set(sidewalks);
  }

  let blockColors = {
    LandUseBlock: "grey",
    RoadAndSidewalk: "green",
    RoadAndCycleLane: "orange",
    CycleLaneAndSidewalk: "yellow",
    DualCarriageway: "purple",
    Unknown: "blue",
  };
  let bundleColors = {
    LandUseBlock: "grey",
    RoadBundle: "green",
    IntersectionBundle: "orange",
  };

  $: colors = $showingBundles ? bundleColors : blockColors;
</script>

<GeoJSON data={$blockGj} generateId>
  <FillLayer
    {...layerId("block")}
    filter={["==", ["get", "type"], "block"]}
    manageHoverState
    paint={{
      "fill-color": caseHelper("kind", colors, "red"),
      "fill-opacity": hoverStateFilter(0.8, 0.4),
    }}
  >
    <Popup openOn="hover" let:data>
      <p>{data.properties.kind}</p>
    </Popup>
  </FillLayer>

  <LineLayer
    {...layerId("block-debug")}
    filter={["!=", ["get", "type"], "block"]}
    paint={{
      "line-color": [
        "case",
        ["==", ["get", "type"], "member-road"],
        "red",
        "black",
      ],
      "line-width": 5,
    }}
  >
    <Popup openOn="hover" let:data>
      <pre>{JSON.stringify(data.properties, null, "  ")}</pre>
    </Popup>
  </LineLayer>
</GeoJSON>

<div>
  Blocks
  <button on:click={clear} disabled={!active}>Clear</button>
  <button on:click={() => findAll(false)}>Find all blocks</button>
  <button on:click={() => findAll(true)}>Find all sidewalk bundles</button>
</div>
{#if active}
  <Legend rows={Object.entries(colors)} />
{/if}
