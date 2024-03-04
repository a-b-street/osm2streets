<div class="left">
  <slot name="left" />
</div>
<div class="main">
  <slot name="main" />
</div>

<style>
  :global(body) {
    margin: 0;
  }

  div {
    float: left;
    box-sizing: border-box;
    font-family: sans-serif;
  }

  .left {
    padding: 8px 16px;
    width: 25%;
    height: 100vh;
    overflow: scroll;
  }
  .main {
    width: 75%;
  }
</style>
