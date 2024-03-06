use abstutil::Tags;
use enumset::EnumSet;
use geom::Distance;
use muv_osm::{
    lanes::{lanes, travel::TravelLane, Lane, LaneVariant},
    units::{self, Unit},
    AccessLevel, Conditional, Lifecycle, TMode, TModes, Tag,
};

use crate::{osm, Direction, DrivingSide, LaneSpec, LaneType, MapConfig, TurnDirection};

/// Purely from OSM tags, determine the lanes that a road segment has.
pub fn get_lane_specs_ltr(tags: &Tags, cfg: &MapConfig) -> Vec<LaneSpec> {
    let mut tags = tags;

    // This'll do weird things for the special cases of railways and cycleways/footways, but the
    // added tags will be ignored, so it doesn't matter too much.
    let mut cloned_tags;
    if cfg.inferred_sidewalks {
        // TODO This hides a potentially expensive (on a hot-path) clone
        cloned_tags = tags.clone();
        infer_sidewalk_tags(&mut cloned_tags, cfg);
        tags = &cloned_tags;
    }

    // As in tests only the driving side is given, we're choosing a country here that drives on
    // the chosen side. This messes up default speed limits and other legal defaults. This
    // is not checked in the tests however and therefore unimportant.
    let country = match (cfg.country_code.as_str(), cfg.driving_side) {
        ("", DrivingSide::Left) => "GB",
        ("", DrivingSide::Right) => "US",
        (country, _) => country,
    };

    let tags: Tag = tags.inner().iter().collect();
    let lanes = lanes(&tags, &[&country]).unwrap();

    let mut specs: Vec<_> = lanes
        .lanes
        .into_iter()
        .enumerate()
        .map(|(i, lane)| {
            from_lane(
                lane,
                traffic_direction(i as u8 * 2, lanes.centre_line, cfg.driving_side),
            )
        })
        .collect();
    if lanes.lifecycle == Lifecycle::Construction {
        for lane in &mut specs {
            lane.lt = LaneType::Construction;
        }
    }
    specs
}

fn traffic_direction(position: u8, centre_line: u8, driving_side: DrivingSide) -> Direction {
    if position + 1 == centre_line {
        return Direction::Fwd;
    }

    if (position < centre_line) == (driving_side == DrivingSide::Left) {
        Direction::Fwd
    } else {
        Direction::Back
    }
}

fn from_lane(lane: Lane, traffic_direction: Direction) -> LaneSpec {
    let (lt, dir, turns) = match &lane.variant {
        LaneVariant::Travel(t) => travel_lane(t, traffic_direction),
        LaneVariant::Parking(_) => parking_lane(traffic_direction),
    };

    let width = lane
        .width
        .map_or_else(|| LaneSpec::typical_lane_width(lt), distance_from_muv);

    LaneSpec {
        lt,
        dir,
        width,
        allowed_turns: turns,
        lane: Some(lane),
    }
}

struct Rank {
    main: TMode,
    secondary: Option<TMode>,
    designated: bool,
    lane_type: LaneType,
}

impl Rank {
    fn normal(main: TMode, lane_type: LaneType) -> Self {
        Self {
            main,
            secondary: None,
            designated: false,
            lane_type,
        }
    }

    fn designated(main: TMode, lane_type: LaneType) -> Self {
        Self {
            main,
            secondary: None,
            designated: true,
            lane_type,
        }
    }

    fn is_allowed(&self, on: &TModes<Conditional<AccessLevel>>) -> bool {
        let Some(access_main) = on.get(self.main).and_then(|c| c.base()) else {
            return false;
        };

        if self.designated {
            if access_main != &AccessLevel::Designated {
                return false;
            }
        } else if !access_level_allowed(*access_main) {
            return false;
        }

        if let Some(secondary) = self.secondary {
            let Some(access_secondary) = on.get(secondary).and_then(|a| a.base()) else {
                return false;
            };
            if access_secondary == &AccessLevel::Designated
                && access_main != &AccessLevel::Designated
            {
                return false;
            }
            return access_level_allowed(*access_secondary);
        }

        true
    }
}

fn access_level_allowed(access: AccessLevel) -> bool {
    matches!(
        access,
        AccessLevel::Designated
            | AccessLevel::Yes
            | AccessLevel::Permissive
            | AccessLevel::Discouraged
            | AccessLevel::Destination
            | AccessLevel::Customers
            | AccessLevel::Permit
            | AccessLevel::Private
    )
}

