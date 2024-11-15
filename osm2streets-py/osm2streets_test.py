# osm2streets_test.py

#Import necessary modules
import osm2streets_python
import json
import os

# Suppress error logs from osm2streets
os.environ["RUST_LOG"] = "off"
print("osm2streets_python imported successfully.")

#Load OSM XML data as bytes
osm_file_path = "../tests/src/neukolln/input.osm"
with open(osm_file_path, "rb") as file:
    osm_input = file.read()
print(f"Loaded {len(osm_input)} bytes from {osm_file_path}.")

# Load the GeoJSON boundary for clipping
with open("../tests/src/neukolln/boundary.json", "r") as f:
    clip_pts_geojson = json.load(f)

# Convert the JSON object to a string format for input
clip_pts_geojson = json.dumps(clip_pts_geojson)

#Define input options for PyStreetNetwork
input_options = {
    "debug_each_step": False,
    "dual_carriageway_experiment": False,
    "sidepath_zipping_experiment": False,
    "inferred_sidewalks": True,
    "inferred_kerbs": True,
    "date_time": None,
    "override_driving_side": "Right"
}
input_options_json = json.dumps(input_options)

#Initialize PyStreetNetwork
try:
    network = osm2streets_python.PyStreetNetwork(osm_input, clip_pts_geojson, input_options_json)
    print("PyStreetNetwork instance created successfully! - package installed correctly")
except Exception as e:
    print(f"Error during initialization: {e}")
