use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

use web_sys::HtmlInputElement;
use yew::prelude::*;

use streets::units::*;

mod control;
use control::Control;

mod map;
use map::MapComponent;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

type ShouldRender = bool;

#[derive(Debug, PartialEq)]
pub struct State {
    pub driving_side: DrivingSide,
    pub id: Option<String>,
    /// The editable input, line and equal separated tags
    // pub road: Option<RoadNetwork>,
    /// Message for user
    pub message: Option<String>,
    /// Ref to input for way id
    pub way_ref: NodeRef,
}

#[derive(Debug)]
pub enum Msg {
    ToggleDrivingSide,
    WayFetch,
    Error(String),
}

pub struct App {
    state: Rc<RefCell<State>>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let state = Rc::new(RefCell::new(State {
            driving_side: DrivingSide::LHT,
            id: None,
            message: None,
            way_ref: NodeRef::default(),
        }));
        let mut app = Self { state };
        app.update_tags();
        app
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> ShouldRender {
        log::trace!("Message: {:?}", msg);
        match msg {
            Msg::ToggleDrivingSide => {
                {
                    let mut state = self.state.borrow_mut();
                    state.driving_side = state.driving_side.opposite();
                }
                self.update_tags();
                true
            }
            Msg::WayFetch => {
                // let mut state = self.state.borrow_mut();
                // let way_id = state.way_ref.cast::<HtmlInputElement>().unwrap().value();
                // log::debug!("WayFetch {}", way_id);
                // match way_id.parse() {
                //     Ok(way_id) => {
                //         ctx.link().send_future(async move {
                //             match get_way(&way_id).await {
                //                 Ok((tags, _geom, locale)) => Msg::TagsLocaleSet {
                //                     id: way_id.to_string(),
                //                     tags,
                //                     locale,
                //                 },
                //                 Err(e) => Msg::Error(e.to_string()),
                //             }
                //         });
                //     },
                //     Err(e) => state.message = Some(format!("Invalid way id: {}", e)),
                // }
                true
            }
            Msg::Error(e) => {
                let mut state = self.state.borrow_mut();
                state.message = Some(format!("Error: {}", e));
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let state = self.state.borrow();

        let callback_error = ctx.link().callback(Msg::Error);
        let callback_msg = ctx.link().callback(|msg| msg);

        html! {
            <>
                <menu>
                    <h1>{"StreetExplorer"}</h1>
                    <Control callback_msg={callback_msg.clone()} state={self.state.clone()}/>
                {
                    if let Some(message) = &state.message {
                        html!{
                            <aside>{message}</aside>
                        }
                    } else {
                        html!{}
                    }
                }
                </menu>

                <MapComponent callback_msg={callback_msg.clone()}/>
            </>
        }
    }
}

impl App {
    fn update_tags(&mut self) {
        let mut state = self.state.borrow_mut();
        let driving_side = &state.driving_side;

        // state.road = Some(road);
        // if warnings.is_empty() {
        //     state.message = Some(format!("Lanes to Tags Error: {}", error));
        // } else {
        //     state.message =
        //         Some(format!("{}\nLanes to Tags Error: {}", warnings, error));
        // }
    }
}

fn main() {
    console_log::init_with_level(log::Level::Debug).expect("logging failed");
    log::trace!("Initializing yew...");
    yew::start_app::<App>();
}
