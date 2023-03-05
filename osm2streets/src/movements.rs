use crate::intersection::TurnMovement;
use crate::lanes::{LtrLaneNum, RoadPosition};
use crate::road::{Road, RoadEnd};
use crate::{Direction, DrivingSide, LaneID};
use geom::{PolyLine, Pt2D};

pub fn from_placement(
    (src_road, src_end): (&Road, RoadEnd),
    (dst_road, dst_end): (&Road, RoadEnd),
) -> Option<Vec<TurnMovement>> {
    let src_pos = src_road.reference_line_placement.road_position_at(src_end);
    let dst_pos = dst_road.reference_line_placement.road_position_at(dst_end);
    if let (Some(RoadPosition::MiddleOf(src_lane)), Some(RoadPosition::MiddleOf(dst_lane))) =
        (src_pos, dst_pos)
    {
        with_matched_lanes((src_road, src_end, src_lane), (dst_road, dst_end, dst_lane))
    } else if let (Some(Some(src_lane)), Some(Some(dst_lane))) = (
        src_pos.map(RoadPosition::left_of_lane),
        dst_pos.map(RoadPosition::left_of_lane),
    ) {
        with_matched_lanes((src_road, src_end, src_lane), (dst_road, dst_end, dst_lane))
    } else {
        None
    }
}

/// Generate the movements coming out of the src_end end of src_road into the dst_end end of
/// dst_road, where src_lane matches up with dst_lane.
///
/// Lanes are paired up in order. Unpaired lanes fork from or merge into their closest partner lane.
fn with_matched_lanes(
    (src_road, src_end, src_lane): (&Road, RoadEnd, LtrLaneNum),
    (dst_road, dst_end, dst_lane): (&Road, RoadEnd, LtrLaneNum),
) -> Option<Vec<TurnMovement>> {
    if src_end.direction_towards() != src_lane.direction()
        || dst_end.direction_from() != dst_lane.direction()
    {
        return None;
    }

    // Figure out which lanes (left-to-right) are involved in driving from src to dst.
    let src_lanes = match src_end {
        RoadEnd::End => driving_lane_endpoints(src_road, Direction::Fwd, RoadEnd::End),
        RoadEnd::Start => driving_lane_endpoints(src_road, Direction::Back, RoadEnd::Start),
    };
    let dst_lanes = match dst_end {
        RoadEnd::Start => driving_lane_endpoints(dst_road, Direction::Fwd, RoadEnd::Start),
        RoadEnd::End => driving_lane_endpoints(dst_road, Direction::Back, RoadEnd::End),
    };

    if src_lanes.is_empty() || dst_lanes.is_empty() {
        return Some(Vec::new());
    }

    let src_len = src_lanes.len();
    let dst_len = dst_lanes.len();
    let mut lanes_offset = (src_lane.number() as isize) - (dst_lane.number() as isize);
    let mut src_i: usize = 0;
    let mut dst_i: usize = 0;

    // Pair up the lanes, offsetting one side so the match lanes line up.
    let mut result = Vec::new();
    loop {
        let (src_lane, src_pt) = src_lanes[src_i];
        let (dst_lane, dst_pt) = dst_lanes[dst_i];
        result.push(TurnMovement {
            from: src_lane,
            to: dst_lane,
            path: PolyLine::new(vec![src_pt, dst_pt])
                .unwrap_or_else(|_| PolyLine::must_new(vec![src_pt, Pt2D::new(0.0, 0.0), dst_pt])),
        });

        // Increment the appropriate road until the offset is reached...
        if lanes_offset > 0 {
            src_i += 1;
            lanes_offset -= 1;
        } else if lanes_offset < 0 {
            dst_i += 1;
            lanes_offset += 1;
        }
        // ...otherwise increment both that haven't run out of lanes.
        else {
            let mut advanced = false;
            if src_i + 1 < src_len {
                src_i += 1;
                advanced = true;
            }
            if dst_i + 1 < dst_len {
                dst_i += 1;
                advanced = true;
            }
            if !advanced {
                break;
            }
        }
    }
    Some(result)
}

/// Limitations: ignores both ways lanes
pub fn default(
    (src_road, src_end): (&Road, RoadEnd),
    (dst_road, dst_end): (&Road, RoadEnd),
    driving_side: DrivingSide,
) -> Option<Vec<TurnMovement>> {
    let mut result = Vec::new();

    // Figure out which lanes (left-to-right) are involved in driving from src to dst.
    let mut src_lanes = match src_end {
        RoadEnd::End => driving_lane_endpoints(src_road, Direction::Fwd, RoadEnd::End),
        RoadEnd::Start => driving_lane_endpoints(src_road, Direction::Back, RoadEnd::Start),
    };
    let mut dst_lanes = match dst_end {
        RoadEnd::Start => driving_lane_endpoints(dst_road, Direction::Fwd, RoadEnd::Start),
        RoadEnd::End => driving_lane_endpoints(dst_road, Direction::Back, RoadEnd::End),
    };

    if src_lanes.is_empty() || dst_lanes.is_empty() {
        return Some(result);
    }

    // Order the lanes inside-to-outside.
    if driving_side == DrivingSide::Left {
        src_lanes.reverse();
        dst_lanes.reverse();
    }

    let src_len = src_lanes.len();
    let dst_len = dst_lanes.len();
    let mut src_i: usize = 0;
    let mut dst_i: usize = 0;

    // Pair up the lanes, matching the inside lanes to each other.
    loop {
        let (src_lane, src_pt) = src_lanes[src_i];
        let (dst_lane, dst_pt) = dst_lanes[dst_i];
        result.push(TurnMovement {
            from: src_lane,
            to: dst_lane,
            path: PolyLine::new(vec![src_pt, dst_pt])
                .unwrap_or_else(|_| PolyLine::must_new(vec![src_pt, Pt2D::new(0.0, 0.0), dst_pt])),
        });

        // If either road is out of lanes, don't advance the counter, reuse the last lane.
        let mut advanced = false;
        if src_i + 1 < src_len {
            src_i += 1;
            advanced = true;
        }
        if dst_i + 1 < dst_len {
            dst_i += 1;
            advanced = true;
        }
        if !advanced {
            break;
        }
    }

    Some(result)
}

/// Returns the driving lanes in the given direction (left-to-right when facing that direction) and
/// their endpoints at the given end.
fn driving_lane_endpoints(road: &Road, dir: Direction, end: RoadEnd) -> Vec<(LaneID, Pt2D)> {
    let lanes_ltr = road
        .lane_specs_ltr
        .iter()
        .zip(road.get_lane_end_points(end))
        .enumerate()
        .filter_map(|(index, (lane, pt))| {
            if lane.lt.is_tagged_by_lanes_suffix() && lane.dir == dir {
                Some((
                    LaneID {
                        road: road.id,
                        index,
                    },
                    pt,
                ))
            } else {
                None
            }
        });
    match dir {
        Direction::Fwd => lanes_ltr.collect(),
        Direction::Back => lanes_ltr.rev().collect(),
    }
}
