use anyhow::Result;
use geojson::Feature;
use geom::{Distance, Line, PolyLine, Polygon, Pt2D, Ring};

use super::{serialize_features, Filter};
use crate::road::RoadEdge;
use crate::{CrossingKind, Intersection, LaneType, Road, StreetNetwork};

impl StreetNetwork {
    pub fn to_intersection_markings_geojson(&self, filter: &Filter) -> Result<String> {
        let mut features = Vec::new();
        for intersection in filter.intersections(self) {
            for polygon in make_sidewalk_corners(self, intersection) {
                let mut f = Feature::from(polygon.to_geojson(Some(&self.gps_bounds)));
                f.set_property("type", "sidewalk corner");
                features.push(f);
            }

            if let Some(ref crossing) = intersection.crossing {
                match crossing.kind {
                    CrossingKind::Signalized | CrossingKind::Marked => {
                        for polygon in draw_zebra_crossing(self, intersection) {
                            let mut f = Feature::from(polygon.to_geojson(Some(&self.gps_bounds)));
                            f.set_property("type", "marked crossing line");
                            features.push(f);
                        }
                    }
                    CrossingKind::Unmarked => {
                        for polygon in draw_unmarked_crossing(self, intersection) {
                            let mut f = Feature::from(polygon.to_geojson(Some(&self.gps_bounds)));
                            f.set_property("type", "unmarked crossing outline");
                            features.push(f);
                        }
                    }
                }
            }
        }
        serialize_features(features)
    }
}

/// For an intersection, show all corners where sidewalks meet.
fn make_sidewalk_corners(streets: &StreetNetwork, intersection: &Intersection) -> Vec<Polygon> {
    // Look at every adjacent pair of edges
    let mut edges = RoadEdge::calculate(
        streets.roads_per_intersection(intersection.id),
        intersection.id,
    );
    if edges.is_empty() {
        error!("Intersection {:?} has no edges", intersection.osm_ids);
        return Vec::new();
    }
    edges.push(edges[0].clone());
    let mut results = Vec::new();
    for pair in edges.windows(2) {
        let one = &pair[0];
        let two = &pair[1];

        // Only want corners between two roads
        if one.road == two.road {
            continue;
        }

        // Only want two sidewalks or shoulders
        if !(one.lane.lt == LaneType::Sidewalk || one.lane.lt == LaneType::Shoulder) {
            continue;
        }
        if !(two.lane.lt == LaneType::Sidewalk || two.lane.lt == LaneType::Shoulder) {
            continue;
        }

        // Only want roads with more than just a sidewalk/shoulder lane
        if streets.roads[&one.road].lane_specs_ltr.len() == 1
            || streets.roads[&two.road].lane_specs_ltr.len() == 1
        {
            continue;
        }

        // These points should be right on the intersection polygon
        let outer_corner1 = one.pl.last_pt();
        let outer_corner2 = two.pl.last_pt();

        let mut pts_along_intersection = Vec::new();
        if outer_corner1 == outer_corner2 {
            pts_along_intersection.push(outer_corner1);
        } else {
            if let Some(pl) = intersection
                .polygon
                .get_outer_ring()
                .get_shorter_slice_btwn(outer_corner1, outer_corner2)
            {
                pts_along_intersection = pl.into_points();
            } else {
                // Something went wrong; bail out
                continue;
            }
        }

        // Now find the inner sides of each sidewalk.
        let inner_pl1 = one.pl.must_shift_right(one.lane.width);
        let inner_pl2 = two.pl.must_shift_left(two.lane.width);

        // Imagine the inner lines extended into the intersection. If the point where they meet is
        // still inside the intersection, let's use it.
        let mut meet_pt = None;
        if let Some(pt) = inner_pl1
            .last_line()
            .infinite()
            .intersection(&inner_pl2.last_line().infinite())
        {
            if intersection.polygon.contains_pt(pt) {
                meet_pt = Some(pt);
            }
        }

        // Assemble everything into a ring.

        // This points from one to two, tracing along the intersection.
        let mut pts = pts_along_intersection;
        // Now add two's inner corner
        pts.push(inner_pl2.last_pt());
        // If we have a point where the two infinite lines meet, use it
        pts.extend(meet_pt);
        // one's inner corner
        pts.push(inner_pl1.last_pt());
        // Close the ring
        pts.push(pts[0]);

        if let Ok(ring) = Ring::deduping_new(pts) {
            results.push(ring.into_polygon());
        }
    }
    results
}

