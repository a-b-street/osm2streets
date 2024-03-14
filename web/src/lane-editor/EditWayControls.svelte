<script lang="ts">
  import AllEdits from "./AllEdits.svelte";
  import Tags from "./Tags.svelte";

  export let way: bigint | null;

  // TODO Is this layering and event plumbing nice?
  let allEdits: AllEdits;
</script>

<AllEdits bind:this={allEdits} />
<hr />

{#if way}
  <a href="http://openstreetmap.org/way/{way}" target="_blank">Way {way}</a>

  {#key way}
    <Tags {way} on:editedWay={(way) => allEdits.handleEditedWay(way)} />
  {/key}
{:else}
  Click a road to edit
{/if}
