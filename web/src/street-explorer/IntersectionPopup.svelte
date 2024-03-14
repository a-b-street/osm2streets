<script lang="ts">
  import type { Polygon } from "geojson";
  import type { FeatureWithProps } from "../osm2streets-svelte/utils";
  import { network } from "../osm2streets-svelte";

  // Note the input is maplibre's GeoJSONFeature, which stringifies nested properties
  export let data: FeatureWithProps<Polygon> | undefined;

  let props = structuredClone(data!.properties);
  props.movements = JSON.parse(props.movements);
  delete props.osm_node_ids;
  let osm_node_ids = JSON.parse(data!.properties.osm_node_ids);

  function collapse() {
    $network!.collapseIntersection(props.id);
    $network = $network;
  }
</script>

<pre>{JSON.stringify(props, null, "  ")}</pre>

<div>
  OSM nodes:
  {#each osm_node_ids as id}
    <a href="https://www.openstreetmap.org/node/{id}" target="_blank">{id}</a>,
  {/each}
</div>

<div>
  <button type="button" on:click={collapse}>Collapse intersection</button>
</div>
