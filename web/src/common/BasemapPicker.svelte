<script lang="ts">
  import { basemap } from "./store";

  // TODO Z-ordering won't work when we change, because layerId() doesn't get recalculated
</script>

<div>
  <label>
    Basemap:
    <select bind:value={$basemap}>
      <option value="dataviz">MapTiler Dataviz</option>
      <option value="streets">MapTiler Streets</option>
      <option value="hybrid">MapTiler Satellite</option>
      <option value="blank">Blank</option>
    </select>
  </label>
</div>
