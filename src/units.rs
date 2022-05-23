#[allow(dead_code, unused_imports)]
pub mod preamble;

pub type Meters = f32;

/// Which end of the thing?
#[derive(Clone, Debug, PartialEq)]
pub enum End {
    Front,
    Back,
}

/// Binary travel direction, frontward & backward.
#[derive(Clone, Debug, PartialEq)]
pub enum Direction {
    Forward,
    Backward,
}

/// The directions that traffic flows.
#[derive(Clone, Debug, PartialEq)]
pub enum TrafficDirections {
    /// All traffic travels forward.
    Forward,
    /// All traffic travels backward.
    Backward,
    /// Traffic negotiates use of the road space.
    BothWays,
    /// Traffic takes turns, negotiated, or with the aid of a control like traffic lights.
    Alternating,
}

/// Which side from your perspective? Expressed as handedness, like one might learn in school.
#[derive(Clone, Debug, PartialEq)]
pub enum Side {
    /// The hand that makes an "L", "port".
    Left,
    /// The more common dominant hand, "starboard".
    Right,
}

/// Towards which hand side, left or right?
#[derive(Clone, Debug, PartialEq)]
pub enum SideDirection {
    Leftward,
    Rightward,
}

/// Which side of the roadway, in terms of lane etiquette and driving side.
/// ```use crate::units::{RoadSide::*,DrivingSide::*};
/// assert_eq!(LHT.get_side(Inside), Right);
#[derive(Clone, Debug, PartialEq)]
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
#[derive(Clone, Debug, PartialEq)]
pub enum DrivingSide {
    /// Left hand traffic.
    LHT,
    /// Right hand traffic.
    RHT,
}

impl DrivingSide {
    fn get_direction(&self, side: Side) -> Direction {
        match (self, side) {
            (Self::LHT, Side::Left) | (Self::RHT, Side::Right) => Direction::Forward,
            (Self::LHT, Side::Right) | (Self::RHT, Side::Left) => Direction::Backward,
        }
    }
    fn get_side(&self, dir: Direction) -> Side {
        match (self, dir) {
            (Self::LHT, Direction::Forward) | (Self::RHT, Direction::Backward) => Side::Left,
            (Self::LHT, Direction::Backward) | (Self::RHT, Direction::Forward) => Side::Right,
        }
    }
}
