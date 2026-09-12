pub mod application;
pub mod domain;
#[cfg(feature = "ssr")]
pub mod infrastructure;
pub mod recommendation;
pub mod web;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::web::app::*;
    leptos::mount::hydrate_body(App);
}
