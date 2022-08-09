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

    const button = L.DomUtil.create("button");
    button.type = "button";
    button.innerHTML = "Confirm";
    button.onclick = () => {
      this.remove();
      document.getElementById("settingsButton").disabled = false;
    };

    var group = makeDiv([checkbox1, checkbox2, button]);
    group.style = "background: black; padding: 10px;";
    L.DomEvent.disableClickPropagation(group);
    return group;
  },
});

export const makeSettingsControl = function (app) {
  return new SettingsControl({ app: app });
};

export class LayerGroup {
  constructor(name, map) {
    this.name = name;
    this.layers = [];
    this.enabled = true;
    this.map = map;
  }

  addLayer(name, layer, { enabled = true } = {}) {
    this.layers.push({ name, enabled, layer });
  }

  // Doesn't re-render
  setEnabled(enabled) {
    this.enabled = enabled;
    for (const layer of this.layers) {
      layer.enabled = enabled;
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
            this.map.addLayer(layer.layer);
          } else {
            this.map.removeLayer(layer.layer);
          }
        }
      );

      const download = L.DomUtil.create("button");
      download.type = "button";
      download.innerHTML = "Download";
      download.onclick = () => {
        downloadGeneratedFile(
          `${layer.name}.geojson`,
          JSON.stringify(layer.layer.toGeoJSON())
        );
      };
      entry.appendChild(download);
      members.push(entry);

      // This is maybe an odd time to sync this state
      if (layer.enabled) {
        this.map.addLayer(layer.layer);
      } else {
        this.map.removeLayer(layer.layer);
      }
    }
    var div = makeDiv(members);
    div.id = this.name;
    return div;
  }
}

// Manages a list of groups
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
    }
    var group = makeDiv(members);
    group.style = "background: black; padding: 10px;";
    L.DomEvent.disableClickPropagation(group);
    return group;
  },

  getLayer: function (groupName, layerName) {
    for (const group of this.options.groups) {
      if (group.name == groupName) {
        for (const layer of group.layers) {
          if (layer.name == layerName) {
            return layer.layer;
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
        for (const layer of group.layers) {
          group.map.removeLayer(layer.layer);
        }
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
