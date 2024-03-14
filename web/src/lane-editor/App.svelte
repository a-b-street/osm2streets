<script lang="ts">
  import init from "osm2streets-js";
  import { onMount } from "svelte";
  import AppSwitcher from "../AppSwitcher.svelte";
  import { Geocoder, Layout, Map } from "../osm2streets-svelte";
  import ImportControls from "../osm2streets-svelte/import/ImportControls.svelte";
  import EditWayControls from "./EditWayControls.svelte";
  import RenderBoundary from "../osm2streets-svelte/layers/RenderBoundary.svelte";
  import RenderIntersectionMarkings from "../osm2streets-svelte/layers/RenderIntersectionMarkings.svelte";
  import RenderIntersectionPolygons from "../osm2streets-svelte/layers/RenderIntersectionPolygons.svelte";
  import RenderLaneMarkings from "../osm2streets-svelte/layers/RenderLaneMarkings.svelte";
  import RenderLanePolygons from "../osm2streets-svelte/layers/RenderLanePolygons.svelte";

  onMount(async () => {
    await init();
  });
</script>

<Layout>
  <div slot="left">
    <h1>osm2streets lane editor</h1>
    <AppSwitcher />
    <p>
      Improve OSM lane tagging with
      <a href="https://github.com/a-b-street/osm2streets" target="_blank"
        >osm2streets</a
      >
      and <a href="https://gitlab.com/LeLuxNet/Muv" target="_blank">Muv</a>.
    </p>
    <hr />

    <ImportControls />
    <hr />

    <EditWayControls />
    <hr />

    <div>
      <strong>Warnings:</strong>
      <ul>
        <li><strong>This tool is an early experiment</strong></li>
        <li>Don't use this tool without understanding OSM tagging</li>
        <li>Be careful around sidepaths, footways, and dual carriageways</li>
        <li>Don't edit a way that's partly clipped</li>
      </ul>
    </div>
  </div>
  <div slot="main">
    <Map>
      <div style:display="none">
        <RenderBoundary />
        <RenderIntersectionPolygons />
        <RenderIntersectionMarkings />
        <RenderLanePolygons />
        <RenderLaneMarkings />
      </div>
      <Geocoder />
    </Map>
  </div>
</Layout>
