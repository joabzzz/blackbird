use crate::types::ThemeMode;
use crate::views::shared::initial_saved_docs;
use crate::views::{ChatView, DocsView, SettingsView};
use dioxus::prelude::*;
use std::time::Duration;

const MOSTRA_CSS: Asset = asset!("/assets/mostra.css");
const SPLASH_LOGO: Asset = asset!("/assets/blackbird_logo_1024.png");
const SPLASH_TITLE: Asset = asset!("/assets/blackbird-title.png");
const SPLASH_HIDE_DELAY: Duration = Duration::from_secs(3);

#[derive(Clone, Copy, PartialEq, Eq)]
enum AppTab {
    Chat,
    Docs,
    Settings,
}

#[component]
pub fn App() -> Element {
    let saved_docs = use_signal(initial_saved_docs);
    let mut active_tab = use_signal(|| AppTab::Chat);
    let base_font_px = use_signal(|| 14i32);
    let theme = use_signal(|| ThemeMode::Dark);
    let show_splash = use_signal(|| true);

    use_effect(move || {
        if show_splash() {
            let mut show_splash_signal = show_splash;
            spawn(async move {
                tokio::time::sleep(SPLASH_HIDE_DELAY).await;
                show_splash_signal.set(false);
            });
        }
    });

    let root_style = format!(":root {{ font-size: {}px; }}", base_font_px());
    let current_theme = theme();
    let theme_css = theme_styles(current_theme);
    let header_wordmark_class = if current_theme == ThemeMode::Octane {
        "header-wordmark header-wordmark-octane"
    } else {
        "header-wordmark"
    };

    rsx! {
        document::Link { rel: "stylesheet", href: MOSTRA_CSS }
        style { dangerous_inner_html: "{root_style}" }
        style { dangerous_inner_html: "{theme_css}" }
        if show_splash() {
            SplashScreen {}
        }
        div { class: "header no-divider",
            div { class: "header-content",
                img { class: "{header_wordmark_class}", src: SPLASH_TITLE, alt: "Blackbird" }
                div { class: "tabs",
                    h1 {
                        class: if active_tab() == AppTab::Chat { "tab active" } else { "tab" },
                        onclick: move |_| active_tab.set(AppTab::Chat),
                        "Chat"
                    }
                    h1 {
                        class: if active_tab() == AppTab::Docs { "tab active" } else { "tab" },
                        onclick: move |_| active_tab.set(AppTab::Docs),
                        "Docs"
                    }
                    h1 {
                        class: if active_tab() == AppTab::Settings { "tab active" } else { "tab" },
                        onclick: move |_| active_tab.set(AppTab::Settings),
                        "Settings"
                    }
                }
            }
        }

        div { class: "tab-panels",
            div {
                class: format_args!(
                    "tab-panel {}",
                    if active_tab() == AppTab::Chat { "active" } else { "" }
                ),
                aria_hidden: (active_tab() != AppTab::Chat).to_string(),
                ChatView { saved_docs, base_font_px }
            }
            div {
                class: format_args!(
                    "tab-panel {}",
                    if active_tab() == AppTab::Docs { "active" } else { "" }
                ),
                aria_hidden: (active_tab() != AppTab::Docs).to_string(),
                DocsView { saved_docs }
            }
            div {
                class: format_args!(
                    "tab-panel {}",
                    if active_tab() == AppTab::Settings { "active" } else { "" }
                ),
                aria_hidden: (active_tab() != AppTab::Settings).to_string(),
                SettingsView { theme }
            }
        }
    }
}

#[component]
fn SplashScreen() -> Element {
    rsx! {
        div { class: "splash-overlay", aria_hidden: "true",
            div { class: "splash-content",
                img { class: "splash-logo", src: SPLASH_LOGO, alt: "Blackbird logo" }
                img { class: "splash-title", src: SPLASH_TITLE, alt: "Blackbird wordmark" }
            }
        }
    }
}

