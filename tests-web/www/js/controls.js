import { downloadGeneratedFile } from "./files.js";

// TODO A Leaflet control isn't the right abstraction. We want a popup modal
// that totally blocks the map and other things. Things like disabling
// settingsButton to stop multiple of these controls is a total hack.
const SettingsControl = L.Control.extend({
  // TODO Centered would be great. https://github.com/Leaflet/Leaflet/issues/8358
  options: {
    position: "topleft",
  },
  onAdd: function (map) {
    const checkbox1 = makeCheckbox(
      "debugEachStep",
      "Debug each transformation step\n",
      this.options.app.importSettings.debugEachStep,
      (checked) => {
        this.options.app.importSettings.debugEachStep = checked;
      }
    );
    const checkbox2 = makeCheckbox(
      "dualCarriagewayExperiment",
      "Enable dual carriageway experiment\n",
      this.options.app.importSettings.dualCarriagewayExperiment,
      (checked) => {
        this.options.app.importSettings.dualCarriagewayExperiment = checked;
      }
    );
    const checkbox3 = makeCheckbox(
      "cycletrackSnappingExperiment",
      "Enable cycletrack snapping experiment\n",
      this.options.app.importSettings.cycletrackSnappingExperiment,
      (checked) => {
        this.options.app.importSettings.cycletrackSnappingExperiment = checked;
      }
    );

    const button = makeButton("Confirm");
    button.onclick = () => {
      this.remove();
      document.getElementById("settingsButton").disabled = false;
    };

    var group = makeDiv([checkbox1, checkbox2, checkbox3, button]);
    group.style = "background: black; padding: 10px;";
    L.DomEvent.disableClickPropagation(group);
    return group;
  },
});

export const makeSettingsControl = function (app) {
  return new SettingsControl({ app: app });
};

export class Layer {
  // Should only be created by LayerGroup.addLayer. That'll set an appropriate extra field with the actual data.
  constructor(name, enabled) {
    this.name = name;
    this.enabled = enabled;
    this.data = null;
  }

  getData() {
    if (this.data == null) {
      console.log(`Lazily evaluating layer ${this.name}`);
      this.data = this.lazilyMakeData();
      this.lazilyMakeData = null;
    }
    return this.data;
  }
}

// Manages a bunch of layers that can be independently toggled. The entire
// group has a toggle to enable or disable everything.
export class LayerGroup {
  constructor(name, map) {
    this.name = name;
    this.layers = [];
    this.enabled = true;
    this.map = map;
  }

  addLayer(name, layerData, { enabled = true } = {}) {
    var layer = new Layer(name, enabled);
    layer.data = layerData;
    this.layers.push(layer);

    if (enabled) {
      this.map.addLayer(layer.getData());
    }
  }

  addLazyLayer(name, lazilyMakeData) {
    const enabled = false;
    var layer = new Layer(name, enabled);
    layer.lazilyMakeData = lazilyMakeData;
    this.layers.push(layer);
  }

  // Updates the map, but doesn't re-render any controls
  setEnabled(enabled) {
    this.enabled = enabled;
    for (const layer of this.layers) {
      layer.enabled = enabled;

      if (enabled) {
        this.map.addLayer(layer.getData());
      } else if (layer.data != null) {
        this.map.removeLayer(layer.data);
      }
    }
  }

  renderControls() {
    var members = [
      makeCheckbox(
        this.name,
        `<u>${this.name}</u>`,
        this.enabled,
        (checked) => {
          this.setEnabled(checked);

          // Rerender
          document.getElementById(this.name).replaceWith(this.renderControls());
        }
      ),
    ];
    for (const layer of this.layers) {
      const entry = makeCheckbox(
        this.name + "_" + layer.name,
        layer.name,
        layer.enabled,
        (checked) => {
          layer.enabled = checked;
          if (checked) {
            this.map.addLayer(layer.getData());
          } else {
            this.map.removeLayer(layer.getData());
          }
        }
      );

      const download = makeButton("Download");
      download.onclick = () => {
        downloadGeneratedFile(
          `${layer.name}.geojson`,
          JSON.stringify(layer.getData().toGeoJSON())
        );
      };
      entry.appendChild(download);
      members.push(entry);
    }
    var div = makeDiv(members);
    div.id = this.name;
    return div;
  }

