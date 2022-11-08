#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

use std::collections::{HashMap, HashSet};

use abstutil::Timer;
use anyhow::Result;
use geom::{GPSBounds, HashablePt2D, LonLat, PolyLine, Ring};

use osm2streets::{CrossingType, DrivingSide, MapConfig, OriginalRoad, StreetNetwork};

pub use self::extract::OsmExtract;

// TODO Clean up the public API of all of this
pub mod clip;
pub mod extract;
pub mod osm_reader;
pub mod split_ways;

/// Configures the creation of a `RawMap` from OSM and other input data.
/// TODO Layering is now strange. Some of these are options are needed just for StreetNetwork, but
/// many are the next level up and just for A/B Street's convert_osm.
pub struct Options {
    pub map_config: MapConfig,

    pub onstreet_parking: OnstreetParking,
    pub public_offstreet_parking: PublicOffstreetParking,
    pub private_offstreet_parking: PrivateOffstreetParking,
    /// OSM railway=rail will be included as light rail if so. Cosmetic only.
    pub include_railroads: bool,
    /// If provided, read polygons from this GeoJSON file and add them to the RawMap as buildings.
    pub extra_buildings: Option<String>,
    /// Only include crosswalks that match a `highway=crossing` OSM node.
    pub filter_crosswalks: bool,
    /// Configure public transit using this URL to a static GTFS feed in .zip format.
    pub gtfs_url: Option<String>,
    pub elevation: bool,
}

impl Options {
    pub fn default() -> Self {
        Self {
            map_config: MapConfig::default(),
            onstreet_parking: OnstreetParking::JustOSM,
            public_offstreet_parking: PublicOffstreetParking::None,
            private_offstreet_parking: PrivateOffstreetParking::FixedPerBldg(1),
            include_railroads: true,
            extra_buildings: None,
            filter_crosswalks: false,
            gtfs_url: None,
            elevation: false,
        }
    }
}

/// What roads will have on-street parking lanes? Data from
/// <https://wiki.openstreetmap.org/wiki/Key:parking:lane> is always used if available.
pub enum OnstreetParking {
    /// If not tagged, there won't be parking.
    JustOSM,
    /// If OSM data is missing, then try to match data from
    /// <http://data-seattlecitygis.opendata.arcgis.com/datasets/blockface>. This is Seattle specific.
    Blockface(String),
    /// If OSM data is missing, then infer parking lanes on some percentage of
    /// "highway=residential" roads.
    SomeAdditionalWhereNoData {
        /// [0, 100]
        pct: usize,
    },
}

/// How many spots are available in public parking garages?
pub enum PublicOffstreetParking {
    None,
    /// Pull data from
    /// <https://data-seattlecitygis.opendata.arcgis.com/datasets/public-garages-or-parking-lots>, a
    /// Seattle-specific data source.
    Gis(String),
}

/// If a building doesn't have anything from public_offstreet_parking and isn't tagged as a garage
/// in OSM, how many private spots should it have?
pub enum PrivateOffstreetParking {
    FixedPerBldg(usize),
    // TODO Based on the number of residents?
}

/// Create a `StreetNetwork` from the contents of an `.osm.xml` file. If `clip_pts` is specified,
/// use theese as a boundary polygon. (Use `LonLat::read_osmosis_polygon` or similar to produce
/// these.)
///
/// You probably want to do `StreetNetwork::apply_transformations` on the result to get a useful
/// result.
pub fn osm_to_street_network(
    osm_xml_input: &str,
    clip_pts: Option<Vec<LonLat>>,
    opts: Options,
    timer: &mut Timer,
) -> Result<StreetNetwork> {
    let mut streets = StreetNetwork::blank();
    // Note that DrivingSide is still incorrect. It'll be set in extract_osm, before Road::new
    // happens in split_ways.
    streets.config = opts.map_config.clone();

    if let Some(ref pts) = clip_pts {
        let gps_bounds = GPSBounds::from(pts.clone());
        streets.boundary_polygon = Ring::new(gps_bounds.convert(pts))?.into_polygon();
        streets.gps_bounds = gps_bounds;
    }

    let extract = extract_osm(&mut streets, osm_xml_input, clip_pts, &opts, timer)?;
    let split_output = split_ways::split_up_roads(&mut streets, extract, timer);
    clip::clip_map(&mut streets, timer)?;

    // Need to do a first pass of removing cul-de-sacs here, or we wind up with loop PolyLines when
    // doing the parking hint matching.
    streets.retain_roads(|r, _| r.i1 != r.i2);

    use_barrier_nodes(
        &mut streets,
        split_output.barrier_nodes,
        &split_output.pt_to_road,
    );
    use_crossing_nodes(
        &mut streets,
        &split_output.crossing_nodes,
        &split_output.pt_to_road,
    );

    if opts.filter_crosswalks {
        filter_crosswalks(
            &mut streets,
            split_output.crossing_nodes,
            split_output.pt_to_road,
            timer,
        );
    }

    Ok(streets)
}

