<script lang="ts">
  import type { Feature, Polygon } from "geojson";
  import { JsStreetNetwork } from "osm2streets-js";
  import BuiltInSelector from "../osm_input/BuiltInSelector.svelte";
  import OverpassSelector from "../osm_input/OverpassSelector.svelte";
  import type { OsmSelection } from "../osm_input/types";
  import {
    boundaryGj as boundaryGjStore,
    map,
    network as networkStore,
    importCounter,
  } from "../store";
  import Osm2streetsSettings from "./Osm2streetsSettings.svelte";
  import { bbox, downloadGeneratedFile, Loading } from "svelte-utils";

  // This component manages a state machine for importing OSM data (from
  // Overpass or built-in files) and letting the user change import settings. It
  // sets the global network and boundaryGj stores.

  interface Settings {
    debug_each_step: boolean;
    dual_carriageway_experiment: boolean;
    sidepath_zipping_experiment: boolean;
    inferred_sidewalks: boolean;
    inferred_kerbs: boolean;
    date_time: string | undefined;
  }

  type Imported =
    | { kind: "nothing" }
    | { kind: "loading"; msg: string }
    | {
        kind: "done";
        boundaryGj: Feature<Polygon>;
        osmXml: string;
        network: JsStreetNetwork;
      };

  let imported: Imported = { kind: "nothing" };

  let settings: Settings;
  let overpassSelector: OverpassSelector;
  let testCase =
    new URLSearchParams(window.location.search).get("test") || "none";

  // Only update when settings change
  function updateForSettings(settings: Settings) {
    if (imported.kind == "done" && settings) {
      console.log("Settings changed, re-importing");
      importNetwork(imported.osmXml, imported.boundaryGj);
    }
  }
  $: updateForSettings(settings);

  $: {
    // Track the testCase in the URL
    let url = new URL(window.location.href);
    if (testCase != "none") {
      url.searchParams.set("test", testCase);
    } else {
      url.searchParams.delete("test");
    }
    window.history.pushState({}, "", url);
  }

  function importNetwork(osmXml: string, boundaryGj: Feature<Polygon>) {
    try {
      imported = { kind: "loading", msg: "Running osm2streets" };
      let network = new JsStreetNetwork(
        // TODO Can we avoid this?
        new Uint8Array(new TextEncoder().encode(osmXml)),
        JSON.stringify(boundaryGj),
        settings,
      );
      imported = {
        kind: "done",
        boundaryGj,
        osmXml,
        network,
      };

      $importCounter++;
      networkStore.set(imported.network);
      boundaryGjStore.set(imported.boundaryGj);
    } catch (err: any) {
      window.alert(err.toString());
      imported = { kind: "nothing" };
    }
  }

  function download() {
    // This type-check is always true; the button only appears sometimes
    if (imported.kind === "done") {
      // TODO If we have a name for the imported area, use that
      downloadGeneratedFile("osm.xml", imported.osmXml);
    }
  }

  function update() {
    if (imported.kind === "done") {
      overpassSelector.importPolygon(imported.boundaryGj);
    }
  }

  function resetView() {
    if (imported.kind === "done") {
      $map!.fitBounds(bbox(imported.boundaryGj), {
        animate: false,
        padding: 10,
      });
    }
  }

  function load(e: CustomEvent<OsmSelection>) {
    importNetwork(e.detail.osmXml, e.detail.boundaryGj);
    testCase = e.detail.testCase;
  }

  function resetToNone(e: CustomEvent<void>) {
    imported = { kind: "nothing" };
    $importCounter++;
    networkStore.set(null);
    boundaryGjStore.set(null);
  }

  function error(e: CustomEvent<string>) {
    window.alert(e.detail);
    imported = { kind: "nothing" };
  }

  function loading(e: CustomEvent<string>) {
    imported = { kind: "loading", msg: e.detail };
  }
</script>

<div class="controls">
  <OverpassSelector
    bind:this={overpassSelector}
    map={$map}
    on:loading={loading}
    on:load={load}
    on:resetToNone={resetToNone}
    on:error={error}
  />
  <fieldset>
    <legend>
      <BuiltInSelector
        bind:testCase
        on:loading={loading}
        on:load={load}
        on:resetToNone={resetToNone}
        on:error={error}
      />
    </legend>

    {#if imported.kind === "nothing"}
      <p>Choose an option above to import somewhere</p>
    {:else if imported.kind === "loading"}
      <Loading loading={imported.msg} />
    {:else if imported.kind === "done"}
      <div>
        <button type="button" on:click={update}>Update OSM data</button>
        <button type="button" on:click={download}>Download osm.xml</button>
        <button type="button" on:click={resetView}>Reset view</button>
      </div>
    {/if}
  </fieldset>

  <Osm2streetsSettings bind:settings />
</div>

<style>
  .controls > :global(*) {
    margin-bottom: 8px;
  }

  fieldset {
    border: 1px groove black;
    margin: 0;
  }
</style>
