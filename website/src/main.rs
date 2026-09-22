#![allow(non_snake_case)]

use leptos::prelude::*;

mod app;
mod bindgen;
mod components;
mod markdown;
mod models;
mod pages;
mod router;
mod types;
mod utils;

fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    wasm_logger::init(wasm_logger::Config::default());

    mount_to_body(app::App);
}
