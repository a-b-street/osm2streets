use itertools::Itertools;

use crate::{Direction, LaneType, StreetNetwork};
use geo::MapCoordsInPlace;
use geom::{Distance, Line, PolyLine, Pt2D};

use LaneType::*;
use SurfaceMaterial::*;

pub struct Surface {
    pub area: geo::Polygon,
    pub material: SurfaceMaterial,
}

pub struct PaintArea {
    pub area: geo::Polygon,
    pub color: PaintColor,
}

impl PaintArea {
    pub fn new(area: geo::Polygon) -> Self {
        Self {
            area,
            color: PaintColor::White,
        }
    }
    pub fn with_color(area: geo::Polygon, color: PaintColor) -> Self {
        Self { area, color }
    }
}

impl StreetNetwork {
    /// Generates polygons covering the road, cycle and footpath areas.
    pub fn get_surfaces(&self) -> Vec<Surface> {
        let mut output = Vec::new();

        // Add polygons for road surfaces.
        for road in self.roads.values() {
            // Generate an area for each contiguous group of footpath, bike path and road lanes.
            let center_offset = road.total_width() / 2.0;
            let mut processed_width = Distance::ZERO;
            for (material, lanes) in road
                .lane_specs_ltr
                .iter()
                .group_by(|l| material_from_lane_type(l.lt))
                .into_iter()
            {
                if let Some(material) = material {
                    let mut width = Distance::ZERO;
                    for lane in lanes {
                        width += lane.width;
                    }
                    output.push(Surface {
                        area: road
                            .center_line
                            .shift_either_direction(processed_width - center_offset + width / 2.0)
                            .unwrap_or_else(|_| road.center_line.clone())
                            .make_polygons(width)
                            .into(),
                        material,
                    });
                    processed_width += width;
                } else {
                    for lane in lanes {
                        processed_width += lane.width;
                    }
                }
            }
        }

        // Polygon per intersection
        for intersection in self.intersections.values() {
            // TODO dissect the area into pieces based on movements, like with Roads.
            output.push(Surface {
                area: intersection.polygon.clone().into(),
                material: SurfaceMaterial::Asphalt,
            });
        }

        // Translate from map coords back to latlon before returning.
        for surface in output.iter_mut() {
            surface.area.map_coords_in_place(|c| {
                let gps = Pt2D::new(c.x, c.y).to_gps(&self.gps_bounds);
                (gps.x(), gps.y()).into()
            })
        }
        output
    }

    // TODO get_designations -> Vec<Designation> {...} // travel areas, parking, etc.

    // TODO pub fn get_markings -> Vec<Marking> {...}

