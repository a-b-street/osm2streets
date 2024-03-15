<script lang="ts">
  import { Popup } from "svelte-maplibre";
  import init from "osm2streets-js";
  import { onMount } from "svelte";
  import AppSwitcher from "../AppSwitcher.svelte";
  import {
    BasemapPicker,
    Geocoder,
    Layout,
    Map,
    TopLeftPanel,
  } from "../common";
  import ImportControls from "../common/import/ImportControls.svelte";
  import RenderBoundary from "../common/layers/RenderBoundary.svelte";
  import RenderIntersectionMarkings from "../common/layers/RenderIntersectionMarkings.svelte";
  import RenderIntersectionPolygons from "../common/layers/RenderIntersectionPolygons.svelte";
  import RenderLaneMarkings from "../common/layers/RenderLaneMarkings.svelte";
  import RenderLanePolygons from "../common/layers/RenderLanePolygons.svelte";
  import DynamicConnectedRoads from "../common/layers/DynamicConnectedRoads.svelte";
  import DynamicMovementArrows from "../common/layers/DynamicMovementArrows.svelte";
  import DynamicRoadOrdering from "../common/layers/DynamicRoadOrdering.svelte";

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

        <DynamicMovementArrows />
        <DynamicRoadOrdering />
        <DynamicConnectedRoads />

        <hr />

        <BasemapPicker />
      </TopLeftPanel>
      <Geocoder />
    </Map>
  </div>
</Layout>
