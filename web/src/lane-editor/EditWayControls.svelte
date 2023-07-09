<script lang="ts">
  import type { GeoJSON } from "geojson";
  import Layer from "../osm2streets-svelte/Layer.svelte";
  import { clickedLane, network } from "../osm2streets-svelte/store";
  import AllEdits from "./AllEdits.svelte";
  import Tags from "./Tags.svelte";

  // TODO Is this layering and event plumbing nice?
  let allEdits: AllEdits;

  let way: bigint | null = null;
  let gj: GeoJSON | null = null;

  let layerStyle = {
    type: "fill",
    paint: {
      "fill-color": "red",
      "fill-opacity": 0.3,
    },
  };

  $: {
    way = null;
    gj = null;

    if ($clickedLane) {
      if ($clickedLane.properties.osm_way_ids.length != 1) {
        window.alert(
          "This road doesn't match up with one OSM way; you can't edit it"
        );
      } else {
        way = BigInt($clickedLane.properties.osm_way_ids[0]);
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
