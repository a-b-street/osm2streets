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
    box-sizing: border-box;
  }

  .left {
    width: 25%;
    height: 100vh;
    overflow: scroll;
  }
  .main {
    width: 75%;
  }
</style>
