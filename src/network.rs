use crate::road_functions::{Intersection, IntersectionType, RoadWay};
use petgraph::dot::{Config, Dot};
use petgraph::graph::{EdgeIndex, NodeIndex};
use petgraph::stable_graph::StableDiGraph;
use petgraph::Direction;

pub struct RoadNetwork {
    graph: StableDiGraph<Intersection, RoadWay>,
    // intersections: BTreeMap <NodeIndex, Intersection>,
    // roads: BTreeMap<EdgeIndex, RoadWay>,
}

impl RoadNetwork {
    pub fn new() -> Self {
        Self {
            graph: StableDiGraph::new(),
            // intersections: BTreeMap::new(),
            // roads: BTreeMap::new(),
        }
    }

    pub fn add_intersection(&mut self, i: Intersection) -> NodeIndex {
        self.graph.add_node(i)
    }
    pub fn add_closing_roadway(&mut self, r: RoadWay, from: NodeIndex, to: NodeIndex) -> EdgeIndex {
        self.graph.add_edge(from, to, r)
    }
    pub fn add_exploring_roadway(&mut self, r: RoadWay, from: NodeIndex) -> (EdgeIndex, NodeIndex) {
        let to = self.graph.add_node(Intersection::default()); // "Intersection::dangling"
        (self.graph.add_edge(from, to, r), to)
    }
    pub fn add_closing_street(
        &mut self,
        r: RoadWay,
        from: NodeIndex,
        to: NodeIndex,
    ) -> (EdgeIndex, EdgeIndex) {
        let r2 = r.clone();
        (
            self.graph.add_edge(from, to, r),
            self.graph.add_edge(to, from, r2),
        )
    }
    pub fn add_exploring_street(
        &mut self,
        r: RoadWay,
        from: NodeIndex,
    ) -> (EdgeIndex, NodeIndex, EdgeIndex) {
        let to = self.graph.add_node(Intersection::default());
        let (there, back_again) = self.add_closing_street(r, from, to);
        (there, to, back_again)
    }
}

#[test]
fn abbigail_to_school() {
    // Let's represent at a made up road network, showing a short walk from a rural house to school.
    let mut map = RoadNetwork::new();
    let home = map.add_intersection(Intersection::turning_circle());
    let driveway_end = map.add_intersection(Intersection::slice());
    let street_end = map.add_intersection(Intersection::merge());
    let intersection = map.add_intersection(Intersection::intersection());
    let school = map.add_intersection(Intersection::slice());

    map.add_closing_roadway(RoadWay::track(), home, driveway_end);
    map.add_closing_street(RoadWay::rural(), driveway_end, street_end);
    map.add_exploring_street(RoadWay::local(), street_end);
    map.add_closing_street(RoadWay::local(), street_end, intersection);
    map.add_exploring_street(RoadWay::local(), intersection);
    map.add_exploring_street(RoadWay::local(), intersection);
    map.add_closing_street(RoadWay::local(), intersection, school);
    map.add_exploring_street(RoadWay::local(), school);

    // TODO assert some things.

    // Print out dot for graphviz visualisation.
    println!("{}", Dot::new(&map.graph));
}
