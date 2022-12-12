use geom::{PolyLine, Ring, Polygon};

use super::Document;

impl Document {
    // TODO This destroys the guarantee that the Document represents raw OSM. Do we need to be
    // careful with lane_editor? Since it just uses node IDs and we don't filter those, it should
    // be OK...
    pub fn clip(&mut self, boundary_polygon: &Polygon) {
        // Remove all nodes that're out-of-bounds. Don't fix up ways and relations referring to
        // these.
        self.nodes.retain(|_, node| boundary_polygon.contains_pt(node.pt));

        // Remove ways that have no nodes within bounds.
        // TODO If there's a way that geometrically crosses the boundary but only has nodes outside
        // it, this'll remove it. Is that desirable?
        self.ways.retain(|_, way| way.nodes.iter().any(|node| self.nodes.contains_key(node)));

        // For line-string ways (not areas), clip them to the boundary. way.pts and way.nodes
        // become out-of-sync.
        for way in self.ways.values_mut() {
            // TODO This could just be a cul-de-sac road
            if way.pts[0] == *way.pts.last().unwrap() {
                continue;
            }

            let pl = PolyLine::unchecked_new(way.pts.clone());
            way.pts = clip_polyline_to_ring(pl, boundary_polygon.get_outer_ring()).into_points();
        }

        // TODO Handle ways that're areas
        // TODO Handle relations
    }
}

// TODO Move to geom and test better
fn clip_polyline_to_ring(pl: PolyLine, ring: &Ring) -> PolyLine {
    let hits = ring.all_intersections(&pl);
    if hits.len() == 2 {
        if let Some((mut dist1, _)) = pl.dist_along_of_point(hits[0]) {
            if let Some((mut dist2, _)) = pl.dist_along_of_point(hits[1]) {
                if dist1 > dist2 {
                    std::mem::swap(&mut dist1, &mut dist2);
                }
                if let Ok(slice) = pl.maybe_exact_slice(dist1, dist2) {
                    return slice;
                }
            }
        }
    }
    // If this fails for any reason, just return the input untransformed
    pl
}
