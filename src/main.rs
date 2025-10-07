mod app;
mod bytes_line_stream;
mod ollama;

use app::*;
use leptos::{logging, mount};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn main() {
    console_error_panic_hook::set_once();
    logging::log!("csr mode - mounting to body");
    mount::mount_to_body(App);
}
