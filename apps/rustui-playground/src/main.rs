#![cfg_attr(target_family = "wasm", no_main)]

#[cfg(not(target_family = "wasm"))]
fn main() {
    rustui_playground::run();
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    rustui_playground::run();
}
