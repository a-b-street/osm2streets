use anyhow::Result;
use serde::{Deserialize, Serialize};

use abstutil::Tags;
use geom::{Angle, Distance, PolyLine};

use crate::lanes::{Placement, RoadPosition};
use crate::{
    get_lane_specs_ltr, osm, CommonEndpoint, Direction, DrivingSide, InputRoad, IntersectionID,
    LaneSpec, LaneType, MapConfig, OriginalRoad, RestrictionType, RoadID, RoadWithEndpoints,
    StreetNetwork,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Road {
    pub id: RoadID,
    /// The original segments of OSM ways making up this road. One road may consist of multiple
    /// segments (when an intersection is collapsed).
    pub osm_ids: Vec<OriginalRoad>,

    pub src_i: IntersectionID,
    pub dst_i: IntersectionID,

    /// The OSM `highway` tag indicating the type of this road. See
    /// <https://wiki.openstreetmap.org/wiki/Key:highway>.
    ///
    /// Note for railways, this is actually the `railway` tag instead.
    pub highway_type: String,
    /// The name of the road in the default OSM-specified language
    pub name: Option<String>,
    /// This road exists only for graph connectivity. It's physically part of a complex
    /// intersection. A transformation will likely collapse it.
    pub internal_junction_road: bool,
    /// The vertical layer of the road, with 0 the default and negative values lower down. See
    /// <https://wiki.openstreetmap.org/wiki/Key:layer>.
    pub layer: isize,

    /// The original OSM geometry (slightly smoothed). This will extend beyond the extent of the
    /// resulting trimmed road, be positioned somewhere within the road according to the placement
    /// tag and might be nonsense for the first/last segment.
    pub reference_line: PolyLine,
    pub reference_line_placement: Placement,
    /// The physical center of all the lanes, including sidewalks (at RoadPosition::FullWidthCenter).
    /// This will differ from reference_line and modified by transformations, notably it will be
    /// offset based on reference_line_placement and trimmed by
    /// `Transformation::GenerateIntersectionGeometry`.
    pub center_line: PolyLine,
    pub turn_restrictions: Vec<(RestrictionType, RoadID)>,
    /// (via, to). For turn restrictions where 'via' is an entire road. Only BanTurns.
    pub complicated_turn_restrictions: Vec<(RoadID, RoadID)>,

    pub lane_specs_ltr: Vec<LaneSpec>,
}

impl Road {
    pub fn new(
        id: RoadID,
        osm_ids: Vec<OriginalRoad>,
        src_i: IntersectionID,
        dst_i: IntersectionID,
        reference_line: PolyLine,
        osm_tags: Tags,
        config: &MapConfig,
    ) -> Self {
        let lane_specs_ltr = get_lane_specs_ltr(&osm_tags, config);

        let layer = if let Some(layer) = osm_tags.get("layer") {
            match layer.parse::<f64>() {
                // Just drop .5 for now
                Ok(l) => l as isize,
                Err(_) => {
                    warn!("Weird layer={layer}");
                    0
                }
            }
        } else {
            0
        };

        // Ignoring errors for now.
        let placement = Placement::parse(&osm_tags).unwrap_or_else(|e| {
            warn!("bad placement value (using default): {e}");
            Placement::Consistent(RoadPosition::Center)
        });

        let mut result = Self {
            id,
            osm_ids,
            src_i,
            dst_i,
            highway_type: osm_tags
                .get(osm::HIGHWAY)
                .or_else(|| osm_tags.get("railway"))
                .cloned()
                .expect("Can't create a Road without the highway or railway tag"),
            name: osm_tags.get("name").cloned(),
            internal_junction_road: osm_tags.is("junction", "intersection"),
            layer,
            reference_line,
            reference_line_placement: placement,
            center_line: PolyLine::dummy(),
            turn_restrictions: Vec::new(),
            complicated_turn_restrictions: Vec::new(),

            lane_specs_ltr,
        };

        result.update_center_line(config.driving_side); // TODO delay this until trim_start and trim_end are calculated
        result
    }

    /// Calculates and sets the center_line from reference_line, reference_line_placement
    /// (and TODO trim_start, trim_end).
    pub fn update_center_line(&mut self, driving_side: DrivingSide) {
        let ref_position = match self.reference_line_placement {
            Placement::Consistent(p) => p,
            Placement::Varying(p, _) => {
                warn!("varying placement not yet supported, using placement:start");
                p
            }
            Placement::Transition => {
                // We haven't calculated the transition yet. At early stages of understanding the
                // OSM data, we pretend these `Road`s have default placement.
                RoadPosition::Center
            }
        };
        let ref_offset = self.left_edge_offset_of(ref_position, driving_side);
        let target_offset = self.left_edge_offset_of(RoadPosition::FullWidthCenter, driving_side);

        self.center_line = self
            .reference_line
            .shift_either_direction(target_offset - ref_offset)
            .unwrap_or_else(|_| {
                warn!("resulting center_line is degenerate!");
                self.reference_line.clone()
            });
    }

    pub fn is_light_rail(&self) -> bool {
        self.lane_specs_ltr
            .iter()
            .all(|spec| spec.lt == LaneType::LightRail)
    }

    pub fn is_service(&self) -> bool {
        self.highway_type == "service"
    }

    pub fn is_cycleway(&self) -> bool {
        let mut bike = false;
        for spec in &self.lane_specs_ltr {
            if spec.lt == LaneType::Biking {
                bike = true;
            } else if spec.lt != LaneType::Shoulder {
                return false;
            }
        }
        bike
    }

    pub fn is_driveable(&self) -> bool {
        self.lane_specs_ltr
            .iter()
            .any(|spec| spec.lt == LaneType::Driving)
    }

    pub fn oneway_for_driving(&self) -> Option<Direction> {
        LaneSpec::oneway_for_driving(&self.lane_specs_ltr)
    }

    pub fn can_drive_out_of_end(&self, which_end: IntersectionID) -> bool {
        if let Some(driving_dir) = self.oneway_for_driving() {
            let required_dir = if self.dst_i == which_end {
                Direction::Fwd
            } else {
                Direction::Back
            };
            return driving_dir == required_dir;
        }
        return true;
    }

    pub(crate) fn can_drive_into_end(&self, which_end: IntersectionID) -> bool {
        if let Some(driving_dir) = self.oneway_for_driving() {
            let required_dir = if self.src_i == which_end {
                Direction::Fwd
            } else {
                Direction::Back
            };
            return driving_dir == required_dir;
        }
        return true;
    }

    pub fn allowed_to_turn_to(&self, dest: RoadID) -> bool {
        let mut has_exclusive_allows = false;
        for (t, other) in self.turn_restrictions.iter() {
            match t {
                RestrictionType::BanTurns => {
                    if *other == dest {
                        return false;
                    }
                }
                RestrictionType::OnlyAllowTurns => {
                    if *other == dest {
                        return true;
                    }
                    has_exclusive_allows = true;
                }
            }
        }
        !has_exclusive_allows
    }

    /// Points from first to last point. Undefined for loops.
    pub fn angle(&self) -> Angle {
        self.reference_line
            .first_pt()
            .angle_to(self.reference_line.last_pt())
    }

    /// The length of the original OSM center line, before any trimming away from intersections
    pub fn untrimmed_length(&self) -> Distance {
        self.reference_line.length()
    }

    /// Returns a line along `RoadPosition::Center` (but untrimmed) and total width for ALL lanes.
    pub fn untrimmed_road_geometry(&self, driving_side: DrivingSide) -> (PolyLine, Distance) {
        let total_width = self.total_width();
        let ref_position = match self.reference_line_placement {
            Placement::Consistent(p) => p,
            Placement::Varying(p, _) => p,
            Placement::Transition => RoadPosition::Center, // Best we can do for now.
        };
        let ref_offset = self.left_edge_offset_of(ref_position, driving_side);
        let center_offset = self.left_edge_offset_of(RoadPosition::Center, driving_side);

        (
            self.reference_line
                .shift_either_direction(center_offset - ref_offset)
                .unwrap(),
            total_width,
        )
    }

    pub fn total_width(&self) -> Distance {
        self.lane_specs_ltr.iter().map(|l| l.width).sum()
    }

    /// Calculates the number of (forward, both_ways, backward) lanes. The order of the lanes
    /// doesn't matter.
    pub fn _travel_lane_counts(&self) -> (usize, usize, usize) {
        let mut result = (0, 0, 0);
        for lane in &self.lane_specs_ltr {
            if !lane.lt.is_tagged_by_lanes_suffix() {
                continue;
            }
            if lane.lt == LaneType::SharedLeftTurn {
                result.1 += 1;
            } else if lane.dir == Direction::Fwd {
                result.0 += 1;
            } else {
                result.2 += 1;
            }
        }
        result
    }

    /// Calculates the distance from the left edge to the placement.
    pub fn left_edge_offset_of(
        &self,
        position: RoadPosition,
        driving_side: DrivingSide,
    ) -> Distance {
        use RoadPosition::*;

        match position {
            FullWidthCenter => self.total_width() / 2.0,
            Center => {
                // Need to find the midpoint between the first and last occurrence of any roadway.
                let mut left_buffer = Distance::ZERO;
                let mut roadway_width = Distance::ZERO;
                let mut right_buffer = Distance::ZERO;
                for lane in &self.lane_specs_ltr {
                    if !lane.lt.is_roadway() {
                        if roadway_width == Distance::ZERO {
                            left_buffer += lane.width;
                        } else {
                            right_buffer += lane.width;
                        }
                    } else {
                        // It turns out right_buffer was actually a middle buffer.
                        roadway_width += right_buffer;
                        right_buffer = Distance::ZERO;
                        roadway_width += lane.width;
                    }
                }

                if roadway_width == Distance::ZERO {
                    left_buffer / 2.0
                } else {
                    left_buffer + roadway_width / 2.0
                }
            }
            Separation => {
                // Need to find the separating line. This is a common concept that we haven't standardised yet.
                // Search for the first occurrence of a right-hand lane.
                // FIXME contraflow lanes (even bike tracks) will break this.

                let left_dir = if driving_side == DrivingSide::Left {
                    Direction::Fwd
                } else {
                    Direction::Back
                };
                let mut found_first_side = false;
                let mut median_width = Distance::ZERO;
                let mut dist_so_far = Distance::ZERO;
                for lane in &self.lane_specs_ltr {
                    if lane.lt == LaneType::SharedLeftTurn {
                        // "separation" is the middle of this lane by definition.
                        return dist_so_far + lane.width / 2.0;
                    } else if lane.lt.is_tagged_by_lanes_suffix() {
                        if lane.dir == left_dir {
                            found_first_side = true;
                        } else {
                            // We found the change in direction! dist_so_far already includes the whole median.
                            return dist_so_far - median_width / 2.0;
                        }

                        median_width = Distance::ZERO;
                    } else {
                        // If it turns out this lane is part of the median, we need to backtrack later.
                        if found_first_side {
                            median_width += lane.width;
                        }
                    }

                    dist_so_far += lane.width;
                }
                // This is oneway. dist_so_far already includes non-road lanes after the road.
                return dist_so_far - median_width;
            }
            LeftOf(target_lane) | MiddleOf(target_lane) | RightOf(target_lane) => {
                let target_dir = target_lane.direction();
                let target_num = target_lane.number();

                let mut dist_so_far = Distance::ZERO;
                let mut lanes_found = 0;
                // Lanes are counted from the left in the direction of the named lane, so we
                // iterate from the right when we're looking for a backward lane.
                let lanes: Box<dyn Iterator<Item = _>> = if target_dir == Direction::Fwd {
                    Box::new(self.lane_specs_ltr.iter())
                } else {
                    Box::new(self.lane_specs_ltr.iter().rev())
                };
                for lane in lanes {
                    if lane.dir == target_dir {
                        lanes_found += 1;
                        if lanes_found == target_num {
                            // The side of the name lane is defined in the direction of the lane
                            // and we're iterating through the lanes left-to-right in that direction.
                            if let MiddleOf(_) = position {
                                dist_so_far += lane.width / 2.0;
                            } else if let RightOf(_) = position {
                                dist_so_far += lane.width;
                            }

                            return if target_dir == Direction::Fwd {
                                dist_so_far
                            } else {
                                self.total_width() - dist_so_far
                            };
                        }
                    }

                    dist_so_far += lane.width;
                }

                warn!("named lane doesn't exist");
                self.total_width() / 2.0
            }
        }
    }

    /// Returns one PolyLine representing the center of each lane in this road. This must be called
    /// after `Transformation::GenerateIntersectionGeometry` is run. The result also faces the same
    /// direction as the road.
    pub(crate) fn get_lane_center_lines(&self) -> Vec<PolyLine> {
        let total_width = self.total_width();

        let mut width_so_far = Distance::ZERO;
        let mut output = Vec::new();
        for lane in &self.lane_specs_ltr {
            width_so_far += lane.width / 2.0;
            output.push(
                self.center_line
                    .shift_from_center(total_width, width_so_far)
                    .unwrap_or_else(|_| self.center_line.clone()),
            );
            width_so_far += lane.width / 2.0;
        }
        output
    }

    /// Returns the untrimmed left and right side of the road, oriented in the same direction of
    /// the road
    pub fn get_untrimmed_sides(&self, driving_side: DrivingSide) -> Result<(PolyLine, PolyLine)> {
        let total_width = self.total_width();
        let ref_position = match self.reference_line_placement {
            Placement::Consistent(p) => p,
            Placement::Varying(p, _) => p,
            Placement::Transition => RoadPosition::Center, // Best we can do for now.
        };
        let ref_offset = self.left_edge_offset_of(ref_position, driving_side);

        let left = self
            .reference_line
            .shift_from_center(total_width, ref_offset - total_width)?;
        let right = self
            .reference_line
            .shift_from_center(total_width, total_width - ref_offset)?;
        Ok((left, right))
    }

    pub fn endpoints(&self) -> Vec<IntersectionID> {
        vec![self.src_i, self.dst_i]
    }

    pub(crate) fn to_input_road(&self) -> InputRoad {
        InputRoad {
            id: self.id,
            src_i: self.src_i,
            dst_i: self.dst_i,
            center_pts: self.center_line.clone(),
            half_width: self.total_width() / 2.0,
            highway_type: self.highway_type.clone(),
        }
    }

    pub fn other_side(&self, i: IntersectionID) -> IntersectionID {
        RoadWithEndpoints::new(self).other_side(i)
    }

    pub fn common_endpoint(&self, other: &Road) -> CommonEndpoint {
        CommonEndpoint::new((self.src_i, self.dst_i), (other.src_i, other.dst_i))
    }
}

impl StreetNetwork {
    pub fn next_road_id(&mut self) -> RoadID {
        let id = RoadID(self.road_id_counter);
        self.road_id_counter += 1;
        id
    }
}
