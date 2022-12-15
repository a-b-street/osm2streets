#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

use abstutil::Timer;
use anyhow::Result;
use geom::{GPSBounds, LonLat, Ring};

use osm2streets::{DrivingSide, MapConfig, StreetNetwork};

pub use self::extract::OsmExtract;
use osm_reader::Document;

// TODO Clean up the public API of all of this
pub mod extract;
pub mod osm_reader;
pub mod split_ways;

/// Create a `StreetNetwork` from the contents of an `.osm.xml` file. If `clip_pts` is specified,
/// use theese as a boundary polygon. (Use `LonLat::read_geojson_polygon` or similar to produce
/// these.)
///
/// You probably want to do `StreetNetwork::apply_transformations` on the result to get a useful
/// result.
pub fn osm_to_street_network(
    osm_xml_input: &str,
    clip_pts: Option<Vec<LonLat>>,
    cfg: MapConfig,
    timer: &mut Timer,
) -> Result<(StreetNetwork, Document)> {
    let mut streets = StreetNetwork::blank();
    // Note that DrivingSide is still incorrect. It'll be set in extract_osm, before Road::new
    // happens in split_ways.
    streets.config = cfg;

    let (extract, doc) = extract_osm(&mut streets, osm_xml_input, clip_pts, timer)?;
    split_ways::split_up_roads(&mut streets, extract, timer);

    // Cul-de-sacs aren't supported yet.
    streets.retain_roads(|r| r.src_i != r.dst_i);

    Ok((streets, doc))
}

fn extract_osm(
    streets: &mut StreetNetwork,
    osm_xml_input: &str,
    clip_pts: Option<Vec<LonLat>>,
    timer: &mut Timer,
) -> Result<(OsmExtract, Document)> {
    let mut doc = Document::read(
        osm_xml_input,
        clip_pts.as_ref().map(|pts| GPSBounds::from(pts.clone())),
        timer,
    )?;
    // If GPSBounds aren't provided above, they'll be computed in the Document
    streets.gps_bounds = doc.gps_bounds.clone().unwrap();

    if let Some(pts) = clip_pts {
        streets.boundary_polygon = Ring::new(streets.gps_bounds.convert(&pts))?.into_polygon();
        doc.clip(&streets.boundary_polygon);
    } else {
        streets.boundary_polygon = streets.gps_bounds.to_bounds().get_rectangle();
        // No need to clip the Document in this case.
    }

    // Calculate DrivingSide from some arbitrary point
    streets.config.driving_side =
        if driving_side::is_left_handed(streets.gps_bounds.get_rectangle()[0].into()) {
            DrivingSide::Left
        } else {
            DrivingSide::Right
        };

    let mut out = OsmExtract::new();

    timer.start_iter("processing OSM nodes", doc.nodes.len());
    for (id, node) in &doc.nodes {
        timer.next();
        out.handle_node(*id, node);
    }

    timer.start_iter("processing OSM ways", doc.ways.len());
    for (id, way) in &doc.ways {
        timer.next();
        out.handle_way(*id, way, &streets.config);
    }

    timer.start_iter("processing OSM relations", doc.relations.len());
    for (id, rel) in &doc.relations {
        timer.next();
        out.handle_relation(*id, rel);
    }

    Ok((out, doc))
}
