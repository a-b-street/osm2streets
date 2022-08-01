/*import {
  makeOsmLayer,
  makePlainGeoJsonLayer,
  makeDetailedGeoJsonLayer,
  makeDotLayer,
} from "./layers.js";
import { JsStreetNetwork } from "./osm2streets-js/osm2streets_js.js";*/

export function loadTests() {
  // FIXME: load the list of tests from the server
  const testNames = [
    "arizona_highways",
    "aurora_sausage_link",
    "borough_sausage_links",
    "bristol_contraflow_cycleway",
    "bristol_sausage_links",
    "i5_exit_ramp",
    "kingsway_junction",
    "montlake_roundabout",
    "oneway_loop",
    "perth_peanut_roundabout",
    "perth_stretched_lights",
    "seattle_slip_lane",
    "seattle_triangle",
    "service_road_loop",
    "taipei",
    "tempe_light_rail",
    "tempe_split",
  ];

  // Add all of the test cases to the list.
  const listNode = window.document.getElementById("test-list");
  for (const t of testNames) {
    const li = listNode.appendChild(window.document.createElement("li"));
    const a = li.appendChild(window.document.createElement("a"));
    // Here we encode the test name in the URL to be read elsewhere.
    a.href = "?test=" + t;
    a.innerHTML = t;

    /*const reimport = li.appendChild(window.document.createElement("button"));
    reimport.type = "button";
    reimport.innerHTML = "Reimport";
    reimport.onclick = async function () {
      for (const x of currentLayers) {
        map.removeLayer(x);
      }
      currentLayers = [];

      try {
        const osmXML = await loadFile(`tests/${t}/input.osm`);
        const driving_side = JSON.parse(await loadFile(`tests/${t}/test.json`))[
          "driving_side"
        ];

        const network = new JsStreetNetwork(osmXML, {
          driving_side: driving_side,
        });
        makePlainGeoJsonLayer(network.toGeojsonPlain()).addTo(map);
        makeDetailedGeoJsonLayer(network.toGeojsonDetailed()).addTo(map);
      } catch (err) {
        window.alert(`Reimport failed: ${err}`);
      }
    };*/
  }
}
