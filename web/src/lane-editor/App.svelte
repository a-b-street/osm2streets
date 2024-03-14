<script lang="ts">
  import type { FeatureWithProps } from "../common/utils";
  import type { Polygon } from "geojson";
  import init from "osm2streets-js";
  import { onMount } from "svelte";
  import AppSwitcher from "../AppSwitcher.svelte";
  import { Geocoder, Layout, Map } from "../common";
  import ImportControls from "../common/import/ImportControls.svelte";
  import EditWayControls from "./EditWayControls.svelte";
  import RenderBoundary from "../common/layers/RenderBoundary.svelte";
  import RenderIntersectionMarkings from "../common/layers/RenderIntersectionMarkings.svelte";
  import RenderIntersectionPolygons from "../common/layers/RenderIntersectionPolygons.svelte";
  import RenderLaneMarkings from "../common/layers/RenderLaneMarkings.svelte";
  import RenderLanePolygons from "../common/layers/RenderLanePolygons.svelte";

  onMount(async () => {
    await init();
  });

  let clickedLane: FeatureWithProps<Polygon> | null = null;
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

    <EditWayControls {clickedLane} />
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
        <RenderLanePolygons
          on:click={(e) => {
            // @ts-expect-error Need to typecast
            clickedLane = e.detail.features[0];
          }}
        />
        <RenderLaneMarkings />
      </div>
      <Geocoder />
    </Map>
  </div>
</Layout>