    /// Generate painted areas.
    pub fn get_paint_areas(&self) -> Vec<PaintArea> {
        // TODO generate semantic markings first in get_markings, then render them into areas here.

        let mut output = Vec::new();

        for road in self.roads.values() {
            // Always oriented in the direction of the road
            let mut lane_centers = road.get_lane_center_lines();

            for (idx, pair) in road.lane_specs_ltr.windows(2).enumerate() {
                // Generate a "center line" between lanes of different directions
                if pair[0].dir != pair[1].dir {
                    if let Ok(separator) = lane_centers[idx].shift_right(pair[0].width / 2.0) {
                        if let Ok(right_line) = separator.shift_right(Distance::centimeters(16)) {
                            output.push(PaintArea::with_color(
                                right_line.make_polygons(Distance::centimeters(16)).into(),
                                PaintColor::Yellow,
                            ));
                        }
                        if let Ok(left_line) = separator.shift_left(Distance::centimeters(16)) {
                            output.push(PaintArea::with_color(
                                left_line.make_polygons(Distance::centimeters(16)).into(),
                                PaintColor::Yellow,
                            ));
                        }
                    }
                    continue;
                }

                // Generate a "lane separator" between driving lanes only.
                if pair[0].lt == LaneType::Driving && pair[1].lt == LaneType::Driving {
                    if let Ok(between) = lane_centers[idx].shift_right(pair[0].width / 2.0) {
                        for poly in between.dashed_lines(
                            Distance::meters(0.16),
                            Distance::meters(1.0),
                            Distance::meters(1.5),
                        ) {
                            output.push(PaintArea::new(poly.into()));
                        }
                    }
                }

                // TODO other cases
            }

            // The renderings that follow need lane centers to point in the direction of the lane.
            for (lane, center) in road.lane_specs_ltr.iter().zip(lane_centers.iter_mut()) {
                if lane.dir == Direction::Back {
                    *center = center.reversed();
                }
            }

            // Draw arrows along oneway roads.
            if road.oneway_for_driving().is_some() {
                for (lane, center) in road.lane_specs_ltr.iter().zip(lane_centers.iter()) {
                    if !lane.lt.is_for_moving_vehicles() {
                        continue;
                    }

                    let step_size = Distance::meters(20.0);
                    let buffer_ends = Distance::meters(5.0);
                    let arrow_len = Distance::meters(1.75);
                    let thickness = Distance::meters(0.16);
                    for (pt, angle) in center.step_along(step_size, buffer_ends) {
                        let arrow = PolyLine::must_new(vec![
                            pt.project_away(arrow_len / 2.0, angle.opposite()),
                            pt.project_away(arrow_len / 2.0, angle),
                        ])
                        .make_arrow(thickness * 2.0, geom::ArrowCap::Triangle);
                        output.push(PaintArea::new(arrow.into()));
                    }
                }
            }

            // Add stripes to show buffers. Ignore the type of the buffer for now -- we need to
            // decide all the types and how to render them.
            for (lane, center) in road.lane_specs_ltr.iter().zip(lane_centers.iter()) {
                if !matches!(lane.lt, LaneType::Buffer(_)) {
                    continue;
                }

                // Mark the sides of the lane clearly
                let thickness = Distance::meters(0.16);
                output.push(PaintArea::new(
                    center
                        .must_shift_right((lane.width - thickness) / 2.0)
                        .make_polygons(thickness)
                        .into(),
                ));
                output.push(PaintArea::new(
                    center
                        .must_shift_left((lane.width - thickness) / 2.0)
                        .make_polygons(thickness)
                        .into(),
                ));

                // Diagonal stripes along the lane
                let step_size = Distance::meters(3.0);
                let buffer_ends = Distance::meters(5.0);
                for (center, angle) in center.step_along(step_size, buffer_ends) {
                    // Extend the stripes into the side lines
                    let left =
                        center.project_away(lane.width / 2.0 + thickness, angle.rotate_degs(45.0));
                    let right = center.project_away(
                        lane.width / 2.0 + thickness,
                        angle.rotate_degs(45.0).opposite(),
                    );
                    output.push(PaintArea::new(
                        Line::must_new(left, right).make_polygons(thickness).into(),
                    ));
                }
            }
        }

        // Translate from map coords back to latlon before returning.
        for paint in output.iter_mut() {
            paint.area.map_coords_in_place(|c| {
                let gps = Pt2D::new(c.x, c.y).to_gps(&self.gps_bounds);
                (gps.x(), gps.y()).into()
            })
        }
        output
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SurfaceMaterial {
    Asphalt,
    FineAsphalt,
    Concrete,
}

impl SurfaceMaterial {
    pub fn to_str(&self) -> &str {
        match self {
            Asphalt => "asphalt",
            FineAsphalt => "fine_asphalt",
            Concrete => "concrete",
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

fn material_from_lane_type(lt: LaneType) -> Option<SurfaceMaterial> {
    match lt {
        Sidewalk | Footway => Some(Concrete),

        Driving | Parking | Shoulder | SharedLeftTurn | Construction | Buffer(_) | Bus => {
            Some(Asphalt)
        }

        Biking | SharedUse => Some(FineAsphalt),

        LightRail => None,
    }
}
