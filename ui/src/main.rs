mod app;
mod components;
mod routes;
mod store;
mod tauri;
mod types;

use wasm_bindgen::JsCast;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to(
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("main")
            .unwrap()
            .unchecked_into(),
        app::App,
    ).forget();
}
