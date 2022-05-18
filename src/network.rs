use crate::{Intersection, RoadWay};
use petgraph::algo::{dijkstra, min_spanning_tree};
use petgraph::data::FromElements;
use petgraph::graph::{EdgeIndex, NodeIndex};
use petgraph::graphmap::DiGraphMap;
use petgraph::stable_graph::StableDiGraph;
use petgraph::visit::Visitable;
use petgraph::Direction;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

pub type IId = NodeIndex;
pub type RId = EdgeIndex;

pub struct RoadNetwork {
    graph: StableDiGraph<Intersection, RoadWay>,
    // intersections: BTreeMap <IId, Intersection>,
    // roads: BTreeMap<RId, RoadWay>,
}

impl RoadNetwork {
    fn new() -> Self {
        Self {
            graph: StableDiGraph::new(),
            // intersections: BTreeMap::new(),
            // roads: BTreeMap::new(),
        }
    }

    fn add_intersection(&mut self, i: Intersection) -> IId {
        self.graph.add_node(i)
    }
    fn add_road(&mut self, r: RoadWay, from: IId, to: IId) -> RId {
        self.graph.add_edge(from, to, r)
    }
}

#[test]
fn it_works() {
    let mut g = RoadNetwork::new();
    let a = g.add_intersection(Intersection::default());
    let b = g.add_intersection(Intersection::default());
    let x = g.add_road(RoadWay::residential(), a, b);
    let xr = g.add_road(RoadWay::residential(), b, a);
    dbg!(g.graph.neighbors(a));
}