fn travel_lane(
    t: &TravelLane,
    traffic_direction: Direction,
) -> (LaneType, Direction, EnumSet<TurnDirection>) {
    let turn_forward = t.forward.turn.get(TMode::All);
    let turn_backward = t.backward.turn.get(TMode::All);
    if let Some((turn_forward, turn_backward)) = turn_forward.zip(turn_backward) {
        let forward_base = turn_forward.base();
        if forward_base.is_some() && forward_base == turn_backward.base() {
            return (LaneType::SharedLeftTurn, Direction::Fwd, EnumSet::new());
        }
    }

    for rank in [
        Rank::normal(TMode::Tram, LaneType::LightRail),
        Rank::normal(TMode::Train, LaneType::LightRail),
        Rank::designated(TMode::Bus, LaneType::Bus),
        Rank::normal(TMode::Motorcar, LaneType::Driving),
        Rank::normal(TMode::Bus, LaneType::Bus),
        Rank {
            main: TMode::Bicycle,
            secondary: Some(TMode::Foot),
            designated: false,
            lane_type: LaneType::SharedUse,
        },
        Rank::designated(TMode::Bicycle, LaneType::Biking),
        Rank::designated(TMode::Foot, LaneType::Sidewalk),
        Rank::normal(TMode::Bicycle, LaneType::Biking),
        Rank::normal(TMode::Foot, LaneType::Sidewalk),
        Rank::normal(TMode::All, LaneType::Shoulder),
    ] {
        let access_forward = rank.is_allowed(&t.forward.access);
        let access_backward = rank.is_allowed(&t.backward.access);

        let dir = match (access_forward, access_backward) {
            (true, false) => Direction::Fwd,
            (false, true) => Direction::Back,
            (true, true) => traffic_direction, // TODO: Both directions
            (false, false) => continue,
        };

        let turns = if access_forward {
            &t.forward
        } else {
            &t.backward
        }
        .turn
        .get(rank.main)
        .and_then(|v| v.base())
        .map(TurnDirection::from_muv)
        .unwrap_or_default();

        return (rank.lane_type, dir, turns);
    }
    (LaneType::Construction, Direction::Fwd, EnumSet::new())
}

fn parking_lane(traffic_direction: Direction) -> (LaneType, Direction, EnumSet<TurnDirection>) {
    (LaneType::Parking, traffic_direction, EnumSet::new())
}

fn distance_from_muv(u: Unit<units::Distance>) -> Distance {
    Distance::meters(u.to(units::Distance::Metre).value.into())
}

// If sidewalks aren't explicitly tagged on a road, fill them in. This only happens when the map is
// configured to have this inference.
pub(crate) fn infer_sidewalk_tags(tags: &mut Tags, cfg: &MapConfig) {
    // Already explicitly mapped
    if tags.contains_key("sidewalk") {
        return;
    }
    // A non-motorized road
    if tags.is_any(
        osm::HIGHWAY,
        vec!["footway", "path", "pedestrian", "steps", "track"],
    ) {
        return;
    }

    if tags.contains_key("sidewalk:left") || tags.contains_key("sidewalk:right") {
        // Attempt to mangle
        // https://wiki.openstreetmap.org/wiki/Key:sidewalk#Separately_mapped_sidewalks_on_only_one_side
        // into left/right/both. We have to make assumptions for missing values.
        let right = !tags.is("sidewalk:right", "no");
        let left = !tags.is("sidewalk:left", "no");
        let value = match (right, left) {
            (true, true) => "both",
            (true, false) => "right",
            (false, true) => "left",
            (false, false) => "none",
        };
        tags.insert("sidewalk", value);
    } else if tags.is_any(osm::HIGHWAY, vec!["motorway", "motorway_link"])
        || tags.is_any("junction", vec!["intersection", "roundabout"])
        || tags.is("foot", "no")
        || tags.is(osm::HIGHWAY, "service")
        || tags.is_any(osm::HIGHWAY, vec!["cycleway", "pedestrian", "track"])
    {
        tags.insert("sidewalk", "none");
    } else if tags.is("oneway", "yes") {
        if cfg.driving_side == DrivingSide::Right {
            tags.insert("sidewalk", "right");
        } else {
            tags.insert("sidewalk", "left");
        }
        if tags.is_any(osm::HIGHWAY, vec!["residential", "living_street"])
            && !tags.is("dual_carriageway", "yes")
        {
            tags.insert("sidewalk", "both");
        }
    } else {
        tags.insert("sidewalk", "both");
    }
}
