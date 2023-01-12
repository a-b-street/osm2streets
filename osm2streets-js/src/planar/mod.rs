mod build;
mod faces;

use std::collections::{BTreeMap, HashSet};

use geom::{PolyLine, Pt2D};

use osm2streets::StreetNetwork;

pub use self::faces::to_geojson_faces;

pub struct PlanarGraph {
    edges: BTreeMap<EdgeID, Edge>,
    nodes: BTreeMap<HashedPoint, Node>,
}

type HashedPoint = (isize, isize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct EdgeID(HashedPoint, HashedPoint);

struct Node {
    // Sorted clockwise
    edges: Vec<EdgeID>,

    // NOTE the direction here is relative to pointing AT this thing
    // if we can do the lookup... probably just by edge and side?
    oriented_edges: Vec<OrientedEdge>,
}

struct Edge {
    geometry: PolyLine,
    sources: HashSet<String>,
}

impl Node {
    fn next_edge(
        &self,
        _this_node: Pt2D,
        oriented_edge: OrientedEdge,
        _graph: &PlanarGraph,
    ) -> Option<OrientedEdge> {
        let idx = self
            .oriented_edges
            .iter()
            .position(|x| *x == oriented_edge)? as isize;
        // ALWAYS go counter-clockwise. Easy.
        let mut next = abstutil::wraparound_get(&self.oriented_edges, idx - 1).clone();
        // Always flip the direction. We just arrived at this node, now we're going away.
        next.direction = next.direction.opposite();
        Some(next)
    }
}

impl PlanarGraph {
    fn add_node(&mut self, pt: Pt2D) {
        self.nodes.insert(
            hashify(pt),
            Node {
                edges: Vec::new(),
                oriented_edges: Vec::new(),
            },
        );
    }

    fn add_edge(&mut self, pl: PolyLine, source: String) {
        let id = EdgeID(hashify(pl.first_pt()), hashify(pl.last_pt()));
        if id.0 == id.1 {
            //info!("Skipping empty edge at {:?}. raw geom is length {}", id, pl.length());
            return;
        }

        if let Some(edge) = self.edges.get_mut(&id) {
            //info!("Already have {:?}, skipping duplicate edge", id);
            edge.sources.insert(source);
            return;
        }

        // TODO be very careful, always work with HashedPoints in here, not Pt2D.
        // do geo::LineString<isize> or something to be paranoid.
        let endpts = [id.0, id.1];

        let mut sources = HashSet::new();
        sources.insert(source);
        self.edges.insert(
            id,
            Edge {
                geometry: pl,
                sources,
            },
        );
        for endpt in endpts {
            let node = self.nodes.get_mut(&endpt).unwrap();
            node.edges.push(id);

            // Re-sort the node
            // (This is the same logic as Intersection::sort_roads)
            let mut pointing_to_node = Vec::new();
            let mut average_endpts = Vec::new();
            for e in &node.edges {
                let mut pl = self.edges[e].geometry.clone();
                if hashify(pl.first_pt()) == endpt {
                    pl = pl.reversed();
                } else {
                    assert_eq!(hashify(pl.last_pt()), endpt);
                }
                average_endpts.push(pl.last_pt());
                pointing_to_node.push((*e, pl, Pt2D::zero()));
            }

            let true_center = Pt2D::center(&average_endpts);
            let shortest_edge = pointing_to_node
                .iter()
                .map(|(_, pl, _)| pl.length())
                .min()
                .unwrap();
            for (_, pl, sorting_pt) in &mut pointing_to_node {
                *sorting_pt = pl.must_dist_along(pl.length() - shortest_edge).0;
            }
            pointing_to_node.sort_by_key(|(_, _, sorting_pt)| {
                sorting_pt.angle_to(true_center).normalized_degrees() as i64
            });

            node.edges = pointing_to_node.into_iter().map(|(id, _, _)| id).collect();

            self.recalculate_oriented_edges(endpt);
        }
    }

    fn recalculate_oriented_edges(&mut self, id: HashedPoint) {
        let node = self.nodes.get_mut(&id).unwrap();

        // trust the edge ordering
        // this is RoadEdge::calculate logic
        node.oriented_edges.clear();
        for e in &node.edges {
            let mut left = OrientedEdge {
                edge: *e,
                side: Side::Left,
                // Tmp
                direction: Direction::Forwards,
            };
            let mut right = OrientedEdge {
                edge: *e,
                side: Side::Right,
                // Tmp
                direction: Direction::Forwards,
            };
            if hashify(self.edges[e].geometry.first_pt()) == id {
                left.direction = Direction::Backwards;
                right.direction = Direction::Backwards;
                node.oriented_edges.push(left);
                node.oriented_edges.push(right);
            } else {
                assert_eq!(hashify(self.edges[e].geometry.last_pt()), id);
                node.oriented_edges.push(right);
                node.oriented_edges.push(left);
            }
        }
    }

    pub fn remove_all_deadends(&mut self) {
        // Horrible fixpoint
        loop {
            if let Some((n, _)) = self.nodes.iter().find(|(_, node)| node.edges.len() == 1) {
                self.remove_deadend(*n)
            } else {
                break;
            }
        }
    }

    fn remove_deadend(&mut self, delete: HashedPoint) {
        assert_eq!(self.nodes[&delete].edges.len(), 1);
        let e = self.nodes.remove(&delete).unwrap().edges[0];
        let keep = if e.0 == delete { e.1 } else { e.0 };

        self.edges.remove(&e).unwrap();

        // Clean up the other node
        self.nodes.get_mut(&keep).unwrap().edges.retain(|x| *x != e);
        self.recalculate_oriented_edges(keep);
    }
}

#[derive(Clone, PartialEq, Debug)]
struct OrientedEdge {
    edge: EdgeID,
    // side is ABSOLUTE to the original forwards orientation of the edge. NOT relative to the
    // direction.
    side: Side,
    direction: Direction,
}

impl OrientedEdge {
    fn to_points(&self, graph: &PlanarGraph) -> Vec<Pt2D> {
        let mut pts = graph.edges[&self.edge].geometry.clone().into_points();
        if self.direction == Direction::Backwards {
            pts.reverse();
        }
        pts
    }

    fn last_pt(&self, graph: &PlanarGraph) -> Pt2D {
        let edge = &graph.edges[&self.edge];
        match self.direction {
            Direction::Forwards => edge.geometry.last_pt(),
            Direction::Backwards => edge.geometry.first_pt(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Side {
    Left,
    Right,
}

impl Side {
    fn _opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Direction {
    Forwards,
    Backwards,
}
impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Forwards => Self::Backwards,
            Self::Backwards => Self::Forwards,
        }
    }
}

pub fn to_geojson_network(streets: &StreetNetwork) -> String {
    let graph = build::streets_to_planar(streets);
    graph.render_network(&streets.gps_bounds)
}

fn hashify(pt: Pt2D) -> HashedPoint {
    let x = (pt.x() * 10.0) as isize;
    let y = (pt.y() * 10.0) as isize;
    (x, y)
}

fn unhashify(pt: HashedPoint) -> Pt2D {
    let x = pt.0 as f64 / 10.0;
    let y = pt.1 as f64 / 10.0;
    Pt2D::new(x, y)
}
