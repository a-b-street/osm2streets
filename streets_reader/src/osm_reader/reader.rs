use std::collections::{BTreeMap, HashMap};
use std::iter::Peekable;

use anyhow::Result;
use xmlparser::Token;

use abstutil::{prettyprint_usize, Tags, Timer};
use geom::{GPSBounds, LonLat};
use osm2streets::osm::{NodeID, OsmID, RelationID, WayID};

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
