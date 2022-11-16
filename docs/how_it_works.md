# How it works

As of November 2022, and probably incomplete. This describes how the codebase currently works, not necessarily how we want it to.

## The model

At its heart, a graph of roads and intersections. (Roads lead between exactly two intersections -- "road segments" might be more precise.) Roads have their lanes listed from left-to-right, with a type, width, and direction. Note osm2streets doesn't model bidirectional lanes yet -- sidewalks and shared center turn lanes are either forwards or backwards right now, and something downstream interprets them in a special way. (osm2lanes has more nuance, but isn't used in osm2streets yet.)

Intersections have a `ControlType` -- stop signs, traffic signals, uncontrolled, etc. This is orthogonal to `IntersectionComplexity` and `ConflictType`... TODO, narrow down valid combinations and give examples. MultiConnection vs Merge, please!

### IDs

Roads and intersections are currently identified by OSM IDs, but this will soon change to opaque IDs. As we merge, split, or fix things, the model diverges from what's in OSM. Instead, roads and intersections will keep a list of OSM IDs that went into the osm2streets object.

## Import walkthrough

### streets_reader

`osm_to_street_network` is the main function, taking raw input OSM XML, an optional boundary clipping polygon, and some options, and returning a `StreetNetwork`. Some callers (A/B Street) repeat the logic of this method and add in extra bits (for adding in other sources of parking and elevation data).

1.   Extract raw info from OSM
2.   Split ways
3.   Clip to the boundary
4.   Match extra stuff

Extraction is straightforward. Since OSM ways often cross many intersections, they don't form a graph yet, so the split step finds nodes common to multiple ways and declares those intersections. Very small roundabouts also get collapsed to a single point here (a hack!). Raw turn restriction data and traffic signal nodes are also matched to a road. After this step, we have the first cut of a `StreetNetwork`. There are no movements filled out and geometry is almost exactly what OSM has.

But from this point, roads do have their lanes filled out, parsed from OSM tags. That currently uses `osm2streets/src/lanes/classic.rs`, but will use a separate project `osm2lanes` in the future.

Clipping takes the boundary polygon (which should be passed in explicitly, but can also just be the bounding box around the input XML) and removes roads totally out of bounds. Roads crossing the boundary will get clipped to the boundary, and that intersection will be marked as a border.

The final step here takes raw crossing and barrier nodes and matches them to roads. This representation of those features is very early stages and will definitely evolve more.

### Transformations

The `StreetNetwork` is techncially usable at this point, but it's still very close to OSM -- which is both under-specified (lane width is almost never tagged, but we need to render something) and imprecise. The rest of the magic happens by calling `apply_transformations`. This performs the specified steps in order. `apply_transformations_stepwise_debugging` can be used by UIs to preserve the intermediate `StreetNetwork` after each step, for debugging and understanding the transformations.

The caller explicitly lists the transformations they want, in order. `standard_for_clipped_areas` is a good list to start with. Splitting things into explicit steps like this is good:

- Users can opt into experimental steps
- Some callers may not want to deviate too far from OSM, because they're using osm2streets to edit directly
- Incremental debugging

But it's also confusing in a few ways:

- Roads and intersections both contain derived state. When we modify something, we may need to re-run some transformations. For example, after collapsing sausage links, a road's trimmed road geometry changes, so we may need to detect and collapse short roads again. Effects from one transformation may need to propagate to adjacent roads and intersections. The dataflow is implicit; the caller must deal with it manually.
- Some transformations fill out state that's pretty fundamental, like intersection complexity and movements. This maybe shouldn't be expressed as a transformation and should happen more upfront, like how `Road::new` immediately determines lanes from OSM tags.

## Operations on a StreetNetwork

Some of the below transformations do something complicated worth calling out. Likely this code shouldn't be in `transform/`.

### Collapsing an intersection

`collapse_intersection` removes an intersection that has only two roads connected to it. Transformations below describe when this is called. The operation itself mostly just stitches together the geometry of the two roads in a straightforward way. It fixes turn restrictions (which should get simpler with opaque IDs). The caller is responsible for fixing up lanes between the two roads -- or rather, making sure they match up compatibly in the first place.

### Collapsing a short road

`collapse_short_road` removes a road, then combines the two intersections into one.

It first calculates trimmed intersection geometry for the two intersections. On each connected road (besides the short one being collapsed, of course), we store the trimming distance in `trim_roads_for_merging`, so that later the intersection geometry algorithm can follow a special case for the single merged intersection.

Most of the complexity here comes from fixing up IDs and turn restrictions. Opaque IDs likely make this a much simpler operation!

## The transformations

TODO: before/after pictures

These're in no particular order. (But that's confusing; they should be)

### ClassifyIntersections

For each intersection, this calculates vehicle movements (at the granularity of roads, not individual lanes). Then based on those movements, which ones conflict, and the number of connecting roads, we classify the intersection. This classification is only used as debug rendering right now, but will likely help later transformations by filtering when some heuristics should apply.

### TrimDeadendCycleways

We sometimes wind up with short dead-end roads that're nice to remove. One example is short service roads -- I recall these happening in Seattle, maybe related to how driveways are tagged there. Another is also a bit Seattle-specific -- when we try to import separate cyclepaths but not footways there, there are some dangling "stubs" of cycleway leftover sometimes.

### SnapCycleways (experimental)

TODO

### RemoveDisconnectedRoads

Often a clipping boundary will bring in some roads that aren't connected to the main street network. This partitions the graph into connected components and removes all but the largest.

### FindShortRoads

This just looks for "short" roads that should get later collapsed. Anything in OSM explicitly tagged `junction=intersection` will get collapsed, and in fact, this transformation artificially creates this tag to signal to the later transformation.

There are a few heuristics, all experimental:

- if the trimmed road geometry is short
- if the road is short and connects two traffic signals
- the short `~~` piece in "dog-leg" intersections:

```text                                                                                      
      |
      |
---X~~X----
   |
   |
```

### CollapseShortRoads

This calls the collapse operation on anything marked by `FindShortRoads`. Complexity again just comes from lack of opaque IDs.

### CollapseDegenerateIntersections

A "degenerate" intersection (`IntersectionComplexity::Connection`) has only two roads connected. Sometimes that intersection can be collapsed and the two roads joined. Currently this happens:

- between two cycleways
- when the lanes match and only "unimportant" OSM tags differ

There are special cases documented in the code.

### CollapseSausageLinks

A "simpl esausage link" is a dual carriageway that split very briefly and then re-joins, with no intermediate roads. These are collapsed into one road between the intersections, with a barrier lane inserted in the middle. The code is well-documented and better reference.

### ShrinkOverlappingRoads

This is a hack to make dual carriageways drawn close together in OSM look half-reasonable, before we successfully merge them. It looks for road polygons that physically overlap, then just halves all lane widths. It doesn't attempt to shift the road center or re-apply placement tags.

### MergeDualCarriageways (experimental)

TODO. Explain branches and bridges.

### GenerateIntersectionGeometry

Along with `ClassifyIntersections`, this is the most important transformation (and maybe should be expressed differently). For every intersection, it runs through [this algorithm](https://a-b-street.github.io/docs/tech/map/geometry/index.html). Road center-lines get "trimmed" back from the intersection, and the intersection gets a polygon.
