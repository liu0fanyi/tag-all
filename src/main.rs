//! Tag-All Frontend Entry Point

mod models;
mod commands;
mod tree;
mod context;
mod store;
mod components;
mod app;
mod markdown;

use app::App;
use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