fn extract_osm(
    streets: &mut StreetNetwork,
    osm_xml_input: &str,
    clip_pts: Option<Vec<LonLat>>,
    opts: &Options,
    timer: &mut Timer,
) -> Result<OsmExtract> {
    let doc = crate::osm_reader::read(osm_xml_input, &streets.gps_bounds, timer)?;

    if clip_pts.is_none() {
        // Use the boundary from .osm.
        streets.gps_bounds = doc.gps_bounds.clone();
        streets.boundary_polygon = streets.gps_bounds.to_bounds().get_rectangle();
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
    for (id, node) in doc.nodes {
        timer.next();
        out.handle_node(id, &node);
    }

    timer.start_iter("processing OSM ways", doc.ways.len());
    for (id, way) in doc.ways {
        timer.next();
        out.handle_way(id, &way, opts);
    }

    timer.start_iter("processing OSM relations", doc.relations.len());
    for (id, rel) in doc.relations {
        timer.next();
        out.handle_relation(id, &rel);
    }

    Ok(out)
}

pub fn use_barrier_nodes(
    streets: &mut StreetNetwork,
    barrier_nodes: HashSet<HashablePt2D>,
    pt_to_road: &HashMap<HashablePt2D, OriginalRoad>,
) {
    let mut pt_to_intersection = HashMap::new();
    for (id, i) in &streets.intersections {
        pt_to_intersection.insert(i.point.to_hashable(), *id);
    }

    for pt in barrier_nodes {
        // Many barriers are on footpaths or roads that we don't retain
        if let Some(road) = pt_to_road.get(&pt).and_then(|r| streets.roads.get_mut(r)) {
            // Filters on roads that're already car-free are redundant
            if road.is_driveable() {
                road.barrier_nodes.push(pt.to_pt2d());
            }
        } else if let Some(i) = pt_to_intersection.get(&pt) {
            let roads = &streets.intersections[i].roads;
            if roads.len() == 2 {
                // Arbitrarily put the barrier on one of the roads
                streets
                    .roads
                    .get_mut(&roads[0])
                    .unwrap()
                    .barrier_nodes
                    .push(pt.to_pt2d());
            } else {
                // TODO Look for real examples at non-2-way intersections to understand what to do.
                // If there's a barrier in the middle of a 4-way, does that disconnect all
                // movements?
                warn!(
                    "There's a barrier at {i}, but there are {} roads connected",
                    roads.len()
                );
            }
        }
    }
}

pub fn use_crossing_nodes(
    streets: &mut StreetNetwork,
    crossing_nodes: &HashSet<(HashablePt2D, CrossingType)>,
    pt_to_road: &HashMap<HashablePt2D, OriginalRoad>,
) {
    for (pt, kind) in crossing_nodes {
        // Some crossings are on footpaths or roads that we don't retain
        if let Some(road) = pt_to_road.get(pt).and_then(|r| streets.roads.get_mut(r)) {
            road.crossing_nodes.push((pt.to_pt2d(), *kind));
        }
    }
}

pub fn filter_crosswalks(
    streets: &mut StreetNetwork,
    crosswalks: HashSet<(HashablePt2D, CrossingType)>,
    pt_to_road: HashMap<HashablePt2D, OriginalRoad>,
    timer: &mut Timer,
) {
    // Normally we assume every road has a crosswalk, but since this map is configured to use OSM
    // crossing nodes, let's reverse that assumption.
    for road in streets.roads.values_mut() {
        road.crosswalk_forward = false;
        road.crosswalk_backward = false;
    }

    // Match each crosswalk node to a road
    timer.start_iter("filter crosswalks", crosswalks.len());
    for (pt, _) in crosswalks {
        timer.next();
        // Some crossing nodes are outside the map boundary or otherwise not on a road that we
        // retained
        if let Some(road) = pt_to_road.get(&pt).and_then(|r| streets.roads.get_mut(r)) {
            // TODO Support cul-de-sacs and other loop roads
            if let Ok(pl) = PolyLine::new(road.osm_center_points.clone()) {
                // Crossings aren't right at an intersection. Where is this point along the center
                // line?
                if let Some((dist, _)) = pl.dist_along_of_point(pt.to_pt2d()) {
                    let pct = dist / pl.length();
                    // Don't throw away any crossings. If it occurs in the first half of the road,
                    // snap to the first intersection. If there's a mid-block crossing mapped,
                    // that'll likely not be correctly interpreted, unless an intersection is there
                    // anyway.
                    if pct <= 0.5 {
                        road.crosswalk_backward = true;
                    } else {
                        road.crosswalk_forward = true;
                    }

                    // TODO Some crosswalks incorrectly snap to the intersection near a short
                    // service road, which later gets trimmed. So the crosswalk effectively
                    // disappears.
                }
            }
        }
    }
}
