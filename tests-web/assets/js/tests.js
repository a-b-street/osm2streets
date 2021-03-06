import { makeOsmLayer, makeJsonLayer, makeDotLayer  } from './layers.js';

/** Load and display all the files associated with a test case. */
export const makeOpenTest = (map) => async (name) => {
    const prefix = `src/${name}/`;
    const input = loadFile(prefix + 'input.osm');
    const rawMap = loadFile(prefix + 'raw_map.json');
    const network = loadFile(prefix + 'road_network.dot');

    const rawMapLayer = makeJsonLayer(await rawMap);
    const bounds = rawMapLayer.getBounds();
    map.fitBounds(bounds, { animate: false });
    map.addLayer(rawMapLayer);

    const inputLayer = makeOsmLayer(await input);
    map.addLayer(inputLayer);

    const networkLayer = await makeDotLayer(await network, { bounds })
    map.addLayer(networkLayer);

    L.control.layers({}, {"Geometry": rawMapLayer, "OSM input": inputLayer, "Graph": networkLayer}).addTo(map);

    // TODO store a reference to the layers so they can be cleaned up when wanted.
}

const loadFile = name => fetch(name).then(body => body.text()).catch(err => console.warn(err));

export const loadTests = async () => {
    // FIXME: load the list of tests from the server
    const testNames = [
        'arizona_highways',
        'aurora_sausage_link',
        'borough_sausage_links',
        'bristol_contraflow_cycleway',
        'bristol_sausage_links',
        'i5_exit_ramp',
        'kingsway_junction',
        'montlake_roundabout',
        'perth_peanut_roundabout',
        'perth_stretched_lights',
        'seattle_slip_lane',
        'seattle_triangle',
        'service_road_loop',
        'taipei',
        'tempe_light_rail',
        'tempe_split',
    ];

    // Add all of the test cases to the list.
    const listNode = window.document.getElementById('test-list');
    for (const t of testNames) {
        const li = listNode.appendChild(window.document.createElement('li'));
        const a = li.appendChild(window.document.createElement('a'));
        // Here we encode the test name in the URL to be read elsewhere.
        a.href = '?test=' + t;
        a.innerHTML = t;
    }
}
