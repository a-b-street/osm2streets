<script lang="ts">
  import init from "osm2streets-js";
  import { onMount } from "svelte";
  import AppSwitcher from "../AppSwitcher.svelte";
  import ImportControls from "../osm2streets-svelte/import/ImportControls.svelte";
  import InteractiveLayers from "../osm2streets-svelte/layers/InteractiveLayers.svelte";
  import MainLayers from "../osm2streets-svelte/layers/MainLayers.svelte";
  import Layout from "../osm2streets-svelte/Layout.svelte";
  import Map from "../osm2streets-svelte/Map.svelte";
  import TopLeftPanel from "../osm2streets-svelte/TopLeftPanel.svelte";
  import Popups from "./Popups.svelte";

  onMount(async () => {
    await init();
  });
</script>

<Layout>
  <div slot="left">
    <h1>osm2streets Street Explorer</h1>
    <AppSwitcher />
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
    </Map>
  </div>
</Layout>
