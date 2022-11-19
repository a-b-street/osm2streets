use anyhow::Result;
use serde::{Deserialize, Serialize};

use abstutil::Tags;
use geom::{Angle, Distance, PolyLine, Pt2D};

use crate::{
    get_lane_specs_ltr, osm, CrossingType, Direction, InputRoad, LaneSpec, LaneType, MapConfig,
    OriginalRoad, RestrictionType,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Road {
    pub id: OriginalRoad,
    pub src_i: osm::NodeID,
    pub dst_i: osm::NodeID,
    /// This represents the original OSM geometry. No transformation has happened, besides slightly
    /// smoothing the polyline.
    pub untrimmed_center_line: PolyLine,
    /// The physical center of the road, including sidewalks. This won't actually be trimmed until
    /// `Transformation::GenerateIntersectionGeometry` runs.
    pub trimmed_center_line: PolyLine,
    pub osm_tags: Tags,
    pub turn_restrictions: Vec<(RestrictionType, OriginalRoad)>,
    /// (via, to). For turn restrictions where 'via' is an entire road. Only BanTurns.
    pub complicated_turn_restrictions: Vec<(OriginalRoad, OriginalRoad)>,
    pub percent_incline: f64,
    /// Is there a tagged crosswalk near each end of the road?
    pub crosswalk_forward: bool,
    pub crosswalk_backward: bool,
    // TODO Preserving these two across transformations (especially merging dual carriageways!)
    // could be really hard. It might be better to split the road into two pieces to match the more
    // often used OSM style.
    /// Barrier nodes along this road's original center line.
    pub barrier_nodes: Vec<Pt2D>,
    /// Crossing nodes along this road's original center line.
    pub crossing_nodes: Vec<(Pt2D, CrossingType)>,

    /// Derived from osm_tags. Not automatically updated.
    pub lane_specs_ltr: Vec<LaneSpec>,
}

impl Road {
    pub fn new(
        id: OriginalRoad,
        src_i: osm::NodeID,
        dst_i: osm::NodeID,
        untrimmed_center_line: PolyLine,
        osm_tags: Tags,
        config: &MapConfig,
    ) -> Self {
        let lane_specs_ltr = get_lane_specs_ltr(&osm_tags, config);
        Self {
            id,
            src_i,
            dst_i,
            untrimmed_center_line,
            trimmed_center_line: PolyLine::dummy(),
            osm_tags,
            turn_restrictions: Vec::new(),
            complicated_turn_restrictions: Vec::new(),
            percent_incline: 0.0,
            // Start assuming there's a crosswalk everywhere, and maybe filter it down
            // later
            crosswalk_forward: true,
            crosswalk_backward: true,
            barrier_nodes: Vec::new(),
            crossing_nodes: Vec::new(),

            lane_specs_ltr,
        }
    }

    // TODO For the moment, treating all rail things as light rail
    pub fn is_light_rail(&self) -> bool {
        self.osm_tags.is_any("railway", vec!["light_rail", "rail"])
    }

    pub fn is_footway(&self) -> bool {
        self.osm_tags.is_any(
            osm::HIGHWAY,
            vec![
                // TODO cycleway in here is weird, reconsider. is_footway is only used in one
                // disabled transformation right now.
                "cycleway",
                "footway",
                "path",
                "pedestrian",
                "steps",
                "track",
            ],
        )
    }

    pub fn is_service(&self) -> bool {
        self.osm_tags.is(osm::HIGHWAY, "service")
    }

    pub fn is_cycleway(&self) -> bool {
        // Don't repeat the logic looking at the tags, just see what lanes we'll create
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

    /// Points from first to last point. Undefined for loops.
    pub fn angle(&self) -> Angle {
        self.untrimmed_center_line
            .first_pt()
            .angle_to(self.untrimmed_center_line.last_pt())
    }

    /// The length of the original OSM center line, before any trimming away from intersections
    pub fn untrimmed_length(&self) -> Distance {
        self.untrimmed_center_line.length()
    }

    pub fn get_zorder(&self) -> isize {
        if let Some(layer) = self.osm_tags.get("layer") {
            match layer.parse::<f64>() {
                // Just drop .5 for now
                Ok(l) => l as isize,
                Err(_) => {
                    warn!("Weird layer={} on {}", layer, self.osm_url());
                    0
                }
            }
        } else {
            0
        }
    }

    /// Returns the corrected (but untrimmed) center and total width for a road
    pub fn untrimmed_road_geometry(&self) -> (PolyLine, Distance) {
        let mut total_width = Distance::ZERO;
        let mut sidewalk_right = None;
        let mut sidewalk_left = None;
        for l in &self.lane_specs_ltr {
            total_width += l.width;
            if l.lt.is_walkable() {
                if l.dir == Direction::Back {
                    sidewalk_left = Some(l.width);
                } else {
                    sidewalk_right = Some(l.width);
                }
            }
        }

        // If there's a sidewalk on only one side, adjust the true center of the road.
        // TODO I don't remember the rationale for doing this in the first place. What if there's a
        // shoulder and a sidewalk of different widths? We don't do anything then
        let mut true_center = self.untrimmed_center_line.clone();
        match (sidewalk_right, sidewalk_left) {
            (Some(w), None) => {
                true_center = true_center.must_shift_right(w / 2.0);
            }
            (None, Some(w)) => {
                true_center = true_center.must_shift_right(w / 2.0);
            }
            _ => {}
        }

        (true_center, total_width)
    }

    pub fn osm_url(&self) -> String {
        // Since we don't store an OriginalRoad (since we may need to update it during
        // transformations), this may be convenient
        format!(
            "http://openstreetmap.org/way/{}",
            self.osm_tags.get(osm::OSM_WAY_ID).unwrap()
        )
    }

    pub fn total_width(&self) -> Distance {
        self.lane_specs_ltr.iter().map(|l| l.width).sum()
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
                self.trimmed_center_line
                    .shift_from_center(total_width, width_so_far)
                    .unwrap_or_else(|_| self.trimmed_center_line.clone()),
            );
            width_so_far += lane.width / 2.0;
        }
        output
    }

    /// Returns the untrimmed left and right side of the road, oriented in the same direction of
    /// the road
    pub fn get_untrimmed_sides(&self) -> Result<(PolyLine, PolyLine)> {
        let (center, total_width) = self.untrimmed_road_geometry();
        let left = center.shift_from_center(total_width, -total_width / 2.0)?;
        let right = center.shift_from_center(total_width, total_width / 2.0)?;
        Ok((left, right))
    }

    pub fn endpoints(&self) -> Vec<osm::NodeID> {
        vec![self.src_i, self.dst_i]
    }

    pub(crate) fn to_input_road(&self) -> InputRoad {
        InputRoad {
            id: self.id,
            src_i: self.src_i,
            dst_i: self.dst_i,
            center_pts: self.trimmed_center_line.clone(),
            half_width: self.total_width() / 2.0,
            osm_tags: self.osm_tags.clone(),
        }
    }
}
