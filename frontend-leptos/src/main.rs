//! CoSave Leptos WebAssembly Client Entrypoint.
//!
//! Initializes the panic hook and mounts the root `App` component to the DOM.

#![deny(dead_code)]
#![deny(unused)]

mod api;
mod app;
mod components;
mod health;

use app::App;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
