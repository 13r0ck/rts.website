// mod types;
mod pages;

use pages::PasswdGen;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[wasm_bindgen(start)]
pub fn run_app() {
    App::<PasswdGen>::new().mount_to_body();
}