  remove() {
    for (const layer of this.layers) {
      // If a lazy layer wasn't evaluated, there's nothing to remove from the map
      if (layer.data != null) {
        this.map.removeLayer(layer.data);
      }
    }
  }
}

// Contains a bunch of LayerGroups, displays exactly one at a time, and can
// scroll through the sequence.
export class SequentialLayerGroup {
  constructor(name, groups) {
    this.name = name;
    this.groups = groups;
    this.current = 0;

    // Start with only the first enabled
    for (var i = 0; i < this.groups.length; i++) {
      this.groups[i].setEnabled(i == this.current);
    }
  }

  renderControls() {
    const prev = makeButton("Previous");
    prev.disabled = this.current == 0;
    prev.onclick = () => {
      this.groups[this.current].setEnabled(false);
      this.current -= 1;
      this.groups[this.current].setEnabled(true);

      // Rerender
      document.getElementById(this.name).replaceWith(this.renderControls());
    };

    const label = document.createTextNode(
      `${this.current + 1} / ${this.groups.length}`
    );

    const next = makeButton("Next");
    next.disabled = this.current == this.groups.length - 1;
    next.onclick = () => {
      this.groups[this.current].setEnabled(false);
      this.current += 1;
      this.groups[this.current].setEnabled(true);

      // Rerender
      document.getElementById(this.name).replaceWith(this.renderControls());
    };

    var row1 = L.DomUtil.create("u");
    row1.innerText = this.name;
    const row2 = makeDiv([prev, label, next]);
    const column = makeDiv([
      row1,
      row2,
      this.groups[this.current].renderControls(),
    ]);
    column.id = this.name;
    return column;
  }

  remove() {
    this.groups[this.current].setEnabled(false);
  }
}

// Manages a list of LayerGroups or SequentialLayerGroups.
//
// The interface required of each member: renderControls(), remove(), name
const LayerControl = L.Control.extend({
  options: {
    position: "bottomleft",
  },
  onAdd: function (map) {
    return this.renderControls();
  },

  renderControls: function () {
    var members = [];
    for (const group of this.options.groups) {
      members.push(group.renderControls());
      members.push(L.DomUtil.create("br"));
    }
    members.pop();
    var group = makeDiv(members);
    group.style = "background: black; padding: 10px;";
    L.DomEvent.disableClickPropagation(group);
    return group;
  },

  getGroup: function (groupName) {
    for (const group of this.options.groups) {
      if (group.name == groupName) {
        return group;
      }
    }
    throw `Can't find group ${groupName}`;
  },

  // May return an underlying layer ({name, layer, enabled}) or a SequentialLayerGroup
  getLayer: function (groupName, layerName) {
    for (const group of this.options.groups) {
      if (group.name == groupName) {
        for (const layer of group.layers) {
          if (layer.name == layerName) {
            return layer;
          }
        }
      }
    }

    throw `Can't find group ${groupName} with layer ${layerName}`;
  },

  removeGroups: function (predicate) {
    var keep = [];
    for (const group of this.options.groups) {
      if (predicate(group.name)) {
        group.remove();
      } else {
        keep.push(group);
      }
    }
    this.options.groups = keep;

    L.DomUtil.empty(this.getContainer());
    this.getContainer().appendChild(this.renderControls());
  },

  addGroup: function (group) {
    this.options.groups.push(group);

    L.DomUtil.empty(this.getContainer());
    this.getContainer().appendChild(this.renderControls());
  },
});

export const makeLayerControl = function (app) {
  return new LayerControl({ app: app, groups: [] });
};

// Helpers

function makeCheckbox(id, label, enabled, callback) {
  var checkbox = L.DomUtil.create("input");
  checkbox.id = id;
  checkbox.type = "checkbox";
  checkbox.checked = enabled;
  checkbox.onclick = () => {
    callback(checkbox.checked);
  };

  var labelElem = L.DomUtil.create("label");
  labelElem.for = id;
  labelElem.innerHTML = label;

  return makeDiv([checkbox, labelElem]);
}

function makeDiv(members) {
  var div = L.DomUtil.create("div");
  for (const child of members) {
    div.appendChild(child);
  }
  return div;
}

function makeButton(label) {
  const button = L.DomUtil.create("button");
  button.type = "button";
  button.innerHTML = label;
  return button;
}
