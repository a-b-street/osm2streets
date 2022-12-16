use geom::{Distance, PolyLine, Polygon};

use super::Document;

impl Document {
    // TODO This destroys the guarantee that the Document represents raw OSM. Do we need to be
    // careful with lane_editor? Since it just uses node IDs and we don't filter those, it should
    // be OK...
    pub fn clip(&mut self, boundary_polygon: &Polygon) {
        // Remove all nodes that're out-of-bounds. Don't fix up ways and relations referring to
        // these.
        self.nodes
            .retain(|_, node| boundary_polygon.contains_pt(node.pt));

        // Remove ways that have no nodes within bounds.
        // TODO If there's a way that geometrically crosses the boundary but only has nodes outside
        // it, this'll remove it. Is that desirable?
        self.ways
            .retain(|_, way| way.nodes.iter().any(|node| self.nodes.contains_key(node)));

        // For line-string ways (not areas), clip them to the boundary. way.pts and way.nodes
        // become out-of-sync.
        for (id, way) in &mut self.ways {
            // TODO This could just be a cul-de-sac road
            if way.pts[0] == *way.pts.last().unwrap() {
                continue;
            }

            let mut polylines =
                clip_polyline_to_ring(PolyLine::unchecked_new(way.pts.clone()), boundary_polygon);
            // Usually there's just one result
            if polylines.len() == 1 {
                way.pts = polylines.pop().unwrap().into_points();
                continue;
            }

            // But occasionally a road crossing the boundary multiple times will get split into
            // multiple pieces. In that case, make copies of the way, each with their own geometry.
            for pl in polylines {
                let mut copy = way.clone();
                copy.pts = pl.into_points();
                self.clipped_copied_ways.push((*id, copy));
            }
        }

        // Remove the "original" from ways that were split into pieces
        for (id, _) in &self.clipped_copied_ways {
            self.ways.remove(id);
        }

        // TODO Handle ways that're areas
        // TODO Handle relations
    }
}

/// Split a polyline into potentially multiple pieces by clipping it against a polygon boundary.
/// Only return slices within the polygon.
// TODO Move to geom and test better
fn clip_polyline_to_ring(pl: PolyLine, polygon: &Polygon) -> Vec<PolyLine> {
    let mut hit_distances = Vec::new();
    for pt in polygon.get_outer_ring().all_intersections(&pl) {
        if let Some((dist, _)) = pl.dist_along_of_point(pt) {
            hit_distances.push(dist);
        } else {
            // This shouldn't happen, but just return the input untransformed if it does
            return vec![pl];
        }
    }
    hit_distances.sort();

    // Split the PolyLine into pieces, every time it crosses the polygon
    let mut start = Distance::ZERO;

    let mut slices = Vec::new();
    for dist in hit_distances {
        // The slice may be tiny; skip if so
        if let Ok(slice) = pl.maybe_exact_slice(start, dist) {
            slices.push(slice);
        }
        start = dist;
    }
    // And the last piece
    slices.extend(pl.maybe_exact_slice(start, pl.length()));

    // Only keep slices in bounds
    slices.retain(|pl| polygon.contains_pt(pl.middle()));

    slices
}
