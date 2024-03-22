<script lang="ts">
  export let settings = {
    debug_each_step: false,
    dual_carriageway_experiment: false,
    sidepath_zipping_experiment: false,
    inferred_sidewalks: false,
    inferred_kerbs: true,
    date_time: undefined as string | undefined,
  };

  let date_time: string;
  $: settings.date_time = date_time && date_time + ":00";
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
  <label>
    <input type="checkbox" bind:checked={settings.inferred_kerbs} />
    Infer kerbs
  </label>
  <div>
    Change the time and date:
    <label>
      <input type="datetime-local" bind:value={date_time} />
    </label>
  </div>
</details>

<style>
  details {
    padding: 0.5em 1em;
    border: 1px groove black;
  }
  summary {
    position: relative;
    width: fit-content;
    top: -1.2em;
    margin-bottom: -1.2em;
    padding: 0 8px;
    background: white;
    font-weight: 600;
  }

  div,
  label {
    display: block;
    margin-inline-start: 1em;
    text-indent: -1em;
  }
  div {
    padding: 0.25em 0;
  }
  div > label {
    white-space: nowrap;
  }
</style>
