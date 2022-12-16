use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use abstutil::{deserialize_btreemap, serialize_btreemap, Tags};
use geom::Distance;

/// None corresponds to the native name
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct NamePerLanguage(
    #[serde(
        serialize_with = "serialize_btreemap",
        deserialize_with = "deserialize_btreemap"
    )]
    pub(crate) BTreeMap<Option<String>, String>,
);

impl NamePerLanguage {
    pub fn get(&self, lang: Option<&String>) -> &String {
        // TODO Can we avoid this clone?
        let lang = lang.cloned();
        if let Some(name) = self.0.get(&lang) {
            return name;
        }
        &self.0[&None]
    }

    pub fn new(tags: &Tags) -> Option<NamePerLanguage> {
        let native_name = tags.get("name")?;
        let mut map = BTreeMap::new();
        map.insert(None, native_name.to_string());
        for (k, v) in tags.inner() {
            if let Some(lang) = k.strip_prefix("name:") {
                map.insert(Some(lang.to_string()), v.to_string());
            }
        }
        Some(NamePerLanguage(map))
    }

    pub fn unnamed() -> NamePerLanguage {
        let mut map = BTreeMap::new();
        map.insert(None, "unnamed".to_string());
        NamePerLanguage(map)
    }

    pub fn languages(&self) -> Vec<&String> {
        self.0.keys().flatten().collect()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MapConfig {
    /// If true, driving happens on the right side of the road (USA). If false, on the left
    /// (Australia).
    ///
    /// Note this is calculated by osm2streets! The value passed in is ignored; don't do any work
    /// to set it.
    pub driving_side: DrivingSide,
    pub bikes_can_use_bus_lanes: bool,
    /// If true, roads without explicitly tagged sidewalks may be assigned sidewalks or shoulders.
    /// If false, no inference will occur and separate sidewalks and crossings will be included.
    pub inferred_sidewalks: bool,
    /// Street parking is divided into spots of this length. 8 meters is a reasonable default, but
    /// people in some regions might be more accustomed to squeezing into smaller spaces. This
    /// value can be smaller than the hardcoded maximum car length; cars may render on top of each
    /// other, but otherwise the simulation doesn't care.
    pub street_parking_spot_length: Distance,
    /// If true, turns on red which do not conflict crossing traffic ('right on red') are allowed
    pub turn_on_red: bool,
    /// If true, use experimental osm2lanes for figuring out lanes per road. If false, use the
    /// classic algorithm.
    pub osm2lanes: bool,
    /// OSM railway=rail will be included as light rail if so. Cosmetic only.
    pub include_railroads: bool,
}

impl MapConfig {
    pub fn default() -> Self {
        Self {
            // Just a dummy value that'll be set later
            driving_side: DrivingSide::Right,
            bikes_can_use_bus_lanes: true,
            inferred_sidewalks: true,
            street_parking_spot_length: Distance::meters(8.0),
            turn_on_red: true,
            osm2lanes: false,
            include_railroads: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum DrivingSide {
    Right,
    Left,
}

/// How a lane of travel is interrupted, as it meets another or ends.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TrafficInterruption {
    Uninterrupted,
    Yield,
    Stop,
    Signal,
    DeadEnd,
}
