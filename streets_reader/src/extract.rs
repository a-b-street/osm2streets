use std::collections::{HashMap, HashSet};

use osm::{NodeID, OsmID, RelationID, WayID};

use abstutil::Tags;
use geom::{HashablePt2D, Pt2D};
use osm2streets::{osm, CrossingType, Direction, RestrictionType};

use crate::osm_reader::{Node, Relation, Way};
use crate::MapConfig;

pub struct OsmExtract {
    /// Unsplit roads. These aren't Roads yet, because they may not obey those invariants.
    pub roads: Vec<(WayID, Vec<Pt2D>, Tags)>,
    /// Traffic signals to the direction they apply
    pub traffic_signals: HashMap<HashablePt2D, Direction>,
    pub osm_node_ids: HashMap<HashablePt2D, NodeID>,
    /// (ID, restriction type, from way ID, via node ID, to way ID)
    pub simple_turn_restrictions: Vec<(RestrictionType, WayID, NodeID, WayID)>,
    /// (relation ID, from way ID, via way ID, to way ID)
    pub complicated_turn_restrictions: Vec<(RelationID, WayID, WayID, WayID)>,
    /// Crossings located at these points, which should be on a Road's center line
    pub crossing_nodes: HashSet<(HashablePt2D, CrossingType)>,
    /// Some kind of barrier nodes at these points. Only the ones on a Road center line are
    /// relevant.
    pub barrier_nodes: HashSet<HashablePt2D>,
}

impl OsmExtract {
    pub fn new() -> Self {
        Self {
            roads: Vec::new(),
            traffic_signals: HashMap::new(),
            osm_node_ids: HashMap::new(),
            simple_turn_restrictions: Vec::new(),
            complicated_turn_restrictions: Vec::new(),
            crossing_nodes: HashSet::new(),
            barrier_nodes: HashSet::new(),
        }
    }

    pub fn handle_node(&mut self, id: NodeID, node: &Node) {
        self.osm_node_ids.insert(node.pt.to_hashable(), id);

        if node.tags.is(osm::HIGHWAY, "traffic_signals") {
            let dir = if node.tags.is("traffic_signals:direction", "backward") {
                Direction::Back
            } else {
                Direction::Fwd
            };
            self.traffic_signals.insert(node.pt.to_hashable(), dir);
        }
        if node.tags.is(osm::HIGHWAY, "crossing") {
            // TODO Look for crossing:signals:* too.
            // https://wiki.openstreetmap.org/wiki/Tag:crossing=traffic%20signals?uselang=en
            let kind = if node.tags.is("crossing", "traffic_signals") {
                CrossingType::Signalized
            } else {
                CrossingType::Unsignalized
            };
            self.crossing_nodes.insert((node.pt.to_hashable(), kind));
        }
        // TODO Any kind of barrier?
        if node.tags.is("barrier", "bollard") {
            self.barrier_nodes.insert(node.pt.to_hashable());
        }
    }

    // Returns true if the way was added as a road
    pub fn handle_way(&mut self, id: WayID, way: &Way, cfg: &MapConfig) -> bool {
        let tags = &way.tags;

        if tags.is("area", "yes") {
            return false;
        }

        // First deal with railways.
        if tags.is("railway", "light_rail") {
            self.roads.push((id, way.pts.clone(), tags.clone()));
            return true;
        }
        if tags.is("railway", "rail") && cfg.include_railroads {
            self.roads.push((id, way.pts.clone(), tags.clone()));
            return true;
        }

        let highway = if let Some(x) = tags.get(osm::HIGHWAY) {
            if x == "construction" {
                // What exactly is under construction?
                if let Some(x) = tags.get("construction") {
                    x
                } else {
                    return false;
                }
            } else {
                x
            }
        } else {
            return false;
        };

        if !vec![
            "cycleway",
            "footway",
            "living_street",
            "motorway",
            "motorway_link",
            "path",
            "pedestrian",
            "primary",
            "primary_link",
            "residential",
            "secondary",
            "secondary_link",
            "service",
            "steps",
            "tertiary",
            "tertiary_link",
            "track",
            "trunk",
            "trunk_link",
            "unclassified",
        ]
        .contains(&highway.as_ref())
        {
            return false;
        }

        // If we're only handling sidewalks tagged on roads, skip crossings and separate sidewalks
        // Note we have to do this here -- get_lane_specs_ltr doesn't support decisions like
        // "actually, let's pretend this road doesn't exist at all"
        if cfg.inferred_sidewalks {
            if tags.is(osm::HIGHWAY, "footway")
                && tags.is_any("footway", vec!["crossing", "sidewalk"])
            {
                return false;
            }
        }

        // Import most service roads. Always ignore driveways, golf cart paths, and always reserve
        // parking_aisles for parking lots.
        if highway == "service" && tags.is_any("service", vec!["driveway", "parking_aisle"]) {
            // An exception -- keep driveways signed for bikes
            if !(tags.is("service", "driveway") && tags.is("bicycle", "designated")) {
                return false;
            }
        }
        if highway == "service" && tags.is("golf", "cartpath") {
            return false;
        }
        if highway == "service" && tags.is("access", "customers") {
            return false;
        }

        self.roads.push((id, way.pts.clone(), tags.clone()));
        true
    }

    // Returns true if the relation was used (turn restrictions only)
    pub fn handle_relation(&mut self, id: RelationID, rel: &Relation) -> bool {
        if !rel.tags.is("type", "restriction") {
            return false;
        }

        let mut from_way_id: Option<WayID> = None;
        let mut via_node_id: Option<NodeID> = None;
        let mut via_way_id: Option<WayID> = None;
        let mut to_way_id: Option<WayID> = None;
        for (role, member) in &rel.members {
            match member {
                OsmID::Way(w) => {
                    if role == "from" {
                        from_way_id = Some(*w);
                    } else if role == "to" {
                        to_way_id = Some(*w);
                    } else if role == "via" {
                        via_way_id = Some(*w);
                    }
                }
                OsmID::Node(n) => {
                    if role == "via" {
                        via_node_id = Some(*n);
                    }
                }
                OsmID::Relation(r) => {
                    warn!("{} contains {} as {}", id, r, role);
                }
            }
        }
        if let Some(restriction) = rel.tags.get("restriction") {
            if let Some(rt) = RestrictionType::new(restriction) {
                if let (Some(from), Some(via), Some(to)) = (from_way_id, via_node_id, to_way_id) {
                    self.simple_turn_restrictions.push((rt, from, via, to));
                } else if let (Some(from), Some(via), Some(to)) =
                    (from_way_id, via_way_id, to_way_id)
                {
                    if rt == RestrictionType::BanTurns {
                        self.complicated_turn_restrictions.push((id, from, via, to));
                    } else {
                        warn!(
                            "Weird complicated turn restriction \"{}\" from {} to {} via {}: \
                             {}",
                            restriction, from, to, via, id
                        );
                    }
                }
            }
        }

        true
    }
}
