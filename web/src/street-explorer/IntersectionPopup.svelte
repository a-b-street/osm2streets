<script lang="ts">
  import type { Feature } from "geojson";
  import { network } from "../osm2streets-svelte/store";

  export let intersection: Feature;

  let props = structuredClone(intersection.properties);
  delete props.osm_node_ids;

  function collapse() {
    $network.collapseIntersection(intersection.properties.id);
    $network = $network;
  }
</script>

<pre>{JSON.stringify(props, null, "  ")}</pre>

<div>
  OSM nodes:
  {#each intersection.properties.osm_node_ids as id}
    <a href="https://www.openstreetmap.org/node/{id}" target="_blank">{id}</a>,
  {/each}
</div>

<div>
  <button type="button" on:click={collapse}>Collapse intersection</button>
</div>
