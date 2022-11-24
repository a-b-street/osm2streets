use anyhow::Result;
use serde::{Deserialize, Serialize};

use abstutil::Tags;
use geom::{Angle, Distance, PolyLine, Pt2D};

use crate::{
    get_lane_specs_ltr, osm, CommonEndpoint, CrossingType, Direction, InputRoad, IntersectionID,
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

    /// This represents the original OSM geometry. No transformation has happened, besides slightly
    /// smoothing the polyline.
    pub untrimmed_center_line: PolyLine,
    /// The physical center of the road, including sidewalks. This won't actually be trimmed until
    /// `Transformation::GenerateIntersectionGeometry` runs.
    pub trimmed_center_line: PolyLine,
    pub turn_restrictions: Vec<(RestrictionType, RoadID)>,
    /// (via, to). For turn restrictions where 'via' is an entire road. Only BanTurns.
    pub complicated_turn_restrictions: Vec<(RoadID, RoadID)>,
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

    pub lane_specs_ltr: Vec<LaneSpec>,
}

impl Road {
    pub fn new(
        id: RoadID,
        osm_id: OriginalRoad,
        src_i: IntersectionID,
        dst_i: IntersectionID,
        untrimmed_center_line: PolyLine,
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

        Self {
            id,
            osm_ids: vec![osm_id],
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
            untrimmed_center_line,
            trimmed_center_line: PolyLine::dummy(),
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
        self.untrimmed_center_line
            .first_pt()
            .angle_to(self.untrimmed_center_line.last_pt())
    }

    /// The length of the original OSM center line, before any trimming away from intersections
    pub fn untrimmed_length(&self) -> Distance {
        self.untrimmed_center_line.length()
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

    pub fn endpoints(&self) -> Vec<IntersectionID> {
        vec![self.src_i, self.dst_i]
    }

    pub(crate) fn to_input_road(&self) -> InputRoad {
        InputRoad {
            id: self.id,
            src_i: self.src_i,
            dst_i: self.dst_i,
            center_pts: self.trimmed_center_line.clone(),
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
