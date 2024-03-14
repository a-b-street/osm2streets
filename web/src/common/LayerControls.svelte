<script lang="ts">
  import type { GeoJSON } from "geojson";
  import { downloadGeneratedFile } from "./utils";

  export let gj: GeoJSON | undefined;
  export let name: string;
  export let show: boolean;
  export let downloadable = true;

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
    <button type="button" on:click={download} disabled={!gj}>Download</button>
  {/if}
</div>
