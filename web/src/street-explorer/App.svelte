<script lang="ts">
  import { Popup } from "svelte-maplibre";
  import init from "osm2streets-js";
  import { onMount } from "svelte";
  import AppSwitcher from "../AppSwitcher.svelte";
  import { Geocoder, Layout, Map, TopLeftPanel } from "../osm2streets-svelte";
  import ImportControls from "../osm2streets-svelte/import/ImportControls.svelte";
  import InteractiveLayers from "../osm2streets-svelte/layers/InteractiveLayers.svelte";
  import RenderBoundary from "../osm2streets-svelte/layers/RenderBoundary.svelte";
  import RenderIntersectionMarkings from "../osm2streets-svelte/layers/RenderIntersectionMarkings.svelte";
  import RenderIntersectionPolygons from "../osm2streets-svelte/layers/RenderIntersectionPolygons.svelte";
  import RenderLaneMarkings from "../osm2streets-svelte/layers/RenderLaneMarkings.svelte";
  import RenderLanePolygons from "../osm2streets-svelte/layers/RenderLanePolygons.svelte";
  import IntersectionPopup from "./IntersectionPopup.svelte";
  import LanePopup from "./LanePopup.svelte";

  onMount(async () => {
    await init();
  });
</script>

<Layout>
  <div slot="left">
    <h1>osm2streets Street Explorer</h1>
    <AppSwitcher />
    <p>
      Understanding OSM streets &amp; intersections with
      <a href="https://github.com/a-b-street/osm2streets" target="_blank"
        >osm2streets</a
      >
      and <a href="https://gitlab.com/LeLuxNet/Muv" target="_blank">Muv</a>.
    </p>
    <hr />

    <ImportControls />
  </div>
  <div slot="main">
    <Map>
      <TopLeftPanel>
        <RenderBoundary />
        <RenderIntersectionPolygons>
          <Popup openOn="click" let:data let:close>
            <IntersectionPopup {data} {close} />
          </Popup>
        </RenderIntersectionPolygons>
        <RenderIntersectionMarkings />
        <RenderLanePolygons>
          <Popup openOn="click" let:data let:close>
            <LanePopup {data} {close} />
          </Popup>
        </RenderLanePolygons>
        <RenderLaneMarkings />
        <hr />
        <InteractiveLayers />
      </TopLeftPanel>
      <Geocoder />
    </Map>
  </div>
</Layout>
