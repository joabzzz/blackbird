use crate::theme::theme_definition;
use crate::types::ThemeMode;
use crate::views::shared::{SavedDoc, initial_saved_docs};
use crate::views::{ChatView, DocsView, SettingsView};
use dioxus::prelude::*;
use std::time::Duration;

const MOSTRA_CSS: Asset = asset!("/assets/mostra.css");
const SPLASH_LOGO: Asset = asset!("/assets/blackbird_logo_1024.png");
const SPLASH_TITLE: Asset = asset!("/assets/blackbird-title.png");
const SPLASH_HIDE_DELAY: Duration = Duration::from_secs(3);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppTab {
    Chat,
    Docs,
    Settings,
}

#[component]
pub fn App() -> Element {
    let saved_docs = use_signal(initial_saved_docs);
    let active_tab = use_signal(|| AppTab::Chat);
    let base_font_px = use_signal(|| 14i32);
    let theme = use_signal(|| ThemeMode::Dark);
    let show_splash = use_signal(|| true);

    use_splash_dismiss(show_splash);

    rsx! {
        ThemeStyles { base_font_px, theme }
        if show_splash() {
            SplashScreen {}
        }
        AppHeader { active_tab, theme: theme() }
        TabPanels {
            active_tab,
            saved_docs,
            base_font_px,
            theme,
        }
    }
}

fn use_splash_dismiss(show_splash: Signal<bool>) {
    use_effect(move || {
        if show_splash() {
            let mut control = show_splash;
            spawn(async move {
                tokio::time::sleep(SPLASH_HIDE_DELAY).await;
                control.set(false);
            });
        }
    });
}

#[component]
fn ThemeStyles(base_font_px: Signal<i32>, theme: Signal<ThemeMode>) -> Element {
    let root_style = format!(":root {{ font-size: {}px; }}", base_font_px());
    let definition = theme_definition(theme());
    rsx! {
        document::Link { rel: "stylesheet", href: MOSTRA_CSS }
        style { dangerous_inner_html: "{root_style}" }
        style { dangerous_inner_html: "{definition.css}" }
    }
}

#[component]
fn AppHeader(active_tab: Signal<AppTab>, theme: ThemeMode) -> Element {
    let theme = theme_definition(theme);
    rsx! {
        div { class: "header no-divider",
            div { class: "header-content",
                img { class: "{theme.wordmark_class}", src: SPLASH_TITLE, alt: "Blackbird" }
                TabNavigation { active_tab }
            }
        }
    }
}

#[component]
fn TabPanels(
    active_tab: Signal<AppTab>,
    saved_docs: Signal<Vec<SavedDoc>>,
    base_font_px: Signal<i32>,
    theme: Signal<ThemeMode>,
) -> Element {
    rsx! {
        div { class: "tab-panels",
            TabPanel {
                active_tab,
                tab: AppTab::Chat,
                children: rsx!( ChatView { saved_docs, base_font_px } ),
            }
            TabPanel {
                active_tab,
                tab: AppTab::Docs,
                children: rsx!( DocsView { saved_docs } ),
            }
            TabPanel {
                active_tab,
                tab: AppTab::Settings,
                children: rsx!( SettingsView { theme } ),
            }
        }
    }
}

#[component]
fn TabPanel(active_tab: Signal<AppTab>, tab: AppTab, children: Element) -> Element {
    let is_active = active_tab() == tab;
    let class_suffix = if is_active { "active" } else { "" };
    rsx! {
        div {
            class: format_args!("tab-panel {}", class_suffix),
            aria_hidden: (!is_active).to_string(),
            {children}
        }
    }
}

#[component]
fn TabNavigation(active_tab: Signal<AppTab>) -> Element {
    rsx! {
        div { class: "tabs",
            TabButton { active_tab, tab: AppTab::Chat, label: "Chat" }
            TabButton { active_tab, tab: AppTab::Docs, label: "Docs" }
            TabButton { active_tab, tab: AppTab::Settings, label: "Settings" }
        }
    }
}

#[component]
fn TabButton(active_tab: Signal<AppTab>, tab: AppTab, label: &'static str) -> Element {
    let mut active_tab = active_tab;
    let class = if active_tab() == tab {
        "tab active"
    } else {
        "tab"
    };
    rsx! {
        h1 {
            class: class,
            onclick: move |_| active_tab.set(tab),
            "{label}"
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
