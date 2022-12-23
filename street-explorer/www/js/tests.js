export function loadTests() {
  // FIXME: load the list of tests from the server
  const testNames = [
    "dog_leg",
    "arizona_highways",
    "aurora_sausage_link",
    "borough_sausage_links",
    "bristol_contraflow_cycleway",
    "bristol_sausage_links",
    "cycleway_rejoin_road",
    "fremantle_placement",
    "i5_exit_ramp",
    "kingsway_junction",
    "leeds_cycleway",
    "montlake_roundabout",
    "northgate_dual_carriageway",
    "oneway_loop",
    "perth_peanut_roundabout",
    "perth_stretched_lights",
    "quad_intersection",
    "roosevelt_cycletrack",
    "seattle_slip_lane",
    "seattle_triangle",
    "service_road_loop",
    "st_georges_cycletrack",
    "taipei",
    "tempe_light_rail",
    "tempe_split",
    "tiny_loop",
  ];

  // Add all the test cases to the list.
  const listNode = window.document.getElementById("test-list");
  for (const t of testNames) {
    const a = listNode.appendChild(window.document.createElement("option"));
    // Here we encode the test name in the URL to be read elsewhere.
    a.value = t;
    a.innerHTML = t;
  }

  listNode.onchange = (ev) => {
    const val = ev.currentTarget.value;
    const q = new URLSearchParams(location.search);
    if (val) {
      q.set("test", val);
    } else {
      q.delete("test");
    }
    location.search = q.toString();
  };
}
