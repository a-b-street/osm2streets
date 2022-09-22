use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use abstutil::{deserialize_btreemap, serialize_btreemap, Tags};
use geom::Distance;

use crate::{osm, OriginalRoad};

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
    pub merge_osm_ways: Vec<OriginalRoad>,
}

impl MapConfig {
    pub fn default_for_side(driving_side: DrivingSide) -> Self {
        Self {
            driving_side,
            bikes_can_use_bus_lanes: true,
            inferred_sidewalks: true,
            street_parking_spot_length: Distance::meters(8.0),
            turn_on_red: true,
            osm2lanes: false,
            find_dog_legs_experiment: false,
            merge_osm_ways: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum DrivingSide {
    Right,
    Left,
}

/// How a lane of travel is interrupted.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum InterruptionType {
    Uninterrupted,
    Yield,
    Stop,
    Signal,
    DeadEnd,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ConflictType {
    Uncontested,
    Diverge,
    Merge,
    Cross,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum IntersectionComplexity {
    /// The edge of the data that we have.
    ///
    /// The road may continue past here in reality and do anything, but the current dataset is clipped here.
    MapEdge,

    /// Two roads connect, one ending where the other begins, without conflict
    ///
    /// Like a road way has been sliced in two, because a lane was added.
    Connection,

    /// Multiple road ways connect, where the carriageway splits, where a road way joins with or
    /// diverges from another without conflict.
    ///
    /// For example, when a single carriageway splits into a dual carriageway,
    /// or when highway entrances and exits join and leave a highway.
    ///
    /// You would expect no lane marking to be interupted, and maybe even a buffer area painted
    /// where the lanes meet each other.
    MultiConnection,

    /// One or more "minor" roads merge into (yielding) or out of a "major" road,
    /// which retains its right of way.
    ///
    /// You would expect lane markings for the major road to continue (largely)
    /// uninterrupted through the intersection, with a yield line ending the minor road.
    Merge,

    /// An area of the road where traffic crosses and priority is shared over time,
    /// by lights, negotiation and priority, etc.
    ///
    /// You would expect normal lane markings to be missing, sometimes with some helpful
    /// markings added (lane dividers for multi-lane turns, etc.).
    Crossing,

    /// The end of the line.
    ///
    /// Turning circles, road end signs, train terminus thingos, ...
    Terminus,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ControlType {
    Uncontrolled, // Pretty sure this is a term that implies right of way rules somewhere.
    //TODO YieldSign,
    StopSign,      // Signed is a good standard of safety
    TrafficSignal, // Signalled is better.
    Border,        //TODO move to using IntersectionComplexity::MapEdge
    Construction,  // Are these treated as "closed"?
}
