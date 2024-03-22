use anyhow::Result;
use geojson::Feature;
use geom::{Polygon, Ring};

use super::{serialize_features, Filter};
use crate::road::RoadEdge;
use crate::{Intersection, LaneType, StreetNetwork};

impl StreetNetwork {
    pub fn to_intersection_markings_geojson(&self, filter: &Filter) -> Result<String> {
        let mut features = Vec::new();
        for intersection in filter.intersections(self) {
            for polygon in make_sidewalk_corners(self, intersection) {
                let mut f = Feature::from(polygon.to_geojson(Some(&self.gps_bounds)));
                f.set_property("type", "sidewalk corner");
                features.push(f);
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
