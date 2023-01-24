use abstutil::Timer;
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
    obj.into_raw()
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_org_osm2streets_StreetNetwork_getRoadSurface(
    env: JNIEnv,
    j_self: JObject,
) -> jobject {
    let inner_pointer = env.get_field(j_self, "pointer", "J").unwrap();
    let streets = &mut *(inner_pointer.j().unwrap() as *mut StreetNetwork);

    let feature_collection = streets.inner.to_road_surface();

    // Cache the JNI stuff for performance.
    let c_ArrayList = env.find_class("java/util/ArrayList").unwrap();
    let c_LatLon = env.find_class("org/osm2streets/LatLon").unwrap();
    let m_LatLon_init = env.get_method_id(c_LatLon, "<init>", "(DD)V").unwrap();
    let m_ArrayList_init = env.get_method_id(c_ArrayList, "<init>", "()V").unwrap();
    let m_ArrayList_add = env
        .get_method_id(c_ArrayList, "add", "(Ljava/lang/Object;)Z")
        .unwrap();
    let t_Void = jni::signature::ReturnType::Primitive(jni::signature::Primitive::Void);

    let areas = env
        .new_object_unchecked(c_ArrayList, m_ArrayList_init, &[])
        .unwrap();
    for feature in feature_collection.features {
        let area_points = env
            .new_object_unchecked(c_ArrayList, m_ArrayList_init, &[])
            .unwrap();
        env.call_method_unchecked(
            areas,
            m_ArrayList_add,
            t_Void.clone(),
            &[JValue::Object(area_points).to_jni()],
        )
        .unwrap();

        if let geojson::Value::Polygon(polygon) = feature.geometry.unwrap().value {
            for point in &polygon[0] {
                let ll = env
                    .new_object_unchecked(
                        c_LatLon,
                        m_LatLon_init,
                        &[point[1].into(), point[0].into()],
                    )
                    .unwrap();
                env.call_method_unchecked(
                    area_points,
                    m_ArrayList_add,
                    t_Void.clone(),
                    &[JValue::Object(ll).to_jni()],
                )
                .unwrap();
            }
        }
    }
    areas.into_raw()
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
    output.into_raw()
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
    output.into_raw()
}
