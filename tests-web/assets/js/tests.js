import { makeOsmLayer, makeJsonLayer, makeDotLayer  } from './layers.js';

/** Load and display all the files associated with a test case. */
export const makeOpenTest = (map) => async (name) => {
    const prefix = `/src/${name}/`;
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

    console.debug({ rawMapLayer, bounds, inputLayer, networkLayer, map });
}

const loadFile = name => fetch(name).then(body => body.text()).catch(err => console.warn(err));
