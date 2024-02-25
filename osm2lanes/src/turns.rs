use anyhow::Result;
use enumset::EnumSet;
use geom::Angle;
use muv_osm::lanes::travel::Turn;

use super::{TurnDirection, TurnDirection::*};

impl TurnDirection {
    pub fn turn_angle(&self) -> Angle {
        match self {
            Through => Angle::degrees(0.),
            SlightRight => Angle::degrees(45.),
            SlightLeft => Angle::degrees(-45.),
            MergeRight => Angle::degrees(45.),
            MergeLeft => Angle::degrees(-45.),
            Right => Angle::degrees(90.),
            Left => Angle::degrees(-90.),
            SharpRight => Angle::degrees(135.),
            SharpLeft => Angle::degrees(-135.),
            Reverse => Angle::degrees(180.),
        }
    }

    pub fn is_merge(&self) -> bool {
        matches!(self, MergeLeft | MergeRight)
    }

    pub fn tag_value(&self) -> &'static str {
        match self {
            Through => "through",
            Left => "left",
            Right => "right",
            SlightLeft => "slight_left",
            SlightRight => "slight_right",
            SharpLeft => "sharp_left",
            SharpRight => "sharp_right",
            MergeLeft => "merge_left",
            MergeRight => "merge_right",
            Reverse => "reverse",
        }
    }

    /// Tries to parse a single turn direction from an osm tag value as per the `turn` scheme,
    /// defined at <https://wiki.openstreetmap.org/wiki/Key:turn>.
    pub fn parse(value: &str) -> Result<Option<Self>> {
        match value {
            "" | "none" => Ok(None),
            "through" => Ok(Some(Through)),
            "left" => Ok(Some(Left)),
            "right" => Ok(Some(Right)),
            "slight_left" => Ok(Some(SlightLeft)),
            "slight_right" => Ok(Some(SlightRight)),
            "sharp_left" => Ok(Some(SharpLeft)),
            "sharp_right" => Ok(Some(SharpRight)),
            "merge_to_left" => Ok(Some(MergeLeft)),
            "merge_to_right" => Ok(Some(MergeRight)),
            "reverse" => Ok(Some(Reverse)),
            _ => bail!("unknown turn direction: {value}"),
        }
    }

    /// Tries to parse a set of turn directions from an OSM tag value according to the `turn` scheme
    /// defined at <https://wiki.openstreetmap.org/wiki/Key:turn>.
    pub fn parse_set(value: &str) -> Result<EnumSet<Self>> {
        let mut result = EnumSet::new();
        for dir_str in value.split(';') {
            if let Some(dir) = TurnDirection::parse(dir_str)? {
                result.insert(dir);
            }
        }
        Ok(result)
    }

    pub(crate) fn from_muv(value: &Turn) -> EnumSet<Self> {
        let mut res = EnumSet::default();
        if value.contains(Turn::Through) {
            res.insert(TurnDirection::Through);
        }
        if value.contains(Turn::Left) {
            res.insert(TurnDirection::Left);
        }
        if value.contains(Turn::Right) {
            res.insert(TurnDirection::Right);
        }
        if value.contains(Turn::SlightLeft) {
            res.insert(TurnDirection::SlightLeft);
        }
        if value.contains(Turn::SlightRight) {
            res.insert(TurnDirection::SlightRight);
        }
        if value.contains(Turn::SharpLeft) {
            res.insert(TurnDirection::SharpLeft);
        }
        if value.contains(Turn::SharpRight) {
            res.insert(TurnDirection::SharpRight);
        }
        if value.contains(Turn::MergeToLeft) {
            res.insert(TurnDirection::MergeLeft);
        }
        if value.contains(Turn::MergeToRight) {
            res.insert(TurnDirection::MergeRight);
        }
        if value.contains(Turn::Reverse) {
            res.insert(TurnDirection::Reverse);
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use crate::TurnDirection;
    use crate::TurnDirection::*;
    use enumset::EnumSet;

    #[test]
    fn turn_direction_parses() {
        assert_eq!(TurnDirection::parse("").unwrap(), None);
        assert_eq!(TurnDirection::parse("none").unwrap(), None);

        for (input, expected) in [
            ("through", Through),
            ("left", Left),
            ("right", Right),
            ("slight_left", SlightLeft),
            ("slight_right", SlightRight),
            ("sharp_left", SharpLeft),
            ("sharp_right", SharpRight),
            ("merge_to_left", MergeLeft),
            ("merge_to_right", MergeRight),
            ("reverse", Reverse),
        ] {
            assert_eq!(TurnDirection::parse(input).unwrap(), Some(expected));
        }

        assert!(TurnDirection::parse("not_a_valid_turn").is_err());
    }

    #[test]
    fn turn_directions_parses() {
        assert_eq!(TurnDirection::parse_set("").unwrap(), EnumSet::empty());
        assert_eq!(
            TurnDirection::parse_set("through").unwrap(),
            EnumSet::only(Through)
        );
        assert_eq!(
            TurnDirection::parse_set("through;right").unwrap(),
            Through | Right
        );
    }
}
