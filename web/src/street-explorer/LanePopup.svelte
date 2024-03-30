<script lang="ts">
  import type { Polygon } from "geojson";
  import type { FeatureWithProps } from "../common/utils";
  import { network } from "../common";
  import { blockGj } from "./stores";

  // Note the input is maplibre's GeoJSONFeature, which stringifies nested properties
  export let data: FeatureWithProps<Polygon> | undefined;
  export let close: () => boolean;

  let props = data!.properties;
  // TODO Hack because TS doesn't work below
  let networkValue = $network!;

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

  function findBlock(left: boolean, sidewalks: boolean) {
    try {
      blockGj.set(JSON.parse($network!.findBlock(props.road, left, sidewalks)));
      close();
    } catch (err) {
      window.alert(err);
    }
  }
</script>

<h2>Lane {props.index} of Road {props.road}</h2>
<p><u>Type</u>: {props.type}</p>
<p><u>Direction</u>: {props.direction}</p>
<p><u>Width</u>: {props.width}m</p>
<p><u>Speed limit</u>: {props.speed_limit}</p>
<p><u>Allowed turns</u>: {props.allowed_turns}</p>
<p><u>Layer</u>: {props.layer}</p>

{#if props.muv}
  <details>
    <summary>Full Muv JSON</summary>
    <pre>{JSON.stringify(JSON.parse(props.muv), null, "  ")}</pre>
  </details>
{/if}

<hr />

<p><u>OSM ways:</u></p>
{#each JSON.parse(props.osm_way_ids) as id}
  <p>
    <a href="https://www.openstreetmap.org/way/{id}" target="_blank">{id}</a>
  </p>
  <details>
    <summary>See OSM tags</summary>
    <table>
      <tbody>
        {#each Object.entries(JSON.parse(networkValue.getOsmTagsForWay(BigInt(id)))) as [key, value]}
          <tr><td>{key}</td><td>{value}</td></tr>
        {/each}
      </tbody>
    </table>
  </details>
{/each}

<div>
  <button type="button" on:click={collapse}>Collapse short road</button>
  <button type="button" on:click={zip}>Zip side-path</button>
</div>
<div>
  <button type="button" on:click={() => findBlock(true, false)}
    >Find block on left</button
  >
  <button type="button" on:click={() => findBlock(false, false)}
    >Find block on right</button
  >
</div>
<div>
  <button type="button" on:click={() => findBlock(true, true)}
    >Trace sidewalks on left</button
  >
  <button type="button" on:click={() => findBlock(false, true)}
    >Trace sidewalks on right</button
  >
</div>

<style>
  td {
    border: solid 1px black;
    padding: 3px;
  }
</style>
