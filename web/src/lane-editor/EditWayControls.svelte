<script lang="ts">
  import type { FeatureWithProps } from "../common/utils";
  import type { GeoJSON, Polygon } from "geojson";
  import { Layer, network } from "../common";
  import AllEdits from "./AllEdits.svelte";
  import Tags from "./Tags.svelte";

  export let clickedLane: FeatureWithProps<Polygon> | null;

  // TODO Is this layering and event plumbing nice?
  let allEdits: AllEdits;

  let way: bigint | null = null;
  let gj: GeoJSON | undefined = undefined;

  let layerStyle = {
    type: "fill",
    paint: {
      "fill-color": "red",
      "fill-opacity": 0.3,
    },
  };

  $: {
    way = null;
    gj = undefined;

    if (clickedLane) {
      let ways = JSON.parse(clickedLane.properties.osm_way_ids);
      if (ways.length != 1) {
        window.alert(
          "This road doesn't match up with one OSM way; you can't edit it",
        );
      } else {
        way = BigInt(ways[0]);
        gj = JSON.parse($network!.getGeometryForWay(way));
      }
    }
  }
</script>

<AllEdits bind:this={allEdits} />
<hr />

{#if way}
  <a href="http://openstreetmap.org/way/{way}" target="_blank">Way {way}</a>

  <Layer source="current-way" {gj} {layerStyle} />

  {#key way}
    <Tags {way} on:editedWay={(way) => allEdits.handleEditedWay(way)} />
  {/key}
{:else}
  Click a road to edit
{/if}
