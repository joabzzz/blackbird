use crate::types::ThemeMode;
use dioxus::prelude::*;

#[component]
pub fn SettingsView(theme: Signal<ThemeMode>) -> Element {
    let language = use_signal(|| "English".to_string());

    rsx! {
        div { class: "main-container",
            div { class: "settings-section",
                h3 { class: "section-title", "Display" }
                div { class: "theme-toggle",
                    button {
                        class: format_args!(
                            "theme-option {}",
                            if matches!(theme(), ThemeMode::Dark) { "active" } else { "" }
                        ),
                        r#type: "button",
                        onclick: move |_| theme.set(ThemeMode::Dark),
                        "Dark"
                    }
                    button {
                        class: format_args!(
                            "theme-option {}",
                            if matches!(theme(), ThemeMode::Light) { "active" } else { "" }
                        ),
                        r#type: "button",
                        onclick: move |_| theme.set(ThemeMode::Light),
                        "Light"
                    }
                    button {
                        class: format_args!(
                            "theme-option {}",
                            if matches!(theme(), ThemeMode::Octane) { "active" } else { "" }
                        ),
                        r#type: "button",
                        onclick: move |_| theme.set(ThemeMode::Octane),
                        "Octane"
                    }
                }
            }
            div { class: "settings-section",
                h3 { class: "section-title", "Language" }
                div { class: "locked-input",
                    input {
                        r#type: "text",
                        value: "{language()}",
                        readonly: true,
                        disabled: true,
                    }
                    span { class: "lock-icon", dangerous_inner_html: "&#128274;" }
                }
            }
            div { class: "settings-section",
                h3 { class: "section-title", "Account" }
                p { class: "text-muted", "Account settings coming soon." }
            }
        }
    }
}
