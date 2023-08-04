extern crate leptos;
use components::home::*;
use leptos::*;

pub mod components {
    pub mod home;
}
pub mod hooks {
    pub mod use_websocket;
}

fn main() {
    console_error_panic_hook::set_once();
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    mount_to_body(|cx| view! {cx, <Home/>})
}
