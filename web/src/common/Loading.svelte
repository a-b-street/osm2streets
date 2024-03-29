<script lang="ts">
  export let msg: string | null = null;
</script>

{#if msg}
  <div>{msg}</div>
{/if}

<style>
  div {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 999;

    color: white;
    font-size: 32px;
  }
</style>
