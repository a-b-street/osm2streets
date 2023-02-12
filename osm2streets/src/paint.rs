use crate::{marking, marking::RoadMarking};

// We use geom and stay in map space. Output is done in latlon.
use geom::{Angle, Distance, Line, PolyLine, Polygon, Pt2D, Ring};

use crate::lanes::TrafficClass;

#[derive(Clone, Debug, PartialEq)]
pub struct PaintArea {
    // Because I'm lazy and don't want to make different "map space" and "lonlat space" PaintArea
    // types, I'm using geo::Polygon, so we can just swap out the coords in place. Not ideal.
    /// A simple ring.
    pub area: geo::Polygon,
    pub color: PaintColor,
}
impl PaintArea {
    pub fn new(area: Ring, color: PaintColor) -> Self {
        Self {
            area: area.into_polygon().into(),
            color,
        }
    }

    pub fn white(area: Ring) -> Self {
        Self {
            area: area.into_polygon().into(),
            color: PaintColor::White,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PaintColor {
    White,
    Yellow,
}

impl PaintColor {
    pub fn to_str(&self) -> &str {
        match self {
            Self::White => "white",
            Self::Yellow => "yellow",
        }
    }
}

trait Paint<T> {
    fn paint(&self, geometry: &T) -> Vec<PaintArea>;
}

impl RoadMarking {
    pub fn paint(&self) -> Vec<PaintArea> {
        match self {
            RoadMarking::Longitudinal(g, m) => m.paint(g),
            RoadMarking::Transverse(g, m) => m.paint(g),
            RoadMarking::Symbol(g0, g1, m) => m.paint(&(*g0, *g1)),
            RoadMarking::Area(g, m) => m.paint(g),
        }
    }
}

// Measurements for the default non-locale rendering of markings.
const LINE_WIDTH: Distance = Distance::const_meters(0.10);
const LINE_WIDTH_THIN: Distance = Distance::const_meters(0.08);
const LINE_WIDTH_THICK: Distance = Distance::const_meters(0.30);
const DASH_LENGTH_SHORT: Distance = Distance::const_meters(1.0);
const DASH_GAP_SHORT: Distance = Distance::const_meters(1.0);
const DASH_LENGTH_LONG: Distance = Distance::const_meters(2.0);
const DASH_GAP_LONG: Distance = Distance::const_meters(4.5);

impl Paint<PolyLine> for marking::Longitudinal {
    fn paint(&self, separator: &PolyLine) -> Vec<PaintArea> {
        // TODO incorporate colors throughout instead of only collecting rings:
        let mut rings: Vec<Ring> = Vec::new();

        match self.kind {
            marking::LongitudinalLine::Dividing {
                overtake_left,
                overtake_right,
            } => match self.lanes.map(|x| x.traffic_class()) {
                [Some(TrafficClass::Motor), _] | [_, Some(TrafficClass::Motor)] => {
                    if let Ok(right_line) = separator.shift_right(LINE_WIDTH) {
                        if overtake_left {
                            rings.append(
                                &mut right_line
                                    .dashed_lines(LINE_WIDTH, DASH_LENGTH_LONG, DASH_GAP_LONG)
                                    .into_iter()
                                    .map(|x| x.into_outer_ring())
                                    .collect(),
                            );
                        } else {
                            rings.push(right_line.make_polygons(LINE_WIDTH).into_outer_ring());
                        }
                    }
                    if let Ok(left_line) = separator.shift_left(LINE_WIDTH) {
                        if overtake_right {
                            rings.append(
                                &mut left_line
                                    .dashed_lines(LINE_WIDTH, DASH_LENGTH_LONG, DASH_GAP_LONG)
                                    .into_iter()
                                    .map(|x| x.into_outer_ring())
                                    .collect(),
                            );
                        } else {
                            rings.push(left_line.make_polygons(LINE_WIDTH).into_outer_ring());
                        }
                    }
                }
                [Some(TrafficClass::Bicycle), _] | [_, Some(TrafficClass::Bicycle)] => {
                    if overtake_left || overtake_right {
                        rings.append(
                            &mut separator
                                .dashed_lines(LINE_WIDTH_THIN, DASH_LENGTH_LONG, DASH_GAP_LONG)
                                .into_iter()
                                .map(|x| x.into_outer_ring())
                                .collect(),
                        );
                    } else {
                        rings.push(separator.make_polygons(LINE_WIDTH_THIN).into_outer_ring())
                    }
                }
                _ => {}
            },
            marking::LongitudinalLine::Lane {
                merge_left,
                merge_right,
            } => match self.lanes.map(|x| x.traffic_class()) {
                [Some(TrafficClass::Motor), Some(TrafficClass::Motor)] => {
                    if merge_left || merge_right {
                        rings.append(
                            &mut separator
                                .dashed_lines(LINE_WIDTH, DASH_LENGTH_LONG, DASH_GAP_LONG)
                                .into_iter()
                                .map(|x| x.into_outer_ring())
                                .collect(),
                        );
                    } else {
                        rings.push(separator.make_polygons(LINE_WIDTH).into_outer_ring())
                    }
                }
                [Some(TrafficClass::Motor), _] | [_, Some(TrafficClass::Motor)] => {
                    rings.push(separator.make_polygons(LINE_WIDTH).into_outer_ring())
                }
                // TODO: Bike lanes, tram lanes.
                _ => {}
            },
            marking::LongitudinalLine::Edge => {
                rings.push(separator.make_polygons(LINE_WIDTH).into_outer_ring())
            }
            marking::LongitudinalLine::Continuity => {
                rings.append(
                    &mut separator
                        .dashed_lines(LINE_WIDTH, DASH_LENGTH_SHORT, DASH_GAP_SHORT)
                        .into_iter()
                        .map(|x| x.into_outer_ring())
                        .collect(),
                );
            }
            marking::LongitudinalLine::Turn => {
                rings.append(
                    &mut separator
                        .dashed_lines(LINE_WIDTH, DASH_LENGTH_LONG, DASH_GAP_SHORT)
                        .into_iter()
                        .map(|x| x.into_outer_ring())
                        .collect(),
                );
            }
        }

        rings.into_iter().map(PaintArea::white).collect()
    }
}

impl Paint<Line> for marking::Transverse {
    fn paint(&self, geometry: &Line) -> Vec<PaintArea> {
        match self {
            marking::Transverse::StopLine => {
                vec![PaintArea::white(
                    geometry.make_polygons(LINE_WIDTH_THICK).into_outer_ring(),
                )]
            }
            marking::Transverse::YieldLine => geometry
                .to_polyline()
                .exact_dashed_polygons(
                    LINE_WIDTH_THICK,
                    Distance::meters(0.6),
                    Distance::meters(0.6),
                )
                .into_iter()
                .map(Polygon::into_outer_ring)
                .map(PaintArea::white)
                .collect(),
        }
    }
}

impl Paint<(Pt2D, Angle)> for marking::Symbol {
    fn paint(&self, &(pt, a): &(Pt2D, Angle)) -> Vec<PaintArea> {
        match self {
            marking::Symbol::TurnArrow(directions) => {
                // TODO draw the specified direction
                let arrow_len = Distance::meters(1.75);
                let thickness = LINE_WIDTH_THICK;
                let arrow = PolyLine::must_new(vec![
                    pt.project_away(arrow_len / 2.0, a.opposite()),
                    pt.project_away(arrow_len / 2.0, a),
                ])
                .make_arrow(thickness * 2.0, geom::ArrowCap::Triangle);
                vec![PaintArea::white(arrow.into_outer_ring())]
            }
            _ => {
                todo!()
            }
        }
    }
}

impl Paint<Polygon> for marking::Area {
    fn paint(&self, geometry: &Polygon) -> Vec<PaintArea> {
        vec![PaintArea::white(geometry.get_outer_ring().clone())]
        // let mut output: Vec<Ring> = Vec::new();
        // // Ring around the outside.
        // output.push(
        //     PolyLine::must_new(geometry.get_outer_ring().points().clone())
        //         .make_polygons(LINE_WIDTH)
        //         .into_outer_ring(),
        // );
        // // Diagonal stripes along the lane.
        // let step_size = Distance::meters(3.0);
        // let buffer_ends = Distance::meters(5.0);
        // for (center, angle) in center.step_along(step_size, buffer_ends) {
        //     // Extend the stripes into the side lines
        //     let left =
        //         center.project_away(lane.width / 2.0 + thickness, angle.rotate_degs(45.0));
        //     let right = center.project_away(
        //         lane.width / 2.0 + thickness,
        //         angle.rotate_degs(45.0).opposite(),
        //     );
        //     output.push(PaintArea::new(
        //         Line::must_new(left, right).make_polygons(thickness).into(),
        //     ));
        // }
        // output
    }
}
