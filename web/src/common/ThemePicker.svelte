<script lang="ts">
  import { theme } from "./store";
</script>

<div>
  <label>
    Theme:
    <select bind:value={$theme}>
      <option value="debug">Debug</option>
      <option value="realistic">Realistic</option>
    </select>
  </label>
</div>
