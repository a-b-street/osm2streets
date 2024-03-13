<script lang="ts">
  import init from "osm2streets-js";
  import { onMount } from "svelte";
  import AppSwitcher from "../AppSwitcher.svelte";
  import { Geocoder, Layout, Map, TopLeftPanel } from "../osm2streets-svelte";
  import ImportControls from "../osm2streets-svelte/import/ImportControls.svelte";
  import InteractiveLayers from "../osm2streets-svelte/layers/InteractiveLayers.svelte";
  import MainLayers from "../osm2streets-svelte/layers/MainLayers.svelte";
  import Popups from "./Popups.svelte";

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
      > and <a href="https://gitlab.com/LeLuxNet/Muv" target="_blank">Muv</a>.
    </p>
    <hr />

    <ImportControls />
  </div>
  <div slot="main">
    <Map>
      <TopLeftPanel>
        <MainLayers />
        <hr />
        <InteractiveLayers />
      </TopLeftPanel>
      <Popups />
      <Geocoder />
    </Map>
  </div>
</Layout>
