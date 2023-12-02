use std::collections::{BTreeMap, HashMap};
use std::io::Cursor;
use std::iter::Peekable;

use abstutil::{Tags, Timer};
use anyhow::Result;
use geom::{GPSBounds, LonLat};
use osmpbf::{BlobDecode, BlobReader, Element as PbfElement, IndexedReader, RelMemberType};
use xmlparser::Token;

use osm2streets::osm::{NodeID, OsmID, RelationID, WayID};
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
    /// Parses raw pbf from buffer and extracts all objects
    pub fn read_pbf(
        input: &[u8],
        gps_bounds: Option<GPSBounds>,
        timer: &mut Timer,
    ) -> Result<Self, anyhow::Error> {
        let mut doc = Self {
            gps_bounds,
            nodes: BTreeMap::new(),
            ways: BTreeMap::new(),
            relations: BTreeMap::new(),
            clipped_copied_ways: Vec::new(),
        };

        let mut blob_reader = BlobReader::new(Cursor::new(input));
        // par_map_reduce would be faster, but would not allow us to read the bbox + header
        timer.start("scrape objects");
        while let Some(Ok(blob)) = blob_reader.next() {
            match blob.decode().unwrap() {
                BlobDecode::OsmHeader(head) => {
                    // if we find a bounding box
                    // in the blob header, use it.
                    if let Some(bbox) = head.bbox() {
                        doc.gps_bounds = Some(GPSBounds {
                            min_lon: bbox.left,
                            min_lat: bbox.top,
                            max_lon: bbox.right,
                            max_lat: bbox.bottom,
                        });
                    } else if doc.gps_bounds.is_none() {
                        doc.gps_bounds = Option::from(
                            scrape_bounds_pbf(
                                IndexedReader::new(
                                    Cursor::new(
                                        input)).unwrap()))
                    }
                }
                BlobDecode::OsmData(block) => {
                    block.elements().for_each(|element| {
                        match element {
                            PbfElement::Node(node) => {
                                let pt = LonLat::new(node.lon(), node.lat()).to_pt(&doc.gps_bounds.clone().unwrap());
                                let mut tags = Tags::new(BTreeMap::new());
                                for (k, v) in node.tags() {
                                    tags.insert(k, v);
                                }
                                doc.nodes.insert(NodeID(node.id()), Node { pt, tags });
                            }
                            PbfElement::DenseNode(node) => {
                                let pt = LonLat::new(node.lon(), node.lat()).to_pt(&doc.gps_bounds.clone().unwrap());

                                let mut tags = Tags::new(BTreeMap::new());
                                for (k, v) in node.tags() {
                                    tags.insert(k, v);
                                }

                                doc.nodes.insert(NodeID(node.id()), Node { pt, tags });
                            }
                            PbfElement::Way(way) => {
                                let mut tags = Tags::new(BTreeMap::new());
                                for (k, v) in way.tags() {
                                    tags.insert(k, v);
                                }

                                let mut nodes = Vec::new();
                                let mut pts = Vec::new();
                                for nd in way.refs() {
                                    let n = NodeID(nd);
                                    // Just skip missing nodes
                                    if let Some(node) = doc.nodes.get(&n) {
                                        nodes.push(n);
                                        pts.push(node.pt);
                                    }
                                }
                                let version = way
                                    .info()
                                    .version()
                                    .map(|x| x as usize);

                                if !nodes.is_empty() {
                                    doc.ways.insert(
                                        WayID(way.id()),
                                        Way {
                                            nodes,
                                            pts,
                                            tags,
                                            version,
                                        },
                                    );
                                }
                            }
                            PbfElement::Relation(relation) => {
                                let mut tags = Tags::new(BTreeMap::new());
                                for (k, v) in relation.tags() {
                                    tags.insert(k, v);
                                }
                                let id = RelationID(relation.id());
                                if doc.relations.contains_key(&id) {
                                    error!("Duplicate IDs detected. Your PBF is corrupt.");
                                    return
                                }
                                    let mut members = Vec::new();
                                for member in relation.members() {
                                    let osm_id = match member.member_type {
                                        RelMemberType::Node => {
                                            let n = NodeID(member.member_id);
                                            if !doc.nodes.contains_key(&n) {
                                                continue;
                                            }
                                            OsmID::Node(n)
                                        }
                                        RelMemberType::Way => {
                                            let w = WayID(member.member_id);
                                            if !doc.ways.contains_key(&w) {
                                                continue;
                                            }
                                            OsmID::Way(w)
                                        }
                                        RelMemberType::Relation => {
                                            let r = RelationID(member.member_id);
                                            if !doc.relations.contains_key(&r) {
                                                continue;
                                            }
                                            OsmID::Relation(r)
                                        }
                                    };
                                    members.push((member.role().unwrap().to_string(), osm_id));
                                }

                                doc.relations.insert(id, Relation { tags, members });
                            }
                        }
                    });
                }
                // Just skip unrecognizable data.
                BlobDecode::Unknown(_) => {}
            }
        }
        timer.stop("scrape objects");
        info!(
            "Found {} nodes, {} ways, {} relations",
            prettyprint_usize(doc.nodes.len()),
            prettyprint_usize(doc.ways.len()),
            prettyprint_usize(doc.relations.len())
        );
        Ok(doc)
    }

    /// Parses raw OSM XML and extracts all objects.
    pub fn read(
        raw_string: &str,
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

        // We use the lower-level xmlparser instead of roxmltree to reduce peak memory usage in
        // large files.
        let mut reader = ElementReader {
            tokenizer: xmlparser::Tokenizer::from(raw_string),
        }
            .peekable();

        timer.start("scrape objects");
        while let Some(obj) = reader.next() {
            match obj.name {
                "bounds" => {
                    // If we weren't provided with GPSBounds, use this.
                    if doc.gps_bounds.is_some() {
                        continue;
                    }
                    doc.gps_bounds = Some(GPSBounds::from(vec![
                        LonLat::new(
                            obj.attribute("minlon").parse::<f64>().unwrap(),
                            obj.attribute("minlat").parse::<f64>().unwrap(),
                        ),
                        LonLat::new(
                            obj.attribute("maxlon").parse::<f64>().unwrap(),
                            obj.attribute("maxlat").parse::<f64>().unwrap(),
                        ),
                    ]));
                }
                "node" => {
                    if doc.gps_bounds.is_none() {
                        warn!(
                            "No clipping polygon provided and the .osm is missing a <bounds> element, \
                             so figuring out the bounds manually."
                        );
                        doc.gps_bounds = Some(scrape_bounds(raw_string));
                    }

                    let id = NodeID(obj.attribute("id").parse::<i64>().unwrap());
                    if doc.nodes.contains_key(&id) {
                        bail!("Duplicate {}, your .osm is corrupt", id);
                    }
                    let pt = LonLat::new(
                        obj.attribute("lon").parse::<f64>().unwrap(),
                        obj.attribute("lat").parse::<f64>().unwrap(),
                    )
                        .to_pt(doc.gps_bounds.as_ref().unwrap());
                    let tags = read_tags(&mut reader);
                    doc.nodes.insert(id, Node { pt, tags });
                }
                "way" => {
                    let id = WayID(obj.attribute("id").parse::<i64>().unwrap());
                    if doc.ways.contains_key(&id) {
                        bail!("Duplicate {}, your .osm is corrupt", id);
                    }
                    let version = obj
                        .attributes
                        .get("version")
                        .and_then(|x| x.parse::<usize>().ok());

                    let mut nodes = Vec::new();
                    let mut pts = Vec::new();
                    while reader.peek().map(|x| x.name == "nd").unwrap_or(false) {
                        let node_ref = reader.next().unwrap();
                        let n = NodeID(node_ref.attribute("ref").parse::<i64>().unwrap());
                        // Just skip missing nodes
                        if let Some(node) = doc.nodes.get(&n) {
                            nodes.push(n);
                            pts.push(node.pt);
                        }
                    }

                    // We assume <nd>'s come before <tag>'s
                    let tags = read_tags(&mut reader);
                    if !nodes.is_empty() {
                        doc.ways.insert(
                            id,
                            Way {
                                nodes,
                                pts,
                                tags,
                                version,
                            },
                        );
                    }
                }
                "relation" => {
                    let id = RelationID(obj.attribute("id").parse::<i64>().unwrap());
                    if doc.relations.contains_key(&id) {
                        bail!("Duplicate {}, your .osm is corrupt", id);
                    }
                    let mut members = Vec::new();
                    while reader.peek().map(|x| x.name == "member").unwrap_or(false) {
                        let child = reader.next().unwrap();
                        let member = match child.attribute("type") {
                            "node" => {
                                let n = NodeID(child.attribute("ref").parse::<i64>().unwrap());
                                if !doc.nodes.contains_key(&n) {
                                    continue;
                                }
                                OsmID::Node(n)
                            }
                            "way" => {
                                let w = WayID(child.attribute("ref").parse::<i64>().unwrap());
                                if !doc.ways.contains_key(&w) {
                                    continue;
                                }
                                OsmID::Way(w)
                            }
                            "relation" => {
                                let r = RelationID(child.attribute("ref").parse::<i64>().unwrap());
                                if !doc.relations.contains_key(&r) {
                                    continue;
                                }
                                OsmID::Relation(r)
                            }
                            _ => continue,
                        };
                        members.push((child.attribute("role").to_string(), member));
                    }

                    // We assume <nd>'s come before <tag>'s
                    let tags = read_tags(&mut reader);

                    doc.relations.insert(id, Relation { tags, members });
                }
                _ => {}
            }
        }
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

fn read_tags(reader: &mut Peekable<ElementReader>) -> Tags {
    let mut tags = Tags::empty();

    while reader.peek().map(|x| x.name == "tag").unwrap_or(false) {
        let obj = reader.next().unwrap();
        let key = obj.attribute("k");
        let value = obj.attribute("v");
        tags.insert(key, unescape(value).unwrap());
    }

    tags
}

fn scrape_bounds(raw_string: &str) -> GPSBounds {
    let mut b = GPSBounds::new();
    for obj in (ElementReader {
        tokenizer: xmlparser::Tokenizer::from(raw_string),
    }) {
        if obj.name == "node" {
            b.update(LonLat::new(
                obj.attribute("lon").parse::<f64>().unwrap(),
                obj.attribute("lat").parse::<f64>().unwrap(),
            ));
        }
    }
    b
}

fn scrape_bounds_pbf(mut reader: IndexedReader<Cursor<&[u8]>>) -> GPSBounds {
    let mut b = GPSBounds::new();
    reader.for_each_node(|el| {
        match el {
            PbfElement::Node(node) => {
                b.update(LonLat::new(
                    node.lon(),
                    node.lat(),
                ));
            }
            PbfElement::DenseNode(node) => {
                b.update(LonLat::new(
                    node.lon(),
                    node.lat(),
                ));
            }
            _ => {}
        }
    }).expect("Failed to scrape bounds from nodes.");
    b
}

// Reads one element with attributes at a time. Ignores/flattens nested elements.
struct ElementReader<'a> {
    tokenizer: xmlparser::Tokenizer<'a>,
}

