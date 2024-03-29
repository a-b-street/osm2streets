<script lang="ts">
  let origPage = window.location.pathname.split("/").pop() || "index.html";
  let page = origPage;

  function gotoPage() {
    if (page != origPage) {
      // Preserve the camera and query parameters
      // (When both are defined, the camera will be clobbered by RenderBoundary)
      window.location.href = `${page}${window.location.search}${window.location.hash}`;
    }
  }
</script>

<div>
  App:
  <select bind:value={page} on:change={gotoPage}>
    <option value="index.html">Street Explorer</option>
    <option value="lane_editor.html">Lane editor</option>
  </select>
</div>
