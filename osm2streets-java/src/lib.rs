use abstutil::Timer;
use ejni::{Class, List, Object};
use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys::{jlong, jobject, jstring};
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
        let (mut network, _) =
            streets_reader::osm_to_street_network(&osm_xml_input, clip_pts, cfg, &mut timer)
                .unwrap();
        let transformations = Transformation::standard_for_clipped_areas();
        network.apply_transformations(transformations, &mut timer);

        Self { inner: network }
    }
}

#[no_mangle]
pub extern "system" fn Java_org_osm2streets_StreetNetwork_create(
    env: JNIEnv,
    _: JClass,
    osm_xml_input: JString,
) -> jobject {
    let osm_xml_input: String = env.get_string(osm_xml_input).unwrap().into();
    let network = StreetNetwork::new(osm_xml_input);

    let pointer = Box::into_raw(Box::new(network)) as jlong;
    let obj_class = env.find_class("org/osm2streets/StreetNetwork").unwrap();
    let obj = env
        .new_object(obj_class, "(J)V", &[JValue::Long(pointer)])
        .unwrap();
    obj.into_inner()
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_org_osm2streets_StreetNetwork_getRoadSurface(
    env: JNIEnv,
    java_pointer: JObject,
) -> jobject {
    // TODO ditch ejni, use jni instead
    let c_LatLon = Class::for_name(&env, "org/osm2streets/LatLon").unwrap();
    let c_Object = Class::Object(&env).unwrap();

    let inner_pointer = env.get_field(java_pointer, "pointer", "J").unwrap();
    let streets = &mut *(inner_pointer.j().unwrap() as *mut StreetNetwork);

    let geojson = streets.inner.to_road_surface();

    // let areas = List::arraylist(&env, c_Object.clone()).unwrap();
    for feature in geojson.features {
        let area_points = List::arraylist(&env, c_LatLon.clone()).unwrap();

        match feature.geometry.unwrap().value {
            geojson::Value::Polygon(polygon) => {
                for point in &polygon[0] {
                    let ll = Object::new(
                        &env,
                        // TODO cache the typechecking steps of new_object outside the loop
                        env.new_object(
                            c_LatLon.clone(),
                            "(DD)V",
                            &[JValue::Double(point[0]), JValue::Double(point[1])],
                        )
                        .unwrap(),
                        c_Object.clone(),
                    );
                    area_points.add(&ll).unwrap();
                }
                return area_points.into();
                // areas.add(&area_points.inner).unwrap();
            }
            _ => {}
        }
    }
    // FIXME this seems to return null when called from Java:
    areas.inner.inner.into_inner()
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_osm2streets_StreetNetwork_toLanePolygonsGeojson(
    env: JNIEnv,
    java_pointer: JObject,
) -> jstring {
    let inner_pointer = env.get_field(java_pointer, "pointer", "J").unwrap();
    let streets = &mut *(inner_pointer.j().unwrap() as *mut StreetNetwork);

    let result = streets.inner.to_lane_polygons_geojson().unwrap();
    let output = env.new_string(result).unwrap();
    output.into_inner()
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_osm2streets_StreetNetwork_toLaneMarkingsGeojson(
    env: JNIEnv,
    java_pointer: JObject,
) -> jstring {
    let inner_pointer = env.get_field(java_pointer, "pointer", "J").unwrap();
    let streets = &mut *(inner_pointer.j().unwrap() as *mut StreetNetwork);

    let result = streets.inner.to_lane_markings_geojson().unwrap();
    let output = env.new_string(result).unwrap();
    output.into_inner()
}
