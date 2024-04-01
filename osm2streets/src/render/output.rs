use itertools::Itertools;

use geo::MapCoordsInPlace;
use geom::{Distance, Line, Pt2D};

use osm2lanes::{RoadPosition, TrafficClass};

use crate::render::marking::{LongitudinalLine, RoadMarking, Transverse};
use crate::render::paint::PaintArea;
use crate::{BufferType, Direction, LaneType, Placement, StreetNetwork, TrafficInterruption};

#[derive(Clone, Debug, PartialEq)]
pub struct Surface {
    pub area: geo::Polygon,
    pub material: SurfaceMaterial,
}

impl StreetNetwork {
    /// Generates polygons covering the road, cycle and footpath areas.
    pub fn calculate_surfaces(&self) -> Vec<Surface> {
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

    // TODO calculate_designations -> Vec<Designation> {...} // travel areas, parking, etc.

    /// Generate markings, described semantically.
    pub fn calculate_markings(&self) -> Vec<RoadMarking> {
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
                    first_lane.lt.traffic_class(),
                    Some(TrafficClass::Bicycle) | Some(TrafficClass::Motor)
                ) {
                    if let Ok(edge_line) = lane_centers[0].shift_left(first_lane.width / 2.0) {
                        markings.push(RoadMarking::longitudinal(
                            edge_line,
                            LongitudinalLine::edge(),
                            [LaneType::Buffer(BufferType::Verge), first_lane.lt],
                        ));
                    }
                }
            }
            // Add longitudinal markings between lanes.
            for (idx, pair) in road.lane_specs_ltr.windows(2).enumerate() {
                if let Ok(separation) = lane_centers[idx].shift_right(pair[0].width / 2.0) {
                    let kind = match (pair[0].lt.traffic_class(), pair[1].lt.traffic_class()) {
                        (Some(TrafficClass::Motor), Some(TrafficClass::Motor)) => {
                            if pair[0].dir != pair[1].dir {
                                LongitudinalLine::dividing(guess_overtaking, guess_overtaking)
                            } else {
                                LongitudinalLine::lane(true, true)
                            }
                        }
                        (Some(TrafficClass::Motor), Some(TrafficClass::Bicycle))
                        | (Some(TrafficClass::Bicycle), Some(TrafficClass::Motor)) => {
                            // AU specifies the use of an "edge line" in this case...
                            LongitudinalLine::lane(false, false)
                        }
                        (Some(TrafficClass::Motor), _) | (_, Some(TrafficClass::Motor)) => {
                            LongitudinalLine::edge()
                        }
                        (Some(TrafficClass::Bicycle), Some(TrafficClass::Bicycle)) => {
                            if pair[0].dir != pair[1].dir {
                                LongitudinalLine::dividing(guess_overtaking, guess_overtaking)
                            } else {
                                LongitudinalLine::lane(true, true)
                            }
                        }
                        (Some(TrafficClass::Bicycle), _) | (_, Some(TrafficClass::Bicycle)) => {
                            LongitudinalLine::edge()
                        }
                        _ => {
                            continue;
                        }
                    };
                    markings.push(RoadMarking::longitudinal(
                        separation,
                        kind,
                        [pair[0].lt, pair[1].lt],
                    ));
                }
            }
            // Add the right road edge.
            if let Some(last_lane) = road.lane_specs_ltr.last() {
                if matches!(
                    last_lane.lt.traffic_class(),
                    Some(TrafficClass::Bicycle) | Some(TrafficClass::Motor)
                ) {
                    if let Ok(edge_line) = lane_centers
                        .last()
                        .expect("lane_centers to have the same length as lane_specs_ltr")
                        .shift_right(last_lane.width / 2.0)
                    {
                        markings.push(RoadMarking::longitudinal(
                            edge_line,
                            LongitudinalLine::edge(),
                            [last_lane.lt, LaneType::Buffer(BufferType::Verge)],
                        ));
                    }
                }
            }

