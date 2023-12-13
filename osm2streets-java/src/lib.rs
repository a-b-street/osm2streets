use abstutil::Timer;
use jni::objects::{JClass, JObject, JValue};
use jni::sys::{jlong, jobject};
use jni::JNIEnv;

use osm2streets::{MapConfig, Transformation};

struct StreetNetwork {
    inner: osm2streets::StreetNetwork,
}

impl StreetNetwork {
    fn new(input_bytes: &[u8]) -> Self {
        let cfg = MapConfig::default();

        let clip_pts = None;
        let mut timer = Timer::throwaway();
        let (mut network, _) =
            streets_reader::osm_to_street_network(input_bytes, clip_pts, cfg, &mut timer).unwrap();
        let transformations = Transformation::standard_for_clipped_areas();
        network.apply_transformations(transformations, &mut timer);

        Self { inner: network }
    }
}

#[no_mangle]
pub extern "system" fn Java_org_osm2streets_StreetNetwork_create(
    env: JNIEnv,
    _: JClass,
    input_bytes: jni::sys::jbyteArray,
) -> jobject {
    let input_bytes: Vec<u8> = env.convert_byte_array(input_bytes).unwrap();
    let network = StreetNetwork::new(&input_bytes);

    let pointer = Box::into_raw(Box::new(network)) as jlong;
    let obj_class = env.find_class("org/osm2streets/StreetNetwork").unwrap();
    let obj = env
        .new_object(obj_class, "(J)V", &[JValue::Long(pointer)])
        .unwrap();
    obj.into_raw()
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_org_osm2streets_StreetNetwork_getSurfaces(
    env: JNIEnv,
    j_self: JObject,
) -> jobject {
    // Calculate the road surface.
    let inner_pointer = env.get_field(j_self, "pointer", "J").unwrap();
    let streets = &mut *(inner_pointer.j().unwrap() as *mut StreetNetwork);
    let surfaces = streets.inner.calculate_surfaces();

    // Cache the JNI stuff for performance.
    let c_ArrayList = env.find_class("java/util/ArrayList").unwrap();
    let c_LatLon = env.find_class("org/osm2streets/LatLon").unwrap();
    let c_Surface = env.find_class("org/osm2streets/Surface").unwrap();
    let m_LatLon_init = env.get_method_id(c_LatLon, "<init>", "(DD)V").unwrap();
    let m_Surface_init = env
        .get_method_id(c_Surface, "<init>", "(Ljava/util/List;Ljava/lang/String;)V")
        .unwrap();
    let m_ArrayList_init = env.get_method_id(c_ArrayList, "<init>", "()V").unwrap();
    let m_ArrayList_add = env
        .get_method_id(c_ArrayList, "add", "(Ljava/lang/Object;)Z")
        .unwrap();
    let t_Void = jni::signature::ReturnType::Primitive(jni::signature::Primitive::Void);

    // Marshal the `Surface`s into java objects.
    let j_surfaces = env
        .new_object_unchecked(c_ArrayList, m_ArrayList_init, &[])
        .unwrap();
    for surface in surfaces {
        let j_area_points = env
            .new_object_unchecked(c_ArrayList, m_ArrayList_init, &[])
            .unwrap();
        for point in surface.area.exterior() {
            let ll = env
                .new_object_unchecked(c_LatLon, m_LatLon_init, &[point.y.into(), point.x.into()])
                .unwrap();
            env.call_method_unchecked(
                j_area_points,
                m_ArrayList_add,
                t_Void.clone(),
                &[JValue::Object(ll).to_jni()],
            )
            .unwrap();
        }

        let j_surface = env
            .new_object_unchecked(
                c_Surface,
                m_Surface_init,
                &[
                    JValue::Object(j_area_points),
                    env.new_string(surface.material.to_str()).unwrap().into(),
                ],
            )
            .unwrap();

        env.call_method_unchecked(
            j_surfaces,
            m_ArrayList_add,
            t_Void.clone(),
            &[JValue::Object(j_surface).to_jni()],
        )
        .unwrap();
    }
    j_surfaces.into_raw()
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_org_osm2streets_StreetNetwork_getPaintAreas(
    env: JNIEnv,
    j_self: JObject,
) -> jobject {
    // Calculate the paint areas.
    let inner_pointer = env.get_field(j_self, "pointer", "J").unwrap();
    let streets = &mut *(inner_pointer.j().unwrap() as *mut StreetNetwork);
    let paint_areas = streets.inner.calculate_paint_areas();

    // Cache the JNI stuff for performance.
    let c_ArrayList = env.find_class("java/util/ArrayList").unwrap();
    let c_LatLon = env.find_class("org/osm2streets/LatLon").unwrap();
    let c_PaintArea = env.find_class("org/osm2streets/PaintArea").unwrap();
    let m_LatLon_init = env.get_method_id(c_LatLon, "<init>", "(DD)V").unwrap();
    let m_PaintArea_init = env
        .get_method_id(
            c_PaintArea,
            "<init>",
            "(Ljava/util/List;Ljava/lang/String;)V",
        )
        .unwrap();
    let m_ArrayList_init = env.get_method_id(c_ArrayList, "<init>", "()V").unwrap();
    let m_ArrayList_add = env
        .get_method_id(c_ArrayList, "add", "(Ljava/lang/Object;)Z")
        .unwrap();
    let t_Void = jni::signature::ReturnType::Primitive(jni::signature::Primitive::Void);

    // Marshal the `Surface`s into java objects.
    let j_paint_areas = env
        .new_object_unchecked(c_ArrayList, m_ArrayList_init, &[])
        .unwrap();
    for paint_area in paint_areas {
        let j_area_points = env
            .new_object_unchecked(c_ArrayList, m_ArrayList_init, &[])
            .unwrap();
        for point in paint_area.area.exterior() {
            let ll = env
                .new_object_unchecked(c_LatLon, m_LatLon_init, &[point.y.into(), point.x.into()])
                .unwrap();
            env.call_method_unchecked(
                j_area_points,
                m_ArrayList_add,
                t_Void.clone(),
                &[JValue::Object(ll).to_jni()],
            )
            .unwrap();
        }

        let j_paint_area = env
            .new_object_unchecked(
                c_PaintArea,
                m_PaintArea_init,
                &[
                    JValue::Object(j_area_points),
                    env.new_string(paint_area.color.to_str()).unwrap().into(),
                ],
            )
            .unwrap();

        env.call_method_unchecked(
            j_paint_areas,
            m_ArrayList_add,
            t_Void.clone(),
            &[JValue::Object(j_paint_area).to_jni()],
        )
        .unwrap();
    }
    j_paint_areas.into_raw()
}
