<script lang="ts">
  import type { Feature, GeoJSON } from "geojson";
  import type {
    GeoJSONSource,
    MapLayerMouseEvent,
    MapMouseEvent,
  } from "maplibre-gl";
  import { onDestroy, onMount } from "svelte";
  import { map } from "./store";
  import { emptyGeojson, getLayerZorder } from "./utils";

  // Input
  export let source: string;
  export let show = true;
  export let gj: GeoJSON | undefined;
  // TODO LayerSpecification doesn't work
  export let layerStyle: any;
  // Make objects hoverable and clickable. The caller should do something with
  // ["feature-state", "hover"] and ["feature-state", "clicked"]
  export let interactive = false;

  // Output
  export let hoveredFeature: Feature | null = null;
  export let clickedFeature: Feature | null = null;

  // An opaque ID assigned by this component
  let hoverId: number | undefined;
  let clickedId: number | undefined;

  let layer = `${source}-layer`;

  onMount(() => {
    fixIDs();
    $map!.addSource(source, {
      type: "geojson",
      data: gj ?? emptyGeojson(),
    });
    $map!.addLayer(
      {
        id: layer,
        source,
        ...layerStyle,
      },
      getLayerZorder(layer)
    );
    // We may need to hide initially
    if (!show) {
      $map!.setLayoutProperty(layer, "visibility", "none");
    }

    if (interactive) {
      // Configure hovering
      $map!.on("mousemove", layer, onMouseMove);
      $map!.on("mouseleave", layer, onMouseLeave);
      // Configure clicking
      $map!.on("click", onClick);
    }
  });

  onDestroy(() => {
    if (interactive) {
      $map!.off("mousemove", onMouseMove);
      $map!.off("mouseleave", onMouseLeave);
      $map!.off("click", onClick);
      unhover();
    }

    if ($map!.getLayer(layer)) {
      $map!.removeLayer(layer);
    }
    $map!.removeSource(source);
  });

  // Make sure every feature has a numeric ID. But don't use generateId,
  // because we need to hold onto the full GeoJSON we give to MapLibre,
  // because there's not a way to get it back out later.
  function fixIDs() {
    // If we've only been passed in one feature, don't bother with IDs
    if (gj && "features" in gj) {
      for (let [idx, f] of gj.features.entries()) {
        // 0 is problematic
        f.id = idx + 1;
      }
    }
    // DON'T do 'gj = gj', to avoid the reactive statement below running spuriously
  }

  $: {
    let sourceObj = $map.getSource(source);
    if (sourceObj) {
      console.log(`GeoJSON data for ${source} changed, updating`);
      fixIDs();
      (sourceObj as GeoJSONSource).setData(gj ?? emptyGeojson());

      hoveredFeature = null;
      hoverId = undefined;
      clickedFeature = null;
      clickedId = undefined;
    }
  }

  // Show/hide
  $: {
    if ($map!.getLayer(layer)) {
      $map!.setLayoutProperty(layer, "visibility", show ? "visible" : "none");
    }
  }

  function onMouseMove(e: MapLayerMouseEvent) {
    if (e.features.length > 0 && hoverId != e.features[0].id) {
      unhover();
      // generateId means this'll be a number
      hoverId = e.features[0].id as number;
      $map!.setFeatureState({ source, id: hoverId }, { hover: true });

      // Per
      // https://maplibre.org/maplibre-gl-js-docs/api/map/#map#queryrenderedfeatures,
      // array and object properties aren't returned. So find the original
      // object in the source.
      if ("features" in gj) {
        hoveredFeature = gj.features.find((f) => f.id == hoverId)!;
      }
    }
  }

  function onMouseLeave() {
    unhover();
    hoveredFeature = null;
    hoverId = undefined;
  }

  function onClick(e: MapMouseEvent) {
    if (clickedFeature !== null) {
      $map!.setFeatureState({ source, id: clickedId }, { clicked: false });
    }

    let features = $map!.queryRenderedFeatures(e.point, { layers: [layer] });
    if (features.length == 1) {
      clickedId = features[0].id as number;
      $map!.setFeatureState({ source, id: clickedId }, { clicked: true });

      if ("features" in gj) {
        clickedFeature = gj.features.find((f) => f.id == clickedId)!;
      }
    } else {
      clickedFeature = null;
      clickedId = undefined;
    }
  }

  function unhover() {
    if (hoverId !== undefined) {
      $map!.setFeatureState({ source, id: hoverId }, { hover: false });
    }
  }
</script>
