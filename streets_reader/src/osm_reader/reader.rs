use std::collections::{BTreeMap, HashMap};

use abstutil::{Tags, Timer};
use anyhow::Result;
use geom::{GPSBounds, LonLat};
use osm_reader::{Element, OsmID};

use osm2streets::utils::prettyprint_usize;

use super::{Document, Node, Relation, Way};

// References to missing objects are just filtered out.
// Per https://wiki.openstreetmap.org/wiki/OSM_XML#Certainties_and_Uncertainties, we assume
// elements come in order: nodes, ways, then relations. We assume ways reference nodes and
// relations reference members before defining their own tags.
//
// TODO Filter out visible=false
// TODO NodeID, WayID, RelationID are nice. Plumb forward through map_model.
// TODO Replicate IDs in each object, and change members to just hold a reference to the object
// (which is guaranteed to exist).
impl Document {
    /// Parses xml or pbf bytes and extracts all objects
    pub fn read(
        input_bytes: &[u8],
        gps_bounds: Option<GPSBounds>,
        timer: &mut Timer,
    ) -> Result<Self> {
        let mut doc = Self {
            gps_bounds,
            nodes: BTreeMap::new(),
            ways: BTreeMap::new(),
            relations: BTreeMap::new(),
            clipped_copied_ways: Vec::new(),
        };

        timer.start("scrape objects");
        osm_reader::parse(input_bytes, |elem| match elem {
            Element::Timestamp(_) => {}
            Element::Bounds {
                min_lon,
                min_lat,
                max_lon,
                max_lat,
            } => {
                // If we weren't provided with GPSBounds, use this.
                if doc.gps_bounds.is_none() {
                    doc.gps_bounds = Some(GPSBounds::from(vec![
                        LonLat::new(min_lon, min_lat),
                        LonLat::new(max_lon, max_lat),
                    ]));
                }
            }
            Element::Node {
                id,
                lon,
                lat,
                tags,
                version,
            } => {
                if doc.gps_bounds.is_none() {
                    warn!(
                        "No clipping polygon provided and the .osm is missing a <bounds> element, \
                         so figuring out the bounds manually."
                    );
                    doc.gps_bounds = Some(scrape_bounds(input_bytes).unwrap());
                }

                if doc.nodes.contains_key(&id) {
                    // TODO Make osm_reader API take fallible callbacks
                    panic!("Duplicate {id}, your .osm is corrupt");
                }
                let pt = LonLat::new(lon, lat).to_pt(doc.gps_bounds.as_ref().unwrap());
                doc.nodes.insert(
                    id,
                    Node {
                        pt,
                        tags: make_tags(tags),
                        version,
                    },
                );
            }
            Element::Way {
                id,
                node_ids,
                tags,
                version,
            } => {
                if doc.ways.contains_key(&id) {
                    panic!("Duplicate {id}, your .osm is corrupt");
                }
                let mut pts = Vec::new();
                let mut nodes = Vec::new();
                for n in node_ids {
                    // Just skip missing nodes
                    if let Some(node) = doc.nodes.get(&n) {
                        nodes.push(n);
                        pts.push(node.pt);
                    }
                }

                if !nodes.is_empty() {
                    doc.ways.insert(
                        id,
                        Way {
                            nodes,
                            pts,
                            tags: make_tags(tags),
                            version,
                        },
                    );
                }
            }
            Element::Relation {
                id,
                tags,
                mut members,
                version,
            } => {
                if doc.relations.contains_key(&id) {
                    panic!("Duplicate {id}, your .osm is corrupt");
                }
                // Filter out missing members
                members.retain(|(_, member)| match member {
                    OsmID::Node(n) => doc.nodes.contains_key(n),
                    OsmID::Way(w) => doc.ways.contains_key(w),
                    OsmID::Relation(r) => doc.relations.contains_key(r),
                });

                doc.relations.insert(
                    id,
                    Relation {
                        tags: make_tags(tags),
                        members,
                        version,
                    },
                );
            }
        })?;
        timer.stop("scrape objects");
        info!(
            "Found {} nodes, {} ways, {} relations",
            prettyprint_usize(doc.nodes.len()),
            prettyprint_usize(doc.ways.len()),
            prettyprint_usize(doc.relations.len())
        );

        Ok(doc)
    }
}

fn scrape_bounds(input_bytes: &[u8]) -> Result<GPSBounds> {
    let mut b = GPSBounds::new();
    osm_reader::parse(input_bytes, |elem| match elem {
        Element::Node { lon, lat, .. } => {
            b.update(LonLat::new(lon, lat));
        }
        _ => {}
    })?;
    Ok(b)
}

// Temporary shim to build from hashmap
fn make_tags(tags: HashMap<String, String>) -> Tags {
    Tags::new(tags.into_iter().collect())
}
