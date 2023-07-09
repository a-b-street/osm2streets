<script lang="ts">
  export let settings = {
    debug_each_step: false,
    dual_carriageway_experiment: false,
    sidepath_zipping_experiment: false,
    inferred_sidewalks: false,
    osm2lanes: false,
  };
</script>

<details>
  <summary>Processing Options</summary>
  <label>
    <input type="checkbox" bind:checked={settings.debug_each_step} />
    Debug each transformation step
  </label>
  <label>
    <input
      type="checkbox"
      bind:checked={settings.dual_carriageway_experiment}
    />
    Enable dual carriageway experiment
  </label>
  <label>
    <input
      type="checkbox"
      bind:checked={settings.sidepath_zipping_experiment}
    />
    Enable sidepath zipping experiment
  </label>
  <label>
    <input type="checkbox" bind:checked={settings.osm2lanes} />
    Enable osm2lanes experiment
  </label>
  <div>
    Sidewalks:
    <label>
      <input
        type="radio"
        bind:group={settings.inferred_sidewalks}
        value={false}
      />
      use mapped footways
    </label>
    <label>
      <input
        type="radio"
        bind:group={settings.inferred_sidewalks}
        value={true}
      />
      infer on roads
    </label>
  </div>
</details>

<style>
  details {
    border: 2px groove silver;
    padding: 0.75em;
    margin-top: 0.5em;
  }
  details > summary:first-of-type {
    position: relative;
    width: fit-content;
    top: -1.6em;
    margin-bottom: -1.6em;
    padding: 0 2px;
    background: var(--bg-color);

    font-weight: bold;
  }

  div,
  label {
    display: block;
    margin-inline-start: 1em;
    text-indent: -1em;
  }
  div > label {
    white-space: nowrap;
  }
</style>
