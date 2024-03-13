<script lang="ts">
  import type { Polygon } from "geojson";
  import { network } from "../osm2streets-svelte";
  import type { FeatureWithProps } from "../osm2streets-svelte/utils";

  export let lane: FeatureWithProps<Polygon>;

  let props = structuredClone(lane.properties);
  delete props.osm_way_ids;
  delete props.muv;

  function collapse() {
    $network!.collapseShortRoad(lane.properties.road);
    $network = $network;
  }

  function zip() {
    $network!.zipSidepath(lane.properties.road);
    $network = $network;
  }

  // TODO Hack because TS doesn't work below
  let networkValue = $network!;
</script>

<pre>{JSON.stringify(props, null, "  ")}</pre>

{#if lane.properties.muv}
  <details>
    <summary>Full Muv JSON</summary>
    <pre>{JSON.stringify(lane.properties.muv, null, "  ")}</pre>
  </details>
{/if}

<hr />

<u>OSM ways:</u>
<ul>
  {#each lane.properties.osm_way_ids as id}
    <li>
      <a href="https://www.openstreetmap.org/way/{id}" target="_blank">{id}</a>
      <details>
        <summary>See OSM tags</summary>
        <pre>{networkValue.getOsmTagsForWay(BigInt(id))}</pre>
      </details>
    </li>
  {/each}
</ul>

<div>
  <button type="button" on:click={collapse}>Collapse short road</button>
  <button type="button" on:click={zip}>Zip side-path</button>
</div>