// TODO The alternate approach would be to preserve the original OSM way representing the crossing,
// even as the intersection polygon gets built
fn get_crossing_line_and_min_width(
    streets: &StreetNetwork,
    intersection: &Intersection,
) -> Option<(PolyLine, Distance)> {
    // Find the pedestrian roads making up the crossing, and the endpoint on the side of the
    // intersection for each
    let mut roads = Vec::new();
    for r in &intersection.roads {
        let road = &streets.roads[r];
        if road.is_footway() {
            let endpt = center_line_pointed_at(road, intersection).last_pt();
            roads.push((road, endpt));
        }
    }

    // Normally there are exactly two to connect. But when a tiny road gets consolidated, there'll
    // be more. Pick the pair farthest away from each other as a heuristic.
    let ((r1, pt1), (r2, pt2)) = farthest_pair(roads)?;

    // Create the line connecting these two roads.
    // TODO Subset the reference_lines by trim_start/end to get more detail
    let pl = PolyLine::new(vec![pt1, pt2]).ok()?;
    let width = r1.total_width().min(r2.total_width());
    Some((pl, width))
}

fn farthest_pair(candidates: Vec<(&Road, Pt2D)>) -> Option<((&Road, Pt2D), (&Road, Pt2D))> {
    let mut max_dist = Distance::ZERO;
    let mut max_pair = None;

    for idx1 in 0..candidates.len() {
        for idx2 in (idx1 + 1)..candidates.len() {
            let dist = candidates[idx1].1.dist_to(candidates[idx2].1);
            if dist > max_dist {
                max_dist = dist;
                max_pair = Some((candidates[idx1], candidates[idx2]));
            }
        }
    }

    max_pair
}

fn draw_zebra_crossing(streets: &StreetNetwork, intersection: &Intersection) -> Vec<Polygon> {
    let mut results = Vec::new();
    let Some((line, total_width)) = get_crossing_line_and_min_width(streets, intersection) else {
        return results;
    };

    // Pretty arbitrary parameters
    let width = 0.8 * total_width;
    let thickness = Distance::meters(0.15);
    let step_size = 3.0 * thickness;
    let buffer_ends = step_size;
    for (pt1, angle) in line.step_along(step_size, buffer_ends) {
        // Project away an arbitrary amount
        let pt2 = pt1.project_away(Distance::meters(1.0), angle);
        results.push(perp_line(Line::must_new(pt1, pt2), width).make_polygons(thickness));
    }

    results
}

fn draw_unmarked_crossing(streets: &StreetNetwork, intersection: &Intersection) -> Vec<Polygon> {
    let mut results = Vec::new();
    let Some((line, total_width)) = get_crossing_line_and_min_width(streets, intersection) else {
        return results;
    };

    let width = 0.8 * total_width;
    let thickness = Distance::meters(0.15);

    for shift in [width / 2.0, -width / 2.0] {
        if let Ok(pl) = line.shift_either_direction(shift) {
            results.push(pl.make_polygons(thickness));
        }
    }

    results
}

fn center_line_pointed_at(road: &Road, intersection: &Intersection) -> PolyLine {
    if road.dst_i == intersection.id {
        road.center_line.clone()
    } else {
        road.center_line.reversed()
    }
}

// this always does it at pt1
fn perp_line(l: Line, length: Distance) -> Line {
    let pt1 = l.shift_right(length / 2.0).pt1();
    let pt2 = l.shift_left(length / 2.0).pt1();
    Line::must_new(pt1, pt2)
}
