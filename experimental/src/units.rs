#![allow(dead_code, unused)]
pub mod preamble;

use enum_map::Enum;

pub type Meters = f64;

/// Which end of the thing?
#[derive(Copy, Clone, Debug, Enum, PartialEq)]
pub enum End {
    Front,
    Backward,
}

/// Binary travel direction, frontward & backward.
#[derive(Copy, Clone, Debug, Enum, PartialEq)]
pub enum Direction {
    Forward,
    Backwardward,
}

/// The directions that traffic flows.
#[derive(Copy, Clone, Debug, Enum, PartialEq)]
pub enum TrafficDirections {
    /// All traffic travels forward.
    Forward,
    /// All traffic travels backward.
    Backwardward,
    /// Traffic negotiates use of the road space.
    BothWays,
    /// Traffic takes turns, negotiated, or with the aid of a control like traffic lights.
    Alternating,
}

/// Which side from your perspective? Expressed as handedness, like one might learn in school.
#[derive(Copy, Clone, Debug, Enum, PartialEq)]
pub enum Side {
    /// The hand that makes an "L", "port".
    Left,
    /// The more common dominant hand, "starboard".
    Right,
}

/// Towards which hand side, left or right?
#[derive(Copy, Clone, Debug, Enum, PartialEq)]
pub enum SideDirection {
    Leftward,
    Rightward,
}

/// Which side of the roadway, in terms of lane etiquette and driving side.
/// ```use crate::units::{RoadSide::*,DrivingSide::*};
/// assert_eq!(LHT.get_side(Inside), Right);
#[derive(Copy, Clone, Debug, Enum, PartialEq)]
pub enum RoadSide {
    /// The faster side of the roadway, where you'd find oncoming cars, the "off side", where overtaking happens.
    Inside,
    /// The slower side of the roadway, where you'd find the edge of the road, the "near side", where undertaking happens.
    Outside,
}

pub enum RoadSideDirection {
    Inward,
    Outward,
}

/// Which side of the road traffic drives on.
#[derive(Copy, Clone, Debug, Enum, PartialEq)]
pub enum DrivingSide {
    /// Left hand traffic.
    LHT,
    /// Right hand traffic.
    RHT,
}

impl DrivingSide {
    pub fn opposite(&self) -> Self {
        match self {
            Self::LHT => Self::RHT,
            Self::RHT => Self::LHT,
        }
    }

    pub fn get_direction(&self, side: Side) -> Direction {
        match (self, side) {
            (Self::LHT, Side::Left) | (Self::RHT, Side::Right) => Direction::Forward,
            (Self::LHT, Side::Right) | (Self::RHT, Side::Left) => Direction::Backwardward,
        }
    }
    pub fn get_side(&self, dir: Direction) -> Side {
        match (self, dir) {
            (Self::LHT, Direction::Forward) | (Self::RHT, Direction::Backwardward) => Side::Left,
            (Self::LHT, Direction::Backwardward) | (Self::RHT, Direction::Forward) => Side::Right,
        }
    }
}
