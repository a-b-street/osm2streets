#[allow(dead_code, unused_imports)]

pub type Meters = f32;

#[derive(Clone, Debug, PartialEq)]
pub enum End {
    Front,
    Back,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Direction {
    Forward,
    Backward,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Directions {
    Forward,
    Backward,
    BothWays,
    Alternating,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Side {
    Left,
    Right,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SideDirection {
    Leftward,
    Rightward,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LaneSide {
    Inside,
    Outside,
}

pub enum LaneDirection {
    Inward,
    Outward,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DrivingSide {
    LHT,
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
