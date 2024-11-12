osm2streets Python Module Documentation

The osm2streets_python module provides Python bindings for the osm2streets library using pyo3, allowing users to interact with OpenStreetMap (OSM) data and convert it into street network representations. The main classes, PyStreetNetwork and PyDebugStreets, offer methods for creating, transforming, and exporting street networks in GeoJSON format.

Imports

	•	Standard Library:
	•	BTreeMap and BTreeSet (ordered collections)
	•	Once (used for one-time initialization of the logger)
	•	Crates:
	•	abstutil, chrono, geom, serde, serde_json (for utilities, datetime, geometry, and serialization/deserialization)
	•	pyo3 (to create Python bindings)
	•	osm2streets (core library for processing OSM data into street networks)

Data Structures

ImportOptions

The ImportOptions struct configures how a street network is imported and transformed. It includes options for debugging, experiments with dual carriageways, and sidewalk inferences.

Fields

	•	debug_each_step: Enable detailed debugging information at each transformation step.
	•	dual_carriageway_experiment: Enable dual carriageway merging.
	•	sidepath_zipping_experiment: Enable sidepath zipping transformations.
	•	inferred_sidewalks: Infer sidewalks automatically.
	•	inferred_kerbs: Infer curbs automatically.
	•	date_time: Optional timestamp for import context.
	•	override_driving_side: Set to "Left" or "Right" to specify the driving side.

Classes

PyStreetNetwork

PyStreetNetwork represents the main street network structure, constructed from raw OSM data and transformation configurations.

Methods

	•	new(osm_input, clip_pts_geojson, input): Initializes a new PyStreetNetwork.
	•	osm_input: Byte array representing OSM data.
	•	clip_pts_geojson: Optional GeoJSON string defining the area to clip.
	•	input: JSON string parsed as ImportOptions.
	•	to_geojson_plain(): Exports the entire street network as a plain GeoJSON.
	•	to_lane_polygons_geojson(): Exports lane polygons as a GeoJSON.
	•	to_lane_markings_geojson(): Exports lane markings as a GeoJSON.
	•	to_intersection_markings_geojson(): Exports intersection markings as a GeoJSON.

PyDebugStreets

PyDebugStreets provides debugging utilities for inspecting and visualizing specific street network elements.

Methods

	•	get_label(): Returns a label for the debug street.
	•	to_debug_geojson(): Exports debug information as GeoJSON, if available.

Module Setup

The osm2streets_python module contains the following classes:

	•	PyStreetNetwork
	•	PyDebugStreets

These classes expose methods to manage, transform, and export street network data, allowing for detailed control over street network modeling from OSM data directly within Python.