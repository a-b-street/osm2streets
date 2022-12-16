use std::collections::{HashSet, BTreeMap};

use abstutil::Counter;
use geom::{Circle, Distance, PolyLine, Pt2D, GPSBounds, Ring, Polygon};

use osm2streets::StreetNetwork;

struct PlanarGraph {
    edges: BTreeMap<EdgeID, PolyLine>,
    nodes: BTreeMap<(isize, isize), Node>,
}

type EdgeID = usize;

struct Node {
    // Sorted clockwise
    edges: Vec<EdgeID>,

    // NOTE the direction here is relative to pointing AT this thing
    // if we can do the lookup... probably just by edge and side?
    oriented_edges: Vec<OrientedEdge>,
}

impl Node {
    fn next_edge(&self, this_node: Pt2D, oriented_edge: OrientedEdge, graph: &PlanarGraph) -> OrientedEdge {
        let idx = self.oriented_edges.iter().position(|x| *x == oriented_edge).unwrap() as isize;
        // ALWAYS go counter-clockwise. Easy.
        let mut next = abstutil::wraparound_get(&self.oriented_edges, idx - 1).clone();
        // Always flip the direction. We just arrived at this node, now we're going away.
        next.direction = next.direction.opposite();
        next
    }
}

impl PlanarGraph {
    fn from_rings(mut input: Vec<(String, Ring)>) -> Self {
        let mut graph = Self {
            edges: BTreeMap::new(),
            nodes: BTreeMap::new(),
        };

        // Similar to split_ways logic
        let mut counts_per_pt = Counter::new();
        for (_, ring) in &input {
            // The first/last point is arbitrary and will always be a node. That lets us actually
            // make a face for every original ring passed in.
            for pt in ring.points() {
                let hash_pt = hashify(*pt);
                let count = counts_per_pt.inc(hash_pt);

                // TODO Can't figure out why we're missing nodes
                //if count == 2 && !graph.nodes.contains_key(&hash_pt) {
                if !graph.nodes.contains_key(&hash_pt) {
                    graph.nodes.insert(hash_pt, Node {
                        edges: Vec::new(),
                        oriented_edges: Vec::new(),
                    });
                }
            }
        }

        for (name, ring) in input {
            let mut pts = Vec::new();
            for pt in ring.into_points() {
                pts.push(pt);
                if pts.len() == 1 {
                    continue;
                }
                if graph.nodes.contains_key(&hashify(pt)) {
                    graph.add_edge(PolyLine::must_new(pts));
                }

                pts = vec![pt];
            }
            assert_eq!(pts.len(), 1);
        }

        graph
    }

    fn add_edge(&mut self, pl: PolyLine) {
        let id = self.edges.len();

        let endpts = [pl.first_pt(), pl.last_pt()];

        self.edges.insert(id, pl);
        for endpt in endpts {
            let node = self.nodes.get_mut(&hashify(endpt)).unwrap();
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

            // and then calculate the oriented edges, trusting the above
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
                if self.edges[e].first_pt() == endpt {
                    left.direction = Direction::Backwards;
                    right.direction = Direction::Backwards;
                    node.oriented_edges.push(left);
                    node.oriented_edges.push(right);
                } else {
                    node.oriented_edges.push(right);
                    node.oriented_edges.push(left);
                }
            }
        }
    }

    fn render_network(&self, gps_bounds: &GPSBounds) -> String {
        let mut pairs = Vec::new();

        // Just show nodes and edges, to start
        for (_, pl) in &self.edges {
            let mut props = serde_json::Map::new();
            props.insert("stroke".to_string(), true.into());
            props.insert("color".to_string(), "cyan".into());
            pairs.push((pl.to_geojson(Some(gps_bounds)), props));
        }

        for pt in self.nodes.keys() {
            let mut props = serde_json::Map::new();
            props.insert("fill".to_string(), true.into());
            props.insert("fillColor".to_string(), "red".into());
            pairs.push((Circle::new(unhashify(*pt), Distance::meters(1.0)).to_polygon().to_geojson(Some(gps_bounds)), props));
        }

        abstutil::to_json(&geom::geometries_with_properties_to_geojson(pairs))
    }

    fn to_faces(&self) -> Vec<Face> {
        let mut faces = Vec::new();
        let mut seen: HashSet<(EdgeID, Side)> = HashSet::new();

        //faces.extend(self.trace_face(EdgeID::Road(RoadID(4)), Side::Right, Direction::Forwards));

        /*
        // Initial direction depends on the orientation of the edge! We MUST go clockwise.
        let ls: geo::LineString = (&self.edges[&start_edge]).into();
        // TODO Handle no winding order
        let start_direction = if ls.is_cw() {
            // The order is funny here because...
            Direction::Forwards
        } else {
            Direction::Backwards
        };
        */

        for e in self.edges.keys() {
            for side in [Side::Left, Side::Right] {
                if seen.contains(&(*e, side)) {
                    continue;
                }

                for dir in [Direction::Forwards, Direction::Backwards] {
                    if let Some(face) = self.trace_face(*e, side, dir) {
                        for member in &face.members {
                            seen.insert((member.edge, member.side));
                        }
                        
                        faces.push(face);
                    }
                }
            }
        }
        faces
    }

    fn trace_face(&self, start_edge: EdgeID, start_side: Side, start_direction: Direction) -> Option<Face> {
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
            /*if members.len() > 10 {
                break;
            }*/

            members.push(current.clone());
            if current == start && !pts.is_empty() {
                pts.push(pts[0]);
                break;
            }
            pts.extend(current.to_points(self));

            let endpt = current.last_pt(self);
            current = self.nodes[&hashify(endpt)].next_edge(endpt, current, self);
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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
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
impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Forwards => Self::Backwards,
            Self::Backwards => Self::Forwards,
        }
    }
}

fn streets_to_planar(streets: &StreetNetwork) -> PlanarGraph {
    let mut input = Vec::new();
    for road in streets.roads.values() {
        /*input.push(road.center_line.must_shift_left(road.half_width()));
        input.push(road.center_line.must_shift_right(road.half_width()));*/
        // Literally pass in rings, lol
        input.push((format!("{}", road.id), road.center_line.make_polygons(road.total_width()).into_outer_ring()));
    }
    for i in streets.intersections.values() {
        input.push((format!("{}", i.id), i.polygon.clone().into_outer_ring()));
    }

    PlanarGraph::from_rings(input)
}

pub fn to_geojson(streets: &StreetNetwork) -> String {
    let graph = streets_to_planar(streets);

    graph.render_network(&streets.gps_bounds)

    /*let mut pairs = Vec::new();
    for face in graph.to_faces() {
        let mut props = serde_json::Map::new();
        props.insert("fill".to_string(), true.into());
        props.insert("fillColor".to_string(), "cyan".into());
        props.insert("fillOpacity".to_string(), 0.5.into());
        props.insert("id".to_string(), pairs.len().into());
        pairs.push((face.polygon.to_geojson(Some(&streets.gps_bounds)), props));
    }
    abstutil::to_json(&geom::geometries_with_properties_to_geojson(pairs))*/
}

fn hashify(pt: Pt2D) -> (isize, isize) {
    let x = (pt.x() * 100.0) as isize;
    let y = (pt.y() * 100.0) as isize;
    (x, y)
}

fn unhashify(pt: (isize, isize)) -> Pt2D {
    let x = pt.0 as f64 / 100.0;
    let y = pt.1 as f64 / 100.0;
    Pt2D::new(x, y)
}
