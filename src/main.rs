#[cfg(not(target_arch = "wasm32"))]
fn load_dotenv() {
    // Load variables from .env if present; ignore errors in prod.
    let _ = dotenvy::dotenv();
}

#[cfg(target_arch = "wasm32")]
fn load_dotenv() {}

fn main() {
    load_dotenv();
    dioxus::launch(blackbird::ui::App);
}
