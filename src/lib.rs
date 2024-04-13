use wasm_bindgen::prelude::*;

pub mod gamepad;

use gamepad::{render_universe, CELL_SIZE};

#[wasm_bindgen(start)]
pub fn init() {
    let window = web_sys::window().expect_throw("no window in current context");
    let width = window
        .inner_width()
        .expect_throw("failed to get inner width")
        .as_f64()
        .unwrap_throw() as usize;
    let height = window
        .inner_height()
        .expect_throw("failed to get inner height")
        .as_f64()
        .unwrap_throw() as usize;
    render_universe(width / CELL_SIZE, height / CELL_SIZE, 1);
}
