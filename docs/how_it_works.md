# How it works

As of November 2022, and probably incomplete. This describes how the codebase currently works, not necessarily how we want it to.

## The model

At its heart, a graph of roads and intersections. A `Road` is a segment of road that leads between exactly two `Intersection`s. An `Intersection`'s kind tells you if it represents a real-life intersection or some other kind of node in the graph. A `MapEdge` connects a single `Road` the edge of the map, a `Terminus` marks an actual dead end. A `Connection` joins multiple `Road`s together where there is no traffic interaction at all, whereas a `Fork` joins multiple roads that merge or diverge without any stop line. Finally, an `IntersectionKind::Intersection` represents everything that you would actually call an "intersection", where traffic merges with, diverges from or crosses other traffic.

Roads have their lanes listed from left-to-right, each with a type, width, and direction. A lane represents any longitudinal feature of a road: travel lanes on the carriageway, separated bike and footpaths, street-side parking, and buffers, medians and verges. Note osm2streets doesn't model bidirectional lanes yet -- sidewalks and shared center turn lanes are either forwards or backwards right now, and something downstream interprets them in a special way.

### IDs

Roads and intersections have opaque (meaningless) IDs. At the very beginning, they map over to exactly one object in OSM, but as the library performs transformations, this mapping becomes more complex. Thus, roads and intersections track a list of OSM objects that they represent.

## Import walkthrough (streets_reader)

`osm_to_street_network` is the main function, taking raw input OSM XML, an optional boundary clipping polygon, and some options, and returning a `StreetNetwork`. Some callers (A/B Street) repeat the logic of this method and add in extra bits (for adding in other sources of parking and elevation data).

1.   Extract raw info from OSM
2.   Split ways
3.   Clip to the boundary
4.   Match extra stuff

Extraction is straightforward. Since OSM ways often cross many intersections, they don't form a graph yet, so the split step finds nodes common to multiple ways and declares those intersections. Very small roundabouts also get collapsed to a single point here (a hack!). Raw turn restriction data and traffic signal nodes are also matched to a road. After this step, we have the first cut of a `StreetNetwork`. There are no movements filled out and geometry is almost exactly what OSM has.

But from this point, roads do have their lanes filled out, parsed from OSM tags. That currently uses `osm2streets/src/lanes/algorithm.rs`.

Clipping takes the boundary polygon (which should be passed in explicitly, but can also just be the bounding box around the input XML) and removes roads totally out of bounds. Roads crossing the boundary will get clipped to the boundary, and that intersection will be marked as a map edge.

The final step here takes raw crossing and barrier nodes and matches them to roads. This representation of those features is very early stages and will definitely evolve more.

See the transformation section below on the rest of the processing.

## Operations on a StreetNetwork

Some complex operations are described here. Some of the code currently lives in `transform/`, but should be moved.

### calculate_movements_and_kind

For each intersection, this calculates vehicle movements (at the granularity of roads, not individual lanes). Then based on those movements, which ones conflict, and the number of connecting roads, we classify the intersection. This classification is only used as debug rendering right now, but will likely help later transformations by filtering when some heuristics should apply.

### Collapsing an intersection

`collapse_intersection` removes an intersection that has only two roads connected to it. Transformations below describe when this is called. The operation itself mostly just stitches together the geometry of the two roads in a straightforward way. It fixes turn restrictions referring to the deleted road. The caller is responsible for fixing up lanes between the two roads -- or rather, making sure they match up compatibly in the first place.

### Collapsing a short road

`collapse_short_road` removes a road, then combines the two intersections into one.

It first calculates trimmed intersection geometry for the two intersections. On each connected road (besides the short one being collapsed, of course), we store the trimming distance in `trim_roads_for_merging`, so that later the intersection geometry algorithm can follow a special case for the single merged intersection.

### update_geometry

This follows [this algorithm](https://a-b-street.github.io/docs/tech/map/geometry/index.html) (outdated!). Road center-lines get "trimmed" back from the intersection, and the intersection gets a polygon.

This process trims every road on both ends. Sometimes the trims overlap and the road disappears entirely. In that case, we mark the road as `internal_junction_road` and remove it entirely with a later pass of `CollapseShortRoads`.

## Transformations

The `StreetNetwork` is techncially usable at this point, but it's still very close to OSM -- which is both under-specified (lane width is almost never tagged, but we need to render something) and imprecise. The rest of the magic happens by calling `apply_transformations`. This performs the specified steps in order. `apply_transformations_stepwise_debugging` can be used by UIs to preserve the intermediate `StreetNetwork` after each step, for debugging and understanding the transformations.

The caller explicitly lists the transformations they want, in order. `standard_for_clipped_areas` is a good list to start with. Splitting things into explicit steps like this is good:

- Users can opt into experimental steps
- Some callers may not want to deviate too far from OSM, because they're using osm2streets to edit directly
- Incremental debugging

But it's also confusing in a few ways:

- Roads and intersections both contain derived state. When we modify something, we may need to re-run some transformations. For example, after collapsing sausage links, a road's trimmed road geometry changes, so we may need to detect and collapse short roads again. Effects from one transformation may need to propagate to adjacent roads and intersections. The dataflow is implicit; the caller must deal with it manually.
- Some transformations fill out state that's pretty fundamental, like road trim distances and intersection geometry. This maybe shouldn't be expressed as a transformation and should happen more upfront, like how `Road::new` immediately determines lanes from OSM tags.

Specific transformations are described below in no particular order. (But that's confusing; they should be)

### SnapCycleways (experimental)

See <https://github.com/a-b-street/osm2streets/pull/61> for now

### RemoveDisconnectedRoads

Often a clipping boundary will bring in some roads that aren't connected to the main street network. This partitions the graph into connected components and removes all but the largest.

### CollapseShortRoads

This calls the collapse operation on anything tagged in OSM as `junction=intersection` and on roads that get trimmed away entirely when calculating intersection geometry.

### CollapseDegenerateIntersections

A "degenerate" intersection has only two roads connected. Sometimes that intersection can be collapsed and the two roads joined. Currently this happens:

- between two cycleways
- when the lanes match and only "unimportant" OSM tags differ

There are special cases documented in the code.

### MergeDualCarriageways (experimental)

TODO. Explain branches and bridges.