fn theme_styles(theme: ThemeMode) -> String {
    match theme {
        ThemeMode::Dark => r#"
:root {
    --color-bg-primary: #000000;
    --color-bg-secondary: #050505;
    --color-bg-overlay: rgba(0, 0, 0, 0.9);
    --color-text-primary: #ffffff;
    --color-text-secondary: #ffffff;
    --color-text-muted: #cfcfcf;
    --color-border: #ffffff;
    --color-surface-muted: #111111;
    --color-input-border: #2a2a2a;
    --color-input-bg: #000000;
    --color-chat-user-bg: #ffffff;
    --color-chat-user-text: #000000;
    --color-chat-assistant-bg: #000000;
    --color-chat-assistant-text: #ffffff;
    --color-doc-card-border: #2a2a2a;
    --color-doc-card-bg: #050505;
    --color-doc-card-hover: #ffffff;
    --color-doc-viewer-bg: rgba(255, 255, 255, 0.02);
    --color-timestamp: #9b9b9b;
    --color-shimmer-base: rgba(255, 53, 9, 0.25);
    --color-shimmer-highlight: #ff3509;
    --color-header-fade: rgba(0, 0, 0, 0.85);
}
body { background: var(--color-bg-primary); color: var(--color-text-primary); }
.header { background: var(--color-bg-primary); }
.btn:hover,
.btn-ghost:hover { background: var(--color-surface-muted); }
.composer textarea { background: var(--color-input-bg); color: var(--color-text-primary); border-color: var(--color-input-border); }
.composer textarea:focus { border-color: var(--color-border); }
.doc-viewer { background: transparent; }
"#.to_string(),
        ThemeMode::Light => r#"
:root {
    --color-bg-primary: #ffffff;
    --color-bg-secondary: #f5f5f5;
    --color-bg-overlay: rgba(255, 255, 255, 0.92);
    --color-text-primary: #000000;
    --color-text-secondary: #000000;
    --color-text-muted: #4a4a4a;
    --color-border: #000000;
    --color-surface-muted: #e6e6e6;
    --color-input-border: #c2c2c2;
    --color-input-bg: #ffffff;
    --color-chat-user-bg: #111111;
    --color-chat-user-text: #ffffff;
    --color-chat-assistant-bg: #ffffff;
    --color-chat-assistant-text: #000000;
    --color-doc-card-border: #d0d0d0;
    --color-doc-card-bg: #f5f5f5;
    --color-doc-card-hover: #000000;
    --color-doc-viewer-bg: rgba(0, 0, 0, 0.04);
    --color-timestamp: #606060;
    --color-shimmer-base: rgba(255, 53, 9, 0.25);
    --color-shimmer-highlight: #ff3509;
    --color-header-fade: rgba(255, 255, 255, 0.9);
}
body { background: var(--color-bg-primary); color: var(--color-text-primary); }
.header { background: var(--color-bg-primary); }
.btn { color: var(--color-text-primary); }
.btn:hover,
.btn-ghost:hover { background: var(--color-surface-muted); }
.composer { background: var(--color-bg-overlay); border-top-color: var(--color-border); }
.composer textarea { background: var(--color-input-bg); color: var(--color-text-primary); border-color: var(--color-input-border); }
.composer textarea:focus { border-color: var(--color-border); }
.doc-viewer { background: var(--color-doc-viewer-bg); }
"#.to_string(),
        ThemeMode::Octane => r#"
:root {
    --color-bg-primary: #ff3509;
    --color-bg-secondary: #ffe0d1;
    --color-bg-overlay: rgba(0, 0, 0, 0.55);
    --color-text-primary: #000000;
    --color-text-secondary: #111111;
    --color-text-muted: #2c2c2c;
    --color-border: #000000;
    --color-surface-muted: rgba(0, 0, 0, 0.12);
    --color-input-border: rgba(0, 0, 0, 0.6);
    --color-input-bg: #ffe8de;
    --color-chat-user-bg: #000000;
    --color-chat-user-text: #ffffff;
    --color-chat-assistant-bg: rgba(255, 255, 255, 0.92);
    --color-chat-assistant-text: #121212;
    --color-doc-card-border: rgba(0, 0, 0, 0.5);
    --color-doc-card-bg: #ffb79f;
    --color-doc-card-hover: #000000;
    --color-doc-viewer-bg: rgba(255, 240, 234, 0.96);
    --color-timestamp: rgba(0, 0, 0, 0.75);
    --color-shimmer-base: rgba(255, 255, 255, 0.3);
    --color-shimmer-highlight: #ffffff;
    --color-accent-primary: #ffffff;
    --color-header-fade: rgba(255, 53, 9, 0.92);
}
body { background: var(--color-bg-primary); color: var(--color-text-primary); }
.header { background: transparent; }
.btn { color: var(--color-text-primary); }
.btn:hover,
.btn-ghost:hover { background: var(--color-surface-muted); }
.btn-primary { border-color: var(--color-accent-primary); color: #000; }
.btn-primary:hover { background: var(--color-accent-primary); color: #000; }
.composer { background: var(--color-bg-overlay); border-top-color: var(--color-border); }
.composer textarea { background: var(--color-input-bg); color: var(--color-text-primary); border-color: var(--color-input-border); }
.composer textarea:focus { border-color: var(--color-border); }
.doc-viewer { background: var(--color-doc-viewer-bg); }
"#.to_string(),
    }
}
