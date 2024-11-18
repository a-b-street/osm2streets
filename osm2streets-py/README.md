
# osm2streets Python module documentation

The osm2streets_python module provides Python bindings for the osm2streets library using [PyO3](https://pyo3.rs), allowing users to interact with OpenStreetMap (OSM) data and convert it into street network representations. 
The main classes of osm2streets (`PyStreetNetwork` and `PyDebugStreets`), offer methods for creating, transforming, and exporting street networks in GeoJSON format.

![alt text](sample_output.png)


## Installation

### If you want to compile it yourself

  1. install latest [Rust](https://www.rust-lang.org/)
  2. install [maturin](https://github.com/PyO3/maturin)
  3. `maturin build --release`
  4. `cd ./target/wheels/`
  5. `pip install [name-wheel].whl` will install it to your local Python

### Development

  1. install [Rust](https://www.rust-lang.org/) (v1.39+)
  2. install [maturin](https://github.com/PyO3/maturin)
  3. `maturin develop`
  4. move to another folder, and `import osm2streets_python` shouldn't return any error



## Usage

You need to install [geopandas](https://geopandas.org) and [geopy](https://geopy.readthedocs.io/en/stable/) in your environment:

```bash
pip install geopandas geopy
```

Once installed, you can verify that the package works by running the osm2streets_test.py in the osm2streets-py folder:

```bash
python3 osm2streets_test.py
```
This should result in the following output: 

```
> python3 osm2streets_test.py
osm2streets_python imported successfully.
Loaded 1466704 bytes from ../tests/src/neukolln/input.osm.
PyStreetNetwork instance created successfully! - package installed correctly
```

Success 🚀 
You've installed the package and are now ready to use its functions. 


## Examples

For some examples have a look at `osm2streets_py_test.ipynb`


### Currently available Classes and Methods in osm2streets_python

**`osm2streets_python.PyStreetNetwork`**

PyStreetNetwork represents the main street network structure, constructed from raw OSM data and transformation configurations.

- **`.new(osm_input, clip_pts_geojson, input)`**: Initializes a new `PyStreetNetwork`.
  - **`.osm_input`**: Byte array representing OSM data.
  - **`.clip_pts_geojson`**: Optional GeoJSON string defining the area to clip.
  - **`.input`**: JSON string parsed as `ImportOptions` to configure the import settings.

- **`.to_geojson_plain()`**: Exports the entire street network as a plain GeoJSON.

- **`.to_lane_polygons_geojson()`**: Exports lane polygons as a GeoJSON.

- **`.to_lane_markings_geojson()`**: Exports lane markings as a GeoJSON.

- **`.to_intersection_markings_geojson()`**: Exports intersection markings as a GeoJSON.

- **`.get_debug_steps()`**: Retrieves a list of `PyDebugStreets` objects representing each debugging step applied to the street network.

- **`.debug_clockwise_ordering_geojson()`**: Exports clockwise ordering information for intersections as a GeoJSON for debugging road connections.

- **`.debug_clockwise_ordering_for_intersection_geojson(intersection)`**: Exports clockwise ordering information for a specific intersection as a GeoJSON.
  - **`intersection`**: Intersection ID to be debugged.

- **`.debug_movements_from_lane_geojson(road, index)`**: Exports movement information from a specific lane as a GeoJSON.
  - **`road`**: Road ID containing the lane.
  - **`index`**: Lane index within the road.

- **`.debug_roads_connected_to_intersection_geojson(i)`**: Exports roads connected to a specified intersection as a GeoJSON.
  - **`i`**: Intersection ID.

- **`.get_osm_tags_for_way(id)`**: Retrieves OSM tags for a specified way as a JSON string.
  - **`id`**: OSM ID of the way.

- **`.to_json()`**: Exports the entire `StreetNetwork` structure as a JSON string.

- **`.get_geometry_for_way(id)`**: Retrieves the buffered geometry of a specified way as a GeoJSON.
  - **`id`**: OSM ID of the way.

- **`.way_to_xml(id)`**: Converts a specified way to an XML representation including its OSM tags.
  - **`id`**: OSM ID of the way.

- **`.find_block(road, left, sidewalks)`**: Finds and exports a block (polygon) on a specific side of the road.
  - **`road`**: Road ID.
  - **`left`**: Boolean indicating if the left side of the road should be used.
  - **`sidewalks`**: Boolean indicating if sidewalks should be included.

- **`.find_all_blocks(sidewalks)`**: Finds and exports all blocks in the network as GeoJSON polygons.
  - **`sidewalks`**: Boolean indicating if sidewalks should be included.

- **`.overwrite_osm_tags_for_way(id, tags)`**: Updates OSM tags for a specific way and applies changes to all affected roads and intersections.
  - **`id`**: OSM ID of the way.
  - **`tags`**: JSON string representing the new OSM tags.

- **`.collapse_short_road(road)`**: Collapses a specified short road by merging it with neighboring segments.
  - **`road`**: Road ID.

- **`.collapse_intersection(intersection)`**: Collapses an intersection if it connects only two roads.
  - **`intersection`**: Intersection ID.

- **`.zip_sidepath(road)`**: Zips a sidepath (e.g., a bike lane or sidewalk) alongside a specified road.
  - **`road`**: Road ID containing the sidepath.

**`osm2streets_python.PyDebugStreets`**

PyDebugStreets provides debugging utilities for inspecting and visualizing specific street network elements.


- **`.get_label()`**: Returns a label for the debug street.
- **`.to_debug_geojson()`**: Exports debug information as GeoJSON, if available.

