<div class="left">
  <slot name="left" />
</div>
<div class="main">
  <slot name="main" />
</div>

<style>
  div {
    float: left;
    padding: 8px;
  }

  .left {
    width: 25%;
    height: 100vh;
    overflow: scroll;
  }
  .main {
    /* TODO 75% wraps?! */
    width: 73%;
  }
</style>
