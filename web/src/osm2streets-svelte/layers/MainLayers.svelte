<script lang="ts">
  import RenderBoundary from "./RenderBoundary.svelte";
  import RenderIntersectionMarkings from "./RenderIntersectionMarkings.svelte";
  import RenderIntersectionPolygons from "./RenderIntersectionPolygons.svelte";
  import RenderLaneMarkings from "./RenderLaneMarkings.svelte";
  import RenderLanePolygons from "./RenderLanePolygons.svelte";
</script>

<RenderBoundary />
<RenderIntersectionPolygons />
<RenderIntersectionMarkings />
<RenderLanePolygons />
<RenderLaneMarkings />
