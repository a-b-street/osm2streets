use abstutil::Timer;
use jni::objects::{JClass, JString};
use jni::sys::{jlong, jstring};
use jni::JNIEnv;

use osm2streets::{MapConfig, Transformation};

struct StreetNetwork {
    inner: osm2streets::StreetNetwork,
}

impl StreetNetwork {
    fn new(osm_xml_input: String) -> Self {
        let cfg = MapConfig::default();

        let clip_pts = None;
        let mut timer = Timer::throwaway();
        let mut network =
            streets_reader::osm_to_street_network(&osm_xml_input, clip_pts, cfg, &mut timer)
                .unwrap();
        let transformations = Transformation::standard_for_clipped_areas();
        network.apply_transformations(transformations, &mut timer);

        Self { inner: network }
    }
}

#[no_mangle]
pub extern "system" fn Java_StreetNetwork_create(
    env: JNIEnv,
    _: JClass,
    osm_xml_input: JString,
) -> jlong {
    let osm_xml_input: String = env.get_string(osm_xml_input).unwrap().into();
    let network = StreetNetwork::new(osm_xml_input);
    Box::into_raw(Box::new(network)) as jlong
}

#[no_mangle]
pub unsafe extern "system" fn Java_StreetNetwork_toGeojsonPlain(
    env: JNIEnv,
    _: JClass,
    pointer: jlong,
) -> jstring {
    let streets = &mut *(pointer as *mut StreetNetwork);
    let result = streets.inner.to_geojson().unwrap();
    let output = env.new_string(result).unwrap();
    output.into_raw()
}

#[no_mangle]
pub unsafe extern "system" fn Java_StreetNetwork_toLanePolygonsGeojson(
    env: JNIEnv,
    _: JClass,
    pointer: jlong,
) -> jstring {
    let streets = &mut *(pointer as *mut StreetNetwork);
    let result = streets.inner.to_lane_polygons_geojson().unwrap();
    let output = env.new_string(result).unwrap();
    output.into_raw()
}

#[no_mangle]
pub unsafe extern "system" fn Java_StreetNetwork_toLaneMarkingsGeojson(
    env: JNIEnv,
    _: JClass,
    pointer: jlong,
) -> jstring {
    let streets = &mut *(pointer as *mut StreetNetwork);
    let result = streets.inner.to_lane_markings_geojson().unwrap();
    let output = env.new_string(result).unwrap();
    output.into_raw()
}
