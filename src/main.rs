/// Bundled config for mobile builds (iOS/Android)
const BUNDLED_CONFIG: &str = include_str!("../assets/config.env");

#[cfg(not(target_arch = "wasm32"))]
fn load_dotenv() {
    // First try to load from .env file (desktop dev)
    if dotenvy::dotenv().is_ok() {
        return;
    }

    // Fall back to bundled config (mobile builds)
    load_bundled_config();
}

#[cfg(target_arch = "wasm32")]
fn load_dotenv() {
    load_bundled_config();
}

fn load_bundled_config() {
    for line in BUNDLED_CONFIG.lines() {
        let line = line.trim();
        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Parse KEY=VALUE
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            // Only set if not already set (allow env override)
            if std::env::var(key).is_err() {
                // SAFETY: We're setting env vars at startup before any threads are spawned
                unsafe {
                    std::env::set_var(key, value);
                }
            }
        }
    }
}

fn main() {
    load_dotenv();
    dioxus::launch(blackbird::ui::App);
}