            // Add stop and yield lines.
            // While stop lines are measured against the reference line, we need the reference line
            // offset. Not bothering to support complicated positioning yet.
            let ref_offset = road.left_edge_offset_of(
                if let Placement::Consistent(p) = road.reference_line_placement {
                    p
                } else {
                    RoadPosition::Center
                },
                self.config.driving_side,
            );
            // Calculate the left edge offset of each lane to position the stop lines.
            let mut dist_so_far = Distance::ZERO;
            let lane_offsets = road.lane_specs_ltr.iter().map(|lane| {
                let offset = dist_so_far;
                dist_so_far += lane.width;
                offset
            });
            // Generate one line per contiguous block of traffic lanes in the same direction.
            for (dir, mut lanes_and_offsets) in road
                .lane_specs_ltr
                .iter()
                .zip(lane_offsets)
                .group_by(|(l, _)| {
                    if l.lt.is_for_moving_vehicles() {
                        Some(l.dir)
                    } else {
                        None
                    }
                })
                .into_iter()
            {
                if let Some(dir) = dir {
                    let stop_line = if dir == Direction::Forward {
                        &road.stop_line_end
                    } else {
                        &road.stop_line_start
                    };

                    let stop_kind = match stop_line.interruption {
                        TrafficInterruption::Stop | TrafficInterruption::Signal => {
                            Transverse::StopLine
                        }
                        TrafficInterruption::Yield => Transverse::YieldLine,
                        TrafficInterruption::Uninterrupted | TrafficInterruption::DeadEnd => {
                            continue;
                        }
                    };

                    // Calculate the distance from the ref_line to the left and right edge of the group.
                    let (first_lane, first_offset) =
                        lanes_and_offsets.next().expect("non-empty group");
                    let (last_lane, last_offset) = lanes_and_offsets
                        .last()
                        .unwrap_or((first_lane, first_offset));
                    let left_dist = first_offset - ref_offset;
                    let right_dist = ref_offset - (last_offset + last_lane.width);

                    // Add the vehicle line.
                    if let Some(dist) = stop_line.vehicle_distance {
                        if let Ok((pt, angle)) = road.reference_line.dist_along(dist) {
                            markings.push(RoadMarking::transverse(
                                Line::must_new(
                                    pt.project_away(left_dist, angle.rotate_degs(90.0)),
                                    pt.project_away(right_dist, angle.rotate_degs(-90.0)),
                                ),
                                stop_kind,
                            ));
                        }
                    }

                    // Add the bike line, aka "Advanced Stop Line".
                    if let Some(dist) = stop_line.bike_distance {
                        if let Ok((pt, angle)) = road.reference_line.dist_along(dist) {
                            markings.push(RoadMarking::transverse(
                                Line::must_new(
                                    pt.project_away(left_dist, angle.rotate_degs(90.0)),
                                    pt.project_away(right_dist, angle.rotate_degs(-90.0)),
                                ),
                                stop_kind, // We could add another variant for this.
                            ));
                        }
                    }
                }
            }

            // The renderings that follow need lane centers to point in the direction of the lane.
            for (lane, center) in road.lane_specs_ltr.iter().zip(lane_centers.iter_mut()) {
                if lane.dir == Direction::Backward {
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
                    markings.push(RoadMarking::turn_arrow(
                        pt,
                        rev_angle.opposite(),
                        lane.allowed_turns.clone(),
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
                        markings.push(RoadMarking::area(center.make_polygons(lane.width)))
                    }
                }
            }
        }

        // TODO intersection markings

        markings
    }

    pub fn calculate_paint_areas(&self) -> Vec<PaintArea> {
        let markings = self.calculate_markings();
        let mut areas: Vec<_> = markings.iter().flat_map(RoadMarking::paint).collect();

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
            Self::Asphalt => "asphalt",
            Self::FineAsphalt => "fine_asphalt",
            Self::Concrete => "concrete",
        }
    }
}

fn material_from_lane_type(lt: LaneType) -> Option<SurfaceMaterial> {
    use LaneType::*;
    match lt {
        Sidewalk | Footway => Some(SurfaceMaterial::Concrete),

        Driving | Parking(_) | Shoulder | SharedLeftTurn | Construction | Buffer(_) | Bus => {
            Some(SurfaceMaterial::Asphalt)
        }

        Biking | SharedUse => Some(SurfaceMaterial::FineAsphalt),

        LightRail => None,
    }
}
