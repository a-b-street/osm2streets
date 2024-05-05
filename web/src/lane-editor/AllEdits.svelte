<script lang="ts">
  import { network, importCounter } from "../common";
  import { downloadGeneratedFile } from "svelte-utils";

  let editedWays: Set<bigint> = new Set();

  // Drop edits when changing areas
  $: if ($importCounter > 0) {
    editedWays = new Set();
  }

  export function handleEditedWay(e: CustomEvent<bigint>) {
    editedWays.add(e.detail);
    editedWays = editedWays;
  }

  function downloadOsc() {
    let contents = `<osmChange version="0.6" generator="osm2streets">\n`;
    contents += `<create/>\n`;
    contents += `<modify>\n`;
    for (let id of editedWays) {
      contents += $network!.wayToXml(id);
      contents += "\n";
    }
    contents += `</modify>\n`;
    contents += `</osmChange>`;

    downloadGeneratedFile("lane_edits.osc", contents);
  }
</script>

<div>{editedWays.size} ways edited</div>
<button type="button" on:click={downloadOsc} disabled={editedWays.size == 0}
  >Download .osc</button
>