struct Element<'a> {
    name: &'a str,
    attributes: HashMap<&'a str, &'a str>,
}

impl<'a> Element<'a> {
    fn attribute(&self, key: &str) -> &str {
        self.attributes.get(key).unwrap()
    }
}

impl<'a> Iterator for ElementReader<'a> {
    type Item = Element<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut name: Option<&'a str> = None;
        let mut attributes = HashMap::new();
        loop {
            match self.tokenizer.next()?.unwrap() {
                Token::ElementStart { local, .. } => {
                    assert!(name.is_none());
                    assert!(attributes.is_empty());
                    name = Some(local.as_str());
                }
                Token::Attribute { local, value, .. } => {
                    assert!(name.is_some());
                    attributes.insert(local.as_str(), value.as_str());
                }
                Token::ElementEnd { .. } => {
                    if name.is_none() {
                        assert!(attributes.is_empty());
                        continue;
                    }

                    return Some(Element {
                        name: name.unwrap(),
                        attributes,
                    });
                }
                _ => {}
            }
        }
    }
}

// Copied from https://github.com/Florob/RustyXML, Apache licensed. Unescapes all valid XML
// entities in a string.
fn unescape(input: &str) -> Result<String> {
    let mut result = String::with_capacity(input.len());

    let mut it = input.split('&');

    // Push everything before the first '&'
    if let Some(sub) = it.next() {
        result.push_str(sub);
    }

    for sub in it {
        match sub.find(';') {
            Some(idx) => {
                let ent = &sub[..idx];
                match ent {
                    "quot" => result.push('"'),
                    "apos" => result.push('\''),
                    "gt" => result.push('>'),
                    "lt" => result.push('<'),
                    "amp" => result.push('&'),
                    ent => {
                        let val = if ent.starts_with("#x") {
                            u32::from_str_radix(&ent[2..], 16).ok()
                        } else if ent.starts_with('#') {
                            u32::from_str_radix(&ent[1..], 10).ok()
                        } else {
                            None
                        };
                        match val.and_then(char::from_u32) {
                            Some(c) => result.push(c),
                            None => bail!("&{};", ent),
                        }
                    }
                }
                result.push_str(&sub[idx + 1..]);
            }
            None => bail!("&".to_owned() + sub),
        }
    }
    Ok(result)
}
