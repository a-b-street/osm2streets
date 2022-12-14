use std::collections::BTreeMap;

use geo::Winding;
use geom::{HashablePt2D, PolyLine, Pt2D, GPSBounds, Ring, Polygon};

use osm2streets::{RoadID, StreetNetwork};

struct PlanarGraph {
    edges: BTreeMap<EdgeID, PolyLine>,
    nodes: BTreeMap<HashablePt2D, Node>,
    gps_bounds: GPSBounds,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum EdgeID {
    Road(RoadID),
    // TODO Boundary(IntersectionID, IntersectionID)  meaning clockwise
}

struct Node {
    // Sorted clockwise
    edges: Vec<EdgeID>,

    // TODO can we do RoadEdge::calculate type logic here?
}

impl Node {
    fn next_edge(&self, this_node: Pt2D, oriented_edge: OrientedEdge, graph: &PlanarGraph) -> OrientedEdge {
        let idx = self.edges.iter().position(|x| *x == oriented_edge.edge).unwrap();

        // TODO Confusing... side is relative to direction too.
        let mut side_for_offset = oriented_edge.side;
        if oriented_edge.direction == Direction::Backwards {
            side_for_offset = side_for_offset.opposite();
        }
        let offset = if side_for_offset == Side::Right {
            // Go counter-clockwise, because we're on the "inside"
            -1
        } else {
            1
        };
        let next_edge = *abstutil::wraparound_get(&self.edges, idx as isize + offset);

        let next_dir = if graph.edges[&next_edge].first_pt() == this_node {
            Direction::Forwards
        } else {
            Direction::Backwards
        };

        let next_side = if next_dir == Direction::Forwards {
            // TODO Right? Or the same?
            oriented_edge.side
        } else {
            oriented_edge.side.opposite()
        };

        OrientedEdge {
            edge: next_edge,
            side: next_side,
            direction: next_dir,
        }
    }
}

impl PlanarGraph {
    fn new(gps_bounds: GPSBounds) -> Self {
        Self {
            edges: BTreeMap::new(),
            nodes: BTreeMap::new(),
            gps_bounds,
        }
    }

    fn add_edge(&mut self, id: EdgeID, pl: PolyLine) {
        let endpts = [pl.first_pt(), pl.last_pt()];
        self.edges.insert(id, pl);
        for endpt in endpts {
            let node = self.nodes.entry(endpt.to_hashable()).or_insert_with(|| Node { edges: Vec::new() });
            node.edges.push(id);

            // Re-sort the node
            // (This is the same logic as Intersection::sort_roads)
            let mut pointing_to_node = Vec::new();
            let mut average_endpts = Vec::new();
            for e in &node.edges {
                let mut pl = self.edges[e].clone();
                if pl.first_pt() == endpt {
                    pl = pl.reversed();
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
                sorting_pt
                    .angle_to(true_center)
                    .normalized_degrees() as i64
            });

            node.edges = pointing_to_node.into_iter().map(|(id, _, _)| id).collect();
        }
    }

    fn render_edges(&self) -> String {
        let mut pairs = Vec::new();

        for (_, pl) in &self.edges {
            let mut props = serde_json::Map::new();
            props.insert("stroke".to_string(), true.into());
            props.insert("color".to_string(), "cyan".into());
            pairs.push((pl.to_geojson(Some(&self.gps_bounds)), props));
        }

        abstutil::to_json(&geom::geometries_with_properties_to_geojson(pairs))
    }

    fn to_faces(&self) -> Vec<Face> {
        let mut faces = Vec::new();

        //faces.extend(self.trace_face(EdgeID::Road(RoadID(46)), Side::Right));
        faces.extend(self.trace_face(EdgeID::Road(RoadID(33)), Side::Left));

        for e in self.edges.keys() {
            //faces.extend(self.trace_face(*e, Side::Right));
            //faces.extend(self.trace_face(*e, Side::Left));
        }
        faces
    }

    fn trace_face(&self, start_edge: EdgeID, start_side: Side) -> Option<Face> {
        // Initial direction depends on the orientation of the edge! We MUST go clockwise.
        let ls: geo::LineString = (&self.edges[&start_edge]).into();
        // TODO Handle no winding order
        let start_direction = if ls.is_cw() {
            // The order is funny here because...
            Direction::Backwards
        } else {
            Direction::Forwards
        };
        let start = OrientedEdge {
            edge: start_edge,
            side: start_side,
            direction: start_direction,
        };

        let mut members = Vec::new();
        // TODO Build this up at the same time or not?
        let mut pts = Vec::new();

        let mut current = start.clone();
        loop {
            // TODO bail out
            if members.len() > 10 {
                break;
            }

            members.push(current.clone());
            if current == start && !pts.is_empty() {
                pts.push(pts[0]);
                break;
            }
            pts.extend(current.to_points(self));

            let endpt = current.last_pt(self);
            current = self.nodes[&endpt.to_hashable()].next_edge(endpt, current, self);
        }

        info!("trace_face found {} members, {} pts", members.len(), pts.len());
        for x in &members {
            info!("  - {:?}", x);
        }

        if let Ok(ring) = Ring::deduping_new(pts) {
            Some(Face {
                members,
                polygon: ring.into_polygon(),
            })
        } else {
            None
        }
    }
}

struct Face {
    polygon: Polygon,
    // Clockwise and first=last
    members: Vec<OrientedEdge>,
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
        let mut pts = graph.edges[&self.edge].clone().into_points();
        if self.direction == Direction::Backwards {
            pts.reverse();
        }
        pts
    }

    fn last_pt(&self, graph: &PlanarGraph) -> Pt2D {
        let edge = &graph.edges[&self.edge];
        match self.direction {
            Direction::Forwards => edge.last_pt(),
            Direction::Backwards => edge.first_pt(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Side {
    Left,
    Right,
}

impl Side {
    fn opposite(self) -> Self {
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

fn streets_to_planar(streets: &StreetNetwork) -> PlanarGraph {
    let mut graph = PlanarGraph::new(streets.gps_bounds.clone());
    for road in streets.roads.values() {
        graph.add_edge(EdgeID::Road(road.id), road.reference_line.clone());
    }
    graph
}

pub fn to_geojson(streets: &StreetNetwork) -> String {
    //streets_to_planar(streets).render_edges()

    let mut pairs = Vec::new();
    for face in streets_to_planar(streets).to_faces() {
        info!("found a face with {} members", face.members.len());
        let mut props = serde_json::Map::new();
        props.insert("fill".to_string(), true.into());
        props.insert("fillColor".to_string(), "cyan".into());
        props.insert("fillOpacity".to_string(), 0.5.into());
        pairs.push((face.polygon.to_geojson(Some(&streets.gps_bounds)), props));
    }
    abstutil::to_json(&geom::geometries_with_properties_to_geojson(pairs))
}
