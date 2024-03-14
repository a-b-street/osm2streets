<script lang="ts">
  import type { Polygon } from "geojson";
  import type { FeatureWithProps } from "../common/utils";
  import { network } from "../common";

  // Note the input is maplibre's GeoJSONFeature, which stringifies nested properties
  export let data: FeatureWithProps<Polygon> | undefined;
  export let close: () => boolean;

  let props = structuredClone(data!.properties);
  props.allowed_turns = JSON.parse(props.allowed_turns);
  delete props.osm_way_ids;
  let osm_way_ids = JSON.parse(data!.properties.osm_way_ids);
  let muv = JSON.parse(data!.properties.muv ?? "{}");
  delete props.muv;

  function collapse() {
    $network!.collapseShortRoad(props.road);
    $network = $network;
    close();
  }

  function zip() {
    $network!.zipSidepath(props.road);
    $network = $network;
    close();
  }

  // TODO Hack because TS doesn't work below
  let networkValue = $network!;
</script>

<pre>{JSON.stringify(props, null, "  ")}</pre>

{#if muv}
  <details>
    <summary>Full Muv JSON</summary>
    <pre>{JSON.stringify(muv, null, "  ")}</pre>
  </details>
{/if}

<hr />

<u>OSM ways:</u>
<ul>
  {#each osm_way_ids as id}
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
