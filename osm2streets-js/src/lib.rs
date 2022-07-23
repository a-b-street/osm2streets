use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// TODO Placeholder

#[derive(Serialize, Deserialize)]
pub struct Input {
    drive_on_right: bool,
}

#[wasm_bindgen]
pub fn import_osm(val: &JsValue) -> String {
    set_panic_hook();

    let input: Input = val.into_serde().unwrap();

    "placeholder".to_string()
}

fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
