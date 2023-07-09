<script lang="ts">
  import { network } from "../osm2streets-svelte/store";
  import { downloadGeneratedFile } from "../osm2streets-svelte/utils";

  let editedWays: Set<bigint> = new Set();

  export function handleEditedWay(e) {
    editedWays.add(e.detail);
    editedWays = editedWays;
  }

  function downloadOsc() {
    let contents = `<osmChange version="0.6" generator="osm2streets">\n`;
    contents += `<create/>\n`;
    contents += `<modify>\n`;
    for (let id of editedWays) {
      contents += $network.wayToXml(id);
      contents += "\n";
    }
    contents += `</modify>\n`;
    contents += `</osmChange>`;

    downloadGeneratedFile("lane_edits.osc", contents);
  }
</script>

<div>{editedWays.size} ways edited</div>
<button type="button" on:click={downloadOsc}>Download .osc</button>
