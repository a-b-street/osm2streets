<script lang="ts">
  import type { Polygon } from "geojson";
  import type { FeatureWithProps } from "../common/utils";
  import { network } from "../common";

  // Note the input is maplibre's GeoJSONFeature, which stringifies nested properties
  export let data: FeatureWithProps<Polygon> | undefined;
  export let close: () => boolean;

  let props = data!.properties;

  function collapse() {
    $network!.collapseIntersection(props.id);
    $network = $network;
    close();
  }
</script>

<h2>Intersection #{props.id}</h2>
<p><u>Kind</u>: {props.intersection_kind}</p>
<p><u>Control</u>: {props.control}</p>
<p><u>Movements</u>: {props.movements}</p>
{#if props.crossing}
  {@const crossing = JSON.parse(props.crossing)}
  <p>
    <u>Crossing</u>: {crossing.kind}
    {#if crossing.has_island}
      (with an island){/if}
  </p>
{/if}

<p>
  <u>OSM nodes</u>:
  {#each JSON.parse(props.osm_node_ids) as id}
    <a href="https://www.openstreetmap.org/node/{id}" target="_blank">{id}</a>,
  {/each}
</p>

<div>
  <button type="button" on:click={collapse}>Collapse intersection</button>
</div>
