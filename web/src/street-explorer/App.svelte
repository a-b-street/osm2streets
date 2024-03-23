<script lang="ts">
  import { Popup } from "svelte-maplibre";
  import init from "osm2streets-js";
  import { onMount } from "svelte";
  import AppSwitcher from "../AppSwitcher.svelte";
  import RenderCycle from "./RenderCycle.svelte";
  import {
    StreetView,
    ThemePicker,
    BasemapPicker,
    Geocoder,
    Layout,
    Map,
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

  let wasmReady = false;
  onMount(async () => {
    await init();
    wasmReady = true;
  });

  // Some of the layer contents need to be under the Map component for Svelte
  // context to work, but the controls themselves should be in the
  // sidebar. This trick moves the DOM node around.
  let controlsContents: HTMLDivElement | null = null;
  let controlsContainer: HTMLSpanElement;
  $: if (controlsContents && controlsContainer) {
    controlsContainer.innerHTML = "";
    controlsContainer.appendChild(controlsContents);
  }
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

    {#if wasmReady}
      <ImportControls />
    {/if}

    <br />

    <details open>
      <summary>Layers</summary>
      <div bind:this={controlsContainer} />
    </details>
  </div>
  <div slot="main">
    <Map>
      <div bind:this={controlsContents}>
        <RenderBoundary />
        <RenderIntersectionPolygons hoverCursor="pointer">
          <Popup openOn="click" popupClass="popup" let:data let:close>
            {#key data}
              <IntersectionPopup {data} {close} />
            {/key}
          </Popup>
        </RenderIntersectionPolygons>
        <RenderIntersectionMarkings />
        <RenderLanePolygons hoverCursor="pointer">
          <Popup openOn="click" popupClass="popup" let:data let:close>
            {#key data}
              <LanePopup {data} {close} />
            {/key}
          </Popup>
        </RenderLanePolygons>
        <RenderLaneMarkings />

        <hr />

        <DynamicMovementArrows />
        <DynamicRoadOrdering />
        <DynamicConnectedRoads />
        <RenderCycle />

        <hr />

        <BasemapPicker />
        <ThemePicker />
        <StreetView />
      </div>
      <Geocoder />
    </Map>
  </div>
</Layout>

<style>
  :global(.popup .maplibregl-popup-content) {
    border: 1px solid black;
    overflow: auto;
    max-height: 50vh;
    max-width: 30vw;
  }

  details {
    border: 1px solid black;
    padding: 4px;
  }
</style>
