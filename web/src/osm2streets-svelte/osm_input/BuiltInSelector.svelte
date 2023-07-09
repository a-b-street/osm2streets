<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import type { OsmSelection } from "./types";

  const dispatch = createEventDispatcher<{
    loading: string;
    load: OsmSelection;
    resetToNone: void;
    error: string;
  }>();

  export let testCase;
  let list: string[] = [];

  onMount(async () => {
    let resp = await fetch(`${import.meta.env.BASE_URL}/tests.json`);
    list = await resp.json();

    // Initially load a test case?
    if (testCase != "none") {
      await reload();
    }
  });

  async function reload() {
    if (testCase == "none") {
      dispatch("resetToNone");
      return;
    }

    try {
      dispatch("loading", "Loading built-in boundary and OSM XML");
      let polygonResp = await fetch(
        `${import.meta.env.BASE_URL}/tests/${testCase}/boundary.json`
      );
      let polygon = await polygonResp.json();
      // Test input is always a FeatureCollection with one object. For uniformity...
      let boundaryGj = polygon.features[0];

      let osmResp = await fetch(
        `${import.meta.env.BASE_URL}/tests/${testCase}/input.osm`
      );
      let osmXml = await osmResp.text();

      dispatch("load", {
        testCase,
        boundaryGj,
        osmXml,
      });
    } catch (err) {
      dispatch("error", err.toString());
    }
  }

  // TODO Hack to have this here, but the rest in ImportControls. We need
  // access to reload here, though... unless we export it?
  async function popState() {
    let prev =
      new URLSearchParams(window.location.search).get("test") || "none";
    console.log(
      `Navigated back in history -- changing test case from ${testCase} to ${prev}`
    );
    testCase = prev;
    await reload();
  }
</script>

<select bind:value={testCase} on:change={reload}>
  <option value="none">None</option>
  {#each list as x}
    <option value={x}>{x}</option>
  {/each}
</select>

<svelte:window on:popstate={popState} />
