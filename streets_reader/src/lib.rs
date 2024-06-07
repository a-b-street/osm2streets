#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

use abstutil::Timer;
use anyhow::Result;
use geom::{GPSBounds, LonLat, Ring};

use osm2streets::{DrivingSide, MapConfig, StreetNetwork};
use osm_reader::Document;

pub use self::extract::OsmExtract;

// TODO Clean up the public API of all of this
pub mod extract;
pub mod osm_reader;
pub mod split_ways;

/// Create a `StreetNetwork` from the contents of an `.osm.xml` or `.pbf` file. If `clip_pts` is
/// specified, use these as a boundary polygon. (Use `LonLat::read_geojson_polygon` or similar to
/// produce these.)
///
/// You probably want to do `StreetNetwork::apply_transformations` on the result to get a useful
/// result.
pub fn osm_to_street_network(
    input_bytes: &[u8],
    clip_pts: Option<Vec<LonLat>>,
    cfg: MapConfig,
    timer: &mut Timer,
) -> Result<(StreetNetwork, Document)> {
    let mut streets = StreetNetwork::blank();
    // Note that DrivingSide is still incorrect. It'll be set in extract_osm, before Road::new
    // happens in split_ways.
    streets.config = cfg;

    let (extract, doc) = extract_osm(&mut streets, input_bytes, clip_pts, timer)?;
    split_ways::split_up_roads(&mut streets, extract, timer);

    // Cul-de-sacs aren't supported yet.
    streets.retain_roads(|r| r.src_i != r.dst_i);

    Ok((streets, doc))
}

/// Set up country code and driving side, using an arbitrary point. This must be called after
/// `gps_bounds` is set.
pub fn detect_country_code(streets: &mut StreetNetwork) {
    if let Some(dir) = streets.config.override_driving_side {
        info!("Ignoring country for driving side; using override {dir:?}");
        streets.config.driving_side = dir;
        return;
    }

    let geocoder = country_geocoder::CountryGeocoder::new();
    let pt = streets.gps_bounds.get_rectangle()[0].into();

    if let (Some(code), Some(left_handed)) = (geocoder.iso_a2(pt), geocoder.drives_on_left(pt)) {
        streets.config.driving_side = if left_handed {
            DrivingSide::Left
        } else {
            DrivingSide::Right
        };
        streets.config.country_code = code.to_string();
    } else {
        error!("detect_country_code failed -- {:?} didn't match to any country. Driving side may be wrong!", pt);
    }
}

fn extract_osm(
    streets: &mut StreetNetwork,
    input_bytes: &[u8],
    clip_pts: Option<Vec<LonLat>>,
    timer: &mut Timer,
) -> Result<(OsmExtract, Document)> {
    let mut doc = Document::read(
        input_bytes,
        clip_pts.as_ref().map(|pts| GPSBounds::from(pts.clone())),
        timer,
    )?;
    // If GPSBounds aren't provided, they'll be computed in the Document
    streets.gps_bounds = doc.gps_bounds.clone().unwrap();

    if let Some(pts) = clip_pts {
        streets.boundary_polygon =
            Ring::deduping_new(streets.gps_bounds.convert(&pts))?.into_polygon();
        doc.clip(&streets.boundary_polygon, timer);
    } else {
        streets.boundary_polygon = streets.gps_bounds.to_bounds().get_rectangle();
        // No need to clip the Document in this case.
    }

    detect_country_code(streets);

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
    timer.start_iter(
        "processing OSM ways split into pieces",
        doc.clipped_copied_ways.len(),
    );
    for (id, way) in &doc.clipped_copied_ways {
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
