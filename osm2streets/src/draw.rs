use crate::{marking, marking::Marking};

// We use geom and stay in map space. Output is done in latlon.
use geom::{Angle, Line, PolyLine, Polygon, Pt2D, Ring};

trait Draw<T> {
    fn draw(&self, geometry: &T) -> Vec<Ring>;
}

impl Marking {
    pub fn draw(&self) -> Vec<Ring> {
        match self {
            Marking::Longitudinal(g, m) => m.draw(g),
            Marking::Transverse(g, m) => m.draw(g),
            Marking::Symbol(g0, g1, m) => m.draw(&(g0, g1)),
            Marking::Area(g, m) => m.draw(g),
        }
    }
}

impl Draw<PolyLine> for marking::Longitudinal {
    fn draw(&self, geometry: &PolyLine) -> Vec<Ring> {
        todo!()
    }
}

impl Draw<Line> for marking::Transverse {
    fn draw(&self, geometry: &Line) -> Vec<Ring> {
        todo!()
    }
}

impl Draw<(&Pt2D, &Angle)> for marking::Symbol {
    fn draw(&self, geometry: &(&Pt2D, &Angle)) -> Vec<Ring> {
        todo!() // font or symbol renderer?
    }
}

impl Draw<Polygon> for marking::Area {
    fn draw(&self, geometry: &Polygon) -> Vec<Ring> {
        todo!()
    }
}
