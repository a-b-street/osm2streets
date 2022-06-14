use std::cell::RefCell;
use std::rc::Rc;

use streets::units::DrivingSide;
use web_sys::{Event, FocusEvent, HtmlInputElement, HtmlSelectElement, KeyboardEvent, MouseEvent};
use yew::{html, Callback, Component, Context, Html, NodeRef, Properties, TargetCast};

use crate::{Msg, State};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub callback_msg: Callback<Msg>,
    pub state: Rc<RefCell<State>>,
}

#[derive(Default)]
pub struct Control {
    textarea_input_ref: NodeRef,
    textarea_output_ref: NodeRef,
}

impl Component for Control {
    type Properties = Props;
    type Message = Msg;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            ..Default::default()
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        ctx.props().callback_msg.emit(msg);
        // we let the parent do this
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let state = ctx.props().state.borrow();

        let driving_side_onchange = ctx.link().callback(|_e: Event| Msg::ToggleDrivingSide);

        let way_id: String = state
            .id
            .as_ref()
            .cloned()
            .unwrap_or_else(|| String::from(""));
        let way_id_onclick = ctx.link().callback(|_e: MouseEvent| Msg::WayFetch);

        html! {
            <>
                <section class="row">
                    <p class="row-item">
                        {"↑↓ LHT"}
                    </p>
                    <label class="row-item switch">
                        <input
                            type="checkbox"
                            checked={state.driving_side == DrivingSide::RHT}
                            onchange={driving_side_onchange}
                        />
                        <span class="slider"></span>
                    </label>
                    <p class="row-item">
                        {"RHT ↓↑"}
                    </p>


                    <hr/>
                    // <label class="row-item" for="way">{"OSM Way ID"}</label>
                    // <input class="row-item" type="text" id="way" name="way" size="12"
                    //     ref={state.way_ref.clone()}
                    //     value={way_id}/>
                    // <button class="row-item" onclick={way_id_onclick}>
                    //     {"Fetch"}
                    // </button>
                </section>
            </>
        }
    }
}
