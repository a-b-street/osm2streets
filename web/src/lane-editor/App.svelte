<script lang="ts">
  import { map } from "../common/store";
  import { GeoJSON, FillLayer, type LayerClickInfo } from "svelte-maplibre";
  import { emptyGeojson, layerId } from "../common/utils";
  import { network, StreetView } from "../common";
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

  let wasmReady = false;
  onMount(async () => {
    await init();
    wasmReady = true;
  });

  let way: bigint | null = null;
  // When changing areas, unset any selected way
  network.subscribe((value) => {
    way = null;
  });

  $: wayGj = way
    ? JSON.parse($network!.getGeometryForWay(way))
    : emptyGeojson();

  function onClickLane(e: CustomEvent<LayerClickInfo>) {
    let ways = JSON.parse(e.detail.features[0].properties!.osm_way_ids);
    if (ways.length != 1) {
      window.alert(
        "This road doesn't match up with one OSM way; you can't edit it",
      );
    } else {
      way = BigInt(ways[0]);
    }
  }
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

    {#if wasmReady}
      <ImportControls />
    {/if}
    <hr />

    <EditWayControls {way} />
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

    {#if $map}
      <StreetView />
    {/if}
  </div>
  <div slot="main">
    <Map>
      <div style:display="none">
        <RenderBoundary />
        <RenderIntersectionPolygons />
        <RenderIntersectionMarkings />
        <RenderLanePolygons hoverCursor="pointer" on:click={onClickLane} />
        <RenderLaneMarkings />
        <GeoJSON data={wayGj}>
          <FillLayer
            {...layerId("current-way")}
            paint={{
              "fill-color": "red",
              "fill-opacity": 0.3,
            }}
          />
        </GeoJSON>
      </div>
      <Geocoder />
    </Map>
  </div>
</Layout>
