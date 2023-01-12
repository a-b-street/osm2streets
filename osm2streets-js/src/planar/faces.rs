use std::collections::{HashMap, HashSet};

use geom::{Polygon, Ring};

use osm2streets::StreetNetwork;

use super::{hashify, Direction, EdgeID, OrientedEdge, PlanarGraph, Side};

pub fn to_geojson_faces(streets: &StreetNetwork) -> String {
    let graph = super::build::streets_to_planar(streets);
    let mut faces = graph.to_faces();

    // TODO Hack to remove the giant polygon that covers everything
    let n = faces.len();
    let area_boundary = streets.boundary_polygon.area();
    faces.retain(|f| f.polygon.area() < area_boundary);
    if faces.len() != n {
        error!("Removed {} gigantic faces", n - faces.len());
    }

    // lol
    let colors = vec!["red", "green", "blue", "cyan", "orange"];
    let color_indices = four_coloring(&faces, colors.len());

    let mut pairs = Vec::new();
    for (face, color_idx) in faces.into_iter().zip(color_indices.into_iter()) {
        let color = if true {
            colors[color_idx]
        } else {
            color_by_classifying(&face)
        };

        let mut props = serde_json::Map::new();
        props.insert("fill".to_string(), true.into());
        props.insert("fillColor".to_string(), color.into());
        props.insert("fillOpacity".to_string(), 0.5.into());
        props.insert("id".to_string(), pairs.len().into());
        props.insert("sources".to_string(), format!("{:?}", face.sources).into());
        pairs.push((face.polygon.to_geojson(Some(&streets.gps_bounds)), props));
    }
    abstutil::to_json(&geom::geometries_with_properties_to_geojson(pairs))
}

fn color_by_classifying(face: &Face) -> &'static str {
    let mut intersections = 0;
    let mut roads = 0;
    let mut boundary = false;
    for x in &face.sources {
        if x == "boundary" {
            boundary = true;
        } else if x.starts_with("Road") {
            roads += 1;
        } else if x.starts_with("Intersection") {
            intersections += 1;
        }
    }

    if intersections == 1 && roads > 0 && !boundary {
        // an intersection
        "black"
    } else if intersections == 2 && roads == 1 && !boundary {
        // a road
        "black"
    } else {
        "cyan"
    }
}

impl PlanarGraph {
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

    fn trace_face(
        &self,
        start_edge: EdgeID,
        start_side: Side,
        start_direction: Direction,
    ) -> Option<Face> {
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
            if let Some(next) = self.nodes[&hashify(endpt)].next_edge(endpt, current.clone(), self)
            {
                current = next;
            } else {
                error!("what happened at {:?}", current);
                return None;
            }
        }

        /*info!(
            "trace_face found {} members, {} pts",
            members.len(),
            pts.len()
        );
        for x in &members {
            info!("  - {:?}", x);
        }*/

        if let Ok(ring) = Ring::deduping_new(pts) {
            let mut sources = HashSet::new();
            for e in &members {
                sources.extend(self.edges[&e.edge].sources.clone());
            }

            Some(Face {
                members,
                polygon: ring.into_polygon(),
                sources,
            })
        } else {
            error!("Traced something, but then bad points");
            None
        }
    }
}

struct Face {
    polygon: Polygon,
    // Clockwise and first=last
    members: Vec<OrientedEdge>,
    sources: HashSet<String>,
}

fn four_coloring(input: &[Face], num_colors: usize) -> Vec<usize> {
    let mut edge_to_faces: HashMap<EdgeID, Vec<usize>> = HashMap::new();
    for (idx, face) in input.iter().enumerate() {
        for id in &face.members {
            edge_to_faces
                .entry(id.edge)
                .or_insert_with(Vec::new)
                .push(idx);
        }
    }

    // Greedily fill out a color for each face, in the same order as the input
    let mut assigned_colors = Vec::new();
    for (this_idx, face) in input.iter().enumerate() {
        let mut available_colors: Vec<bool> = std::iter::repeat(true).take(num_colors).collect();
        // Find all neighbors
        for id in &face.members {
            for other_idx in &edge_to_faces[&id.edge] {
                // We assign colors in order, so any neighbor index smaller than us has been
                // chosen
                if *other_idx < this_idx {
                    available_colors[assigned_colors[*other_idx]] = false;
                }
            }
        }
        if let Some(color) = available_colors.iter().position(|x| *x) {
            assigned_colors.push(color);
        } else {
            // Too few colors?
            return std::iter::repeat(0).take(input.len()).collect();
        }
    }
    assigned_colors
}
