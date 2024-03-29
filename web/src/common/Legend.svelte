<script lang="ts">
  // (Label, color) pairs
  export let rows: [string, string][];
</script>

<ul>
  {#each rows as [label, color]}
    <li>
      <span style:background={color} />
      {label}
    </li>
  {/each}
</ul>

<style>
  span {
    display: block;
    float: left;
    height: 16px;
    width: 30px;
    margin-right: 5px;
    border: 1px solid #999;
  }
</style>
