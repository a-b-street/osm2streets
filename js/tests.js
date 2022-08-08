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
  }
}
