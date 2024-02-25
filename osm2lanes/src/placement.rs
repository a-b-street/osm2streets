use abstutil::Tags;
use anyhow::Result;

use super::{LtrLaneNum, Placement, RoadPosition};

use Placement::*;
use RoadPosition::*;

impl RoadPosition {
    /// Tries to parse a road position from an osm tag value as per the `placement` scheme.
    /// See https://wiki.openstreetmap.org/wiki/Proposed_features/placement#Tagging
    ///
    /// The direction is treated as forward, use `reverse()` on the result if the context is backward.
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "" => Ok(Center),
            "separation" => Ok(Separation),
            _ => {
                if let Some((kind, lane_str)) = value.split_once(':') {
                    if let Ok(lane) = lane_str.parse::<usize>() {
                        match kind {
                            "left_of" => Ok(LeftOf(LtrLaneNum::Forward(lane))),
                            "middle_of" => Ok(MiddleOf(LtrLaneNum::Forward(lane))),
                            "right_of" => Ok(RightOf(LtrLaneNum::Forward(lane))),
                            _ => bail!("unknown lane position specifier: {kind}"),
                        }
                    } else {
                        bail!("bad lane number: {lane_str}")
                    }
                } else {
                    bail!("unknown placement value: {value}")
                }
            }
        }
    }
}

impl Placement {
    /// Tries to parse a placement from a set of OSM tags according to the `placement` scheme.
    /// See https://wiki.openstreetmap.org/wiki/Proposed_features/placement#Tagging
    ///
    /// Limitations:
    /// - Doesn't validate tag combos, just returns the first interpretation it finds.
    /// - Doesn't allow `:end` and `:start` to mix `:forward` and `:back`. Maybe it should?
    pub fn parse(tags: &Tags) -> Result<Self> {
        if let Some(transition_or_pos) = tags.get("placement") {
            if transition_or_pos == "transition" {
                Ok(Transition)
            } else {
                Ok(Consistent(RoadPosition::parse(transition_or_pos.as_str())?))
            }
        } else if tags.has_any(vec!["placement:start", "placement:end"]) {
            Ok(Varying(
                RoadPosition::parse(tags.get("placement:start").map_or("", |s| s.as_str()))?,
                RoadPosition::parse(tags.get("placement:end").map_or("", |s| s.as_str()))?,
            ))
        } else if let Some(pos) = tags.get("placement:forward") {
            Ok(Consistent(RoadPosition::parse(pos.as_str())?))
        } else if tags.has_any(vec!["placement:forward:start", "placement:forward:end"]) {
            Ok(Varying(
                RoadPosition::parse(
                    tags.get("placement:forward:start")
                        .map_or("", |s| s.as_str()),
                )?,
                RoadPosition::parse(tags.get("placement:forward:end").map_or("", |s| s.as_str()))?,
            ))
        } else if let Some(backwards_pos) = tags.get("placement:backward") {
            Ok(Consistent(
                RoadPosition::parse(backwards_pos.as_str())?.reverse(),
            ))
        } else if tags.has_any(vec!["placement:backward:start", "placement:backward:end"]) {
            Ok(Varying(
                RoadPosition::parse(
                    tags.get("placement:backward:start")
                        .map_or("", |s| s.as_str()),
                )?
                .reverse(),
                RoadPosition::parse(
                    tags.get("placement:backward:end")
                        .map_or("", |s| s.as_str()),
                )?
                .reverse(),
            ))
        } else {
            Ok(Consistent(Center)) // The default when not tagged.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use LtrLaneNum::*;

    #[test]
    fn road_position_parses() {
        assert_eq!(RoadPosition::parse("").unwrap(), Center);
        assert_eq!(RoadPosition::parse("separation").unwrap(), Separation);
        assert_eq!(
            RoadPosition::parse("left_of:1").unwrap(),
            LeftOf(Forward(1))
        );
        assert_eq!(
            RoadPosition::parse("middle_of:1").unwrap(),
            MiddleOf(Forward(1))
        );
        assert_eq!(
            RoadPosition::parse("right_of:1").unwrap(),
            RightOf(Forward(1))
        );
    }

    #[test]
    fn placement_parses() {
        assert_eq!(
            Placement::parse(&Tags::new(BTreeMap::from([(
                "placement".into(),
                "transition".into()
            )])))
            .unwrap(),
            Transition
        );

        assert_eq!(
            Placement::parse(&Tags::new(BTreeMap::from([(
                "placement".into(),
                "right_of:1".into()
            )])))
            .unwrap(),
            Consistent(RightOf(Forward(1)))
        );

        assert_eq!(
            Placement::parse(&Tags::new(BTreeMap::from([(
                "placement:forward".into(),
                "right_of:1".into()
            )])))
            .unwrap(),
            Consistent(RightOf(Forward(1)))
        );

        assert_eq!(
            Placement::parse(&Tags::new(BTreeMap::from([(
                "placement:backward".into(),
                "right_of:1".into()
            )])))
            .unwrap(),
            Consistent(RightOf(Backward(1)))
        );

        assert_eq!(
            Placement::parse(&Tags::new(BTreeMap::from([
                ("placement:start".into(), "right_of:1".into()),
                ("placement:end".into(), "right_of:2".into())
            ])))
            .unwrap(),
            Varying(RightOf(Forward(1)), RightOf(Forward(2)))
        );

        assert_eq!(
            Placement::parse(&Tags::new(BTreeMap::from([
                ("placement:backward:start".into(), "right_of:1".into()),
                ("placement:backward:end".into(), "right_of:2".into())
            ])))
            .unwrap(),
            Varying(RightOf(Backward(1)), RightOf(Backward(2)))
        );
    }
}
