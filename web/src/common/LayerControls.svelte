<script lang="ts">
  import type { GeoJSON } from "geojson";
  import { downloadGeneratedFile } from "svelte-utils";

  export let gj: GeoJSON;
  export let name: string;
  export let show: boolean;
  export let downloadable = true;

  $: empty = gj.type == "FeatureCollection" && gj.features.length == 0;

  function download() {
    downloadGeneratedFile(`${name}.geojson`, JSON.stringify(gj));
  }
</script>

<div>
  <label>
    <input type="checkbox" bind:checked={show} />
    {name}
  </label>

  {#if downloadable}
    <button type="button" on:click={download} disabled={empty}>Download</button>
  {/if}
</div>
