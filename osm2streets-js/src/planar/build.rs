use std::collections::{BTreeMap, HashSet};

use geom::{Circle, Distance, GPSBounds, Line, PolyLine, Polygon};

use osm2streets::StreetNetwork;

use super::{hashify, unhashify, HashedPoint, PlanarGraph};

pub fn streets_to_planar(streets: &StreetNetwork) -> PlanarGraph {
    let mut input = Vec::new();

    if false {
        // Road and intersection geometry as input
        for road in streets.roads.values() {
            input.push((
                format!("{}", road.id),
                polygon_to_lines(&road.center_line.make_polygons(road.total_width())),
            ));
        }
        for i in streets.intersections.values() {
            input.push((format!("{}", i.id), polygon_to_lines(&i.polygon)));
        }
    } else {
        // Just road center lines
        for road in streets.roads.values() {
            input.push((format!("{}", road.id), road.reference_line.clone()));
        }
    }

    input.push((
        "boundary".to_string(),
        polygon_to_lines(&streets.boundary_polygon),
    ));

    PlanarGraph::new(input)
}

fn polygon_to_lines(polygon: &Polygon) -> PolyLine {
    PolyLine::unchecked_new(polygon.get_outer_ring().clone().into_points())
}

impl PlanarGraph {
    pub fn new(input: Vec<(String, PolyLine)>) -> Self {
        let line_segments: Vec<(String, Line)> = explode_lines(input);

        let mut graph = Self {
            edges: BTreeMap::new(),
            nodes: BTreeMap::new(),
        };
        for (_, line) in &line_segments {
            // This'll repeatedly overwrite nodes
            graph.add_node(line.pt1());
            graph.add_node(line.pt2());
        }
        for (source, line) in line_segments {
            graph.add_edge(line.to_polyline(), source);
        }

        graph
    }

    pub fn render_network(&self, gps_bounds: &GPSBounds) -> String {
        let mut pairs = Vec::new();

        // Just show nodes and edges, to start
        for (id, edge) in &self.edges {
            let pl = &edge.geometry;
            let mut props = serde_json::Map::new();
            props.insert("stroke".to_string(), true.into());
            props.insert("weight".to_string(), 5.into());
            props.insert("color".to_string(), "cyan".into());
            props.insert("opacity".to_string(), 0.9.into());
            props.insert("id".to_string(), format!("{:?}", id).into());
            props.insert("sources".to_string(), format!("{:?}", edge.sources).into());

            // To emphasize edges that aren't split properly, trim lines from both ends a bit
            // TODO Not super helpful
            let buffer = Distance::meters(2.0);
            if pl.length() > buffer * 3.0 {
                pairs.push((
                    pl.exact_slice(buffer, pl.length() - buffer)
                        .to_geojson(Some(gps_bounds)),
                    props,
                ));
            } else {
                pairs.push((pl.to_geojson(Some(gps_bounds)), props));
            }
        }

        let close = self.find_close_nodes();
        for pt in self.nodes.keys() {
            let mut props = serde_json::Map::new();
            props.insert("fill".to_string(), true.into());
            if close.contains(pt) {
                props.insert("fillColor".to_string(), "red".into());
            } else {
                props.insert("fillColor".to_string(), "green".into());
            }
            props.insert("fillOpacity".to_string(), 0.9.into());
            props.insert("id".to_string(), format!("{:?}", pt).into());
            pairs.push((
                Circle::new(unhashify(*pt), Distance::meters(1.0))
                    .to_polygon()
                    .to_geojson(Some(gps_bounds)),
                props,
            ));
        }

        abstutil::to_json(&geom::geometries_with_properties_to_geojson(pairs))
    }

    fn find_close_nodes(&self) -> HashSet<HashedPoint> {
        let mut close = HashSet::new();
        for pt1 in self.nodes.keys() {
            for pt2 in self.nodes.keys() {
                if pt1 != pt2 && unhashify(*pt1).dist_to(unhashify(*pt2)) < Distance::meters(0.5) {
                    close.insert(*pt1);
                    close.insert(*pt2);
                }
            }
        }
        close
    }
}

fn explode_lines(input: Vec<(String, PolyLine)>) -> Vec<(String, Line)> {
    // First explode the input into line segments
    // TODO Rewrite as a geo operation!
    let mut line_segments: Vec<(String, Line)> = Vec::new();
    info!("{} input rings", input.len());
    for (name, pl) in &input {
        for pair in pl.points().windows(2) {
            if let Ok(line) = Line::new(pair[0], pair[1]) {
                line_segments.push((name.clone(), line));
            }
        }
    }
    info!("becomes {} line segments", line_segments.len());

    // Then find every line that intersects another, and split the line as needed
    // index of a line -> all points to split at
    let mut hits: BTreeMap<usize, HashSet<HashedPoint>> = BTreeMap::new();
    for (idx1, (name1, line1)) in line_segments.iter().enumerate() {
        for (idx2, (name2, line2)) in line_segments.iter().enumerate() {
            if name1 == name2 {
                continue;
            }
            if let Some(pt) = line1.intersection(line2) {
                hits.entry(idx1)
                    .or_insert_with(HashSet::new)
                    .insert(hashify(pt));
                hits.entry(idx2)
                    .or_insert_with(HashSet::new)
                    .insert(hashify(pt));
            }
        }
    }
    info!("{} lines need to be split somewhere", hits.len());

    // TODO Very messy, expensive way of doing it, but avoids index mess
    let mut output = Vec::new();
    for (idx, (name, orig_line)) in line_segments.into_iter().enumerate() {
        if let Some(split_pts) = hits.remove(&idx) {
            let mut points = vec![orig_line.pt1(), orig_line.pt2()];
            for pt in split_pts {
                points.push(unhashify(pt));
            }

            // TODO Shouldn't need to, but
            points.retain(|pt| orig_line.dist_along_of_point(*pt).is_some());
            points.sort_by_key(|pt| orig_line.dist_along_of_point(*pt).unwrap());

            for pair in points.windows(2) {
                if let Ok(line) = Line::new(pair[0], pair[1]) {
                    output.push((name.clone(), line));
                }
            }
        } else {
            output.push((name, orig_line));
        }
    }
    output
}
