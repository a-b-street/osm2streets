use itertools::Itertools;

use crate::{BufferType, Direction, LaneType, StreetNetwork};
use geo::MapCoordsInPlace;
use geom::{Distance, Line, Pt2D};

use crate::lanes::TrafficMode;
use crate::marking::{LaneEdgeKind, Marking, TurnDirections};
use crate::paint::PaintArea;
use LaneType::*;
use SurfaceMaterial::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Surface {
    pub area: geo::Polygon,
    pub material: SurfaceMaterial,
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

    /// Generate markings, described semantically.
    pub fn get_markings(&self) -> Vec<Marking> {
        let mut markings = Vec::new();

        for road in self.roads.values() {
            // Always oriented in the direction of the road
            let mut lane_centers = road.get_lane_center_lines();
            let guess_overtaking = match road.highway_type.as_str() {
                "motorway" | "trunk" | "primary" => false,
                _ => true,
            };

            // Add the left road edge.
            if let Some(first_lane) = road.lane_specs_ltr.first() {
                if matches!(
                    first_lane.lt.to_traffic_mode(),
                    Some(TrafficMode::Bike) | Some(TrafficMode::Motor)
                ) {
                    if let Ok(edge_line) = lane_centers[0].shift_left(first_lane.width / 2.0) {
                        markings.push(Marking::longitudinal(
                            edge_line,
                            LaneEdgeKind::edge(),
                            [LaneType::Buffer(BufferType::Verge), first_lane.lt],
                        ));
                    }
                }
            }
            // Add longitudinal markings between lanes.
            for (idx, pair) in road.lane_specs_ltr.windows(2).enumerate() {
                if let Ok(separation) = lane_centers[idx].shift_right(pair[0].width / 2.0) {
                    let kind = match (pair[0].lt.to_traffic_mode(), pair[1].lt.to_traffic_mode()) {
                        (Some(TrafficMode::Motor), Some(TrafficMode::Motor)) => {
                            if pair[0].dir != pair[1].dir {
                                LaneEdgeKind::oncoming(guess_overtaking, guess_overtaking)
                            } else {
                                LaneEdgeKind::separation(true, true)
                            }
                        }
                        (Some(TrafficMode::Motor), Some(TrafficMode::Bike))
                        | (Some(TrafficMode::Bike), Some(TrafficMode::Motor)) => {
                            LaneEdgeKind::separation(false, false)
                        }
                        (Some(TrafficMode::Motor), _) | (_, Some(TrafficMode::Motor)) => {
                            LaneEdgeKind::edge()
                        }
                        (Some(TrafficMode::Bike), Some(TrafficMode::Bike)) => {
                            if pair[0].dir != pair[1].dir {
                                LaneEdgeKind::oncoming(guess_overtaking, guess_overtaking)
                            } else {
                                LaneEdgeKind::separation(true, true)
                            }
                        }
                        (Some(TrafficMode::Bike), _) | (_, Some(TrafficMode::Bike)) => {
                            LaneEdgeKind::edge()
                        }
                        _ => {
                            continue;
                        }
                    };
                    markings.push(Marking::longitudinal(
                        separation,
                        kind,
                        [pair[0].lt, pair[1].lt],
                    ));
                }
            }
            // Add the right road edge.
            if let Some(last_lane) = road.lane_specs_ltr.last() {
                if matches!(
                    last_lane.lt.to_traffic_mode(),
                    Some(TrafficMode::Bike) | Some(TrafficMode::Motor)
                ) {
                    if let Ok(edge_line) = lane_centers
                        .last()
                        .expect("lane_centers to have the same length as lane_specs_ltr")
                        .shift_right(last_lane.width / 2.0)
                    {
                        markings.push(Marking::longitudinal(
                            edge_line,
                            LaneEdgeKind::edge(),
                            [last_lane.lt, LaneType::Buffer(BufferType::Verge)],
                        ));
                    }
                }
            }

            // The renderings that follow need lane centers to point in the direction of the lane.
            for (lane, center) in road.lane_specs_ltr.iter().zip(lane_centers.iter_mut()) {
                if lane.dir == Direction::Back {
                    *center = center.reversed();
                }
            }

            // Draw arrows along oneway roads.
            for (lane, center) in road.lane_specs_ltr.iter().zip(lane_centers.iter()) {
                if !lane.lt.is_for_moving_vehicles() {
                    continue;
                }

                // Add arrows along the lane, starting at the end.
                let step_size = Distance::meters(20.0);
                let buffer_ends = Distance::meters(5.0);
                for (pt, rev_angle) in center.reversed().step_along(step_size, buffer_ends) {
                    markings.push(Marking::turn_arrow(
                        pt,
                        rev_angle.opposite(),
                        // TODO use lane.turn_restrictions
                        TurnDirections::through(),
                    ))
                }
            }

            // Add markings for painted buffers.
            for (lane, center) in road.lane_specs_ltr.iter().zip(lane_centers.iter()) {
                if let LaneType::Buffer(buffer) = lane.lt {
                    if matches!(
                        buffer,
                        BufferType::FlexPosts | BufferType::JerseyBarrier | BufferType::Stripes
                    ) {
                        markings.push(Marking::area(center.make_polygons(lane.width)))
                    }
                }
            }
        }

        // TODO transverse markings
        // TODO intersection markings

        markings
    }

    pub fn get_paint_areas(&self) -> Vec<PaintArea> {
        let markings = self.get_markings();
        let mut areas: Vec<_> = markings.iter().flat_map(Marking::paint).collect();

        // Translate from map coords back to lonlat before returning.
        for paint in areas.iter_mut() {
            paint.area.map_coords_in_place(|c| {
                let gps = Pt2D::new(c.x, c.y).to_gps(&self.gps_bounds);
                (gps.x(), gps.y()).into()
            })
        }

        areas
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
