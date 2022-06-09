use gloo_utils::document;
use leaflet::{GeoJSON, LatLng, Map, MouseEvent, Path, Polyline, TileLayer};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, Node};
use yew::prelude::*;
use yew::Html;

use crate::Msg as WebMsg;

#[allow(clippy::large_enum_variant)]
pub enum Msg {
    MapClick(LatLng),
    Error(String),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point(pub f64, pub f64);

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub callback_msg: Callback<WebMsg>,
}

pub struct MapComponent {
    container: HtmlElement,
    map: Map,
    point: Point,
    path: Option<Path>,
    _map_click_closure: Closure<dyn Fn(MouseEvent)>,
}

impl MapComponent {
    fn render_map(&self) -> Html {
        let node: &Node = &self.container.clone().into();
        Html::VRef(node.clone())
    }
}

impl Component for MapComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let container: Element = document().create_element("div").unwrap();
        let container: HtmlElement = container.dyn_into().unwrap();
        container.set_id("map");
        let map = Map::new_with_element(&container, &JsValue::NULL);

        let map_click_callback = ctx.link().callback(Msg::MapClick);
        let map_click_closure = Closure::<dyn Fn(MouseEvent)>::wrap(Box::new(move |click_event| {
            log::debug!("map click");
            map_click_callback.emit(click_event.latlng());
        }));
        map.on("click", map_click_closure.as_ref());

        Self {
            container,
            map,
            point: Point(40.0, 10.0),
            path: None,
            // to avoid dropping the closure and invalidating the callback
            _map_click_closure: map_click_closure,
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            self.map
                .setView(&LatLng::new(self.point.0, self.point.1), 4.0);
            log::debug!("add tile layer");
            add_tile_layer(&self.map);
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::MapClick(lat_lng) => {
                // ctx.link().send_future(async move {
                //     match get_nearby((lat_lng.lat(), lat_lng.lng())).await {
                //         Ok((id, tags, geometry, locale)) => {
                //             Msg::MapUpdate(id.to_string(), tags, locale, geometry)
                //         },
                //         Err(e) => Msg::Error(e.to_string()),
                //     }
                // });
                true
            }
            // Msg::MapUpdate() => {
            //     if let Some(path) = self.path.take() {
            //         path.remove();
            //     }
            //
            //     let polyline = Polyline::new(
            //         geometry
            //             .into_iter()
            //             .map(|lat_lon| LatLng::new(lat_lon.lat, lat_lon.lon).into())
            //             .collect(),
            //     );
            //     let bounds = polyline.getBounds();
            //     let path = Path::from(polyline);
            //     path.addTo(&self.map);
            //     self.path = Some(path);
            //     self.map.fitBounds(&bounds);
            //     ctx.props()
            //         .callback_msg
            //         .emit(WebMsg::TagsLocaleSet { id, tags, locale });
            //     true
            // },
            Msg::Error(_) => todo!(),
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! { self.render_map() }
    }
}

fn add_tile_layer(map: &Map) {
    TileLayer::new(
        "https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png",
        &JsValue::NULL,
    )
    .addTo(map);
}

// fn add_osm_layer(map: &Map, xml: &Reader) {
//     OSM.DataLayer::new(xml)
//     .addTo(map);
// }
