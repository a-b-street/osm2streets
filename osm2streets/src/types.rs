use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use abstutil::{deserialize_btreemap, serialize_btreemap, Tags};
use geom::Distance;

use crate::{osm, OriginalRoad, RoadID};

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
        let native_name = tags.get(osm::NAME)?;
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

    /// Enable experimental dog-leg intersection merging
    pub find_dog_legs_experiment: bool,
    /// Experimentally merge these OSM ways
    pub merge_osm_ways: BTreeSet<OriginalRoad>,
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
            find_dog_legs_experiment: false,
            merge_osm_ways: BTreeSet::new(),
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

/// How two lanes of travel conflict with each other.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TrafficConflict {
    Uncontested,
    Diverge,
    Merge,
    Cross,
}

/// What kind of feature an `Intersection` actually represents. Any connection between roads in the
/// network graph is represented by an `Intersection`, but many of them are not traffic
/// "intersections" in the common sense.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum IntersectionKind {
    /// A `Road` ends because the road crosses the map boundary.
    MapEdge,

    /// A single `Road` ends because the actual roadway ends; "the end of the line".
    ///
    /// E.g. turning circles, road end signs, train terminus thingos, ...
    Terminus,

    /// Multiple `Road`s connect but no flow of traffic interacts with any other.
    ///
    /// Usually one `Road` ends and another begins because the number of lanes has changed or some
    /// other attribute of the roadway has changed. More than two `Road`s could be involved,
    /// e.g. when a single carriageway (a bidirectional `Road`) splits into a dual carriageway
    /// (two oneway `Road`s).
    Connection,

    /// One flow of traffic forks into multiple, or multiple merge into one, but all traffic is
    /// expected to keep flowing.
    ///
    /// E.g. highway on-ramps and off-ramps.
    Fork,

    /// At least three `Road`s meet at an actual "intersection" where at least one flow of traffic
    /// gives way to, or conflicts with, another.
    Intersection,
}

/// The kind of traffic control present at an intersection.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum IntersectionControl {
    Uncontrolled,
    Signed,
    Signalled,
    Construction,
}

/// The path that some group of adjacent lanes of traffic can take through an intersection.
pub type Movement = (RoadID, RoadID);
