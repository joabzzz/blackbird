use crate::bridge;
use crate::types::ThemeMode;
use crate::views::shared::SavedApp;
use dioxus::{
    events::{FormEvent, Key, KeyboardEvent, MouseEvent},
    prelude::*,
};
use time::{OffsetDateTime, UtcOffset, format_description::FormatItem, macros::format_description};

/// CSS that gets injected into apps to match Blackbird's theme
fn app_theme_css(theme: ThemeMode) -> &'static str {
    match theme {
        ThemeMode::Dark => {
            r#"
            :root {
                --bg: #000000;
                --bg-secondary: #050505;
                --text: #ffffff;
                --text-muted: #cfcfcf;
                --border: #ffffff;
                --accent: #ff3509;
                --surface: #111111;
            }
            body {
                background: var(--bg);
                color: var(--text);
                font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
                margin: 0;
                padding: 1rem;
                min-height: 100vh;
                box-sizing: border-box;
            }
            * { box-sizing: border-box; }
            button {
                background: transparent;
                border: 1px solid var(--border);
                color: var(--text);
                padding: 0.6rem 1rem;
                border-radius: 8px;
                cursor: pointer;
                font-weight: 600;
            }
            button:hover { background: var(--surface); }
            input, textarea {
                background: var(--bg);
                border: 1px solid var(--border);
                color: var(--text);
                padding: 0.6rem;
                border-radius: 8px;
            }
        "#
        }
        ThemeMode::Light => {
            r#"
            :root {
                --bg: #ffffff;
                --bg-secondary: #f5f5f5;
                --text: #000000;
                --text-muted: #4a4a4a;
                --border: #000000;
                --accent: #ff3509;
                --surface: #e6e6e6;
            }
            body {
                background: var(--bg);
                color: var(--text);
                font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
                margin: 0;
                padding: 1rem;
                min-height: 100vh;
                box-sizing: border-box;
            }
            * { box-sizing: border-box; }
            button {
                background: transparent;
                border: 1px solid var(--border);
                color: var(--text);
                padding: 0.6rem 1rem;
                border-radius: 8px;
                cursor: pointer;
                font-weight: 600;
            }
            button:hover { background: var(--surface); }
            input, textarea {
                background: var(--bg);
                border: 1px solid var(--border);
                color: var(--text);
                padding: 0.6rem;
                border-radius: 8px;
            }
        "#
        }
        ThemeMode::Octane => {
            r#"
            :root {
                --bg: #ff3509;
                --bg-secondary: #ffe0d1;
                --text: #000000;
                --text-muted: #2c2c2c;
                --border: #000000;
                --accent: #ffffff;
                --surface: rgba(0, 0, 0, 0.12);
            }
            body {
                background: var(--bg);
                color: var(--text);
                font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
                margin: 0;
                padding: 1rem;
                min-height: 100vh;
                box-sizing: border-box;
            }
            * { box-sizing: border-box; }
            button {
                background: transparent;
                border: 1px solid var(--border);
                color: var(--text);
                padding: 0.6rem 1rem;
                border-radius: 8px;
                cursor: pointer;
                font-weight: 600;
            }
            button:hover { background: var(--surface); }
            input, textarea {
                background: var(--bg-secondary);
                border: 1px solid var(--border);
                color: var(--text);
                padding: 0.6rem;
                border-radius: 8px;
            }
        "#
        }
    }
}

fn inject_theme_css(html: &str, theme_css: &str) -> String {
    if let Some(pos) = html.to_lowercase().find("<head>") {
        let insert_pos = pos + 6;
        format!(
            "{}<style>{}</style>{}",
            &html[..insert_pos],
            theme_css,
            &html[insert_pos..]
        )
    } else if let Some(pos) = html.to_lowercase().find("<html") {
        if let Some(end) = html[pos..].find('>') {
            let insert_pos = pos + end + 1;
            format!(
                "{}<head><style>{}</style></head>{}",
                &html[..insert_pos],
                theme_css,
                &html[insert_pos..]
            )
        } else {
            format!("<style>{}</style>{}", theme_css, html)
        }
    } else {
        format!("<style>{}</style>{}", theme_css, html)
    }
}

/// Inject both theme CSS and the Blackbird SDK into app HTML
fn inject_theme_and_sdk(html: &str, theme_css: &str, app_id: &str) -> String {
    let sdk_script = bridge::get_sdk_script(app_id);
    let themed = inject_theme_css(html, theme_css);

    // Insert SDK script before </body> or at the end
    if let Some(pos) = themed.to_lowercase().rfind("</body>") {
        format!("{}{}{}", &themed[..pos], sdk_script, &themed[pos..])
    } else if let Some(pos) = themed.to_lowercase().rfind("</html>") {
        format!("{}{}{}", &themed[..pos], sdk_script, &themed[pos..])
    } else {
        format!("{}{}", themed, sdk_script)
    }
}

const APP_DATE_FORMAT: &[FormatItem<'static>] =
    format_description!("[month repr:short] [day padding:zero], [year]");

#[derive(Clone, Copy, PartialEq, Eq)]
enum AppSort {
    Newest,
    Oldest,
    Title,
}

#[component]
pub fn AppsView(saved_apps: Signal<Vec<SavedApp>>, theme: Signal<ThemeMode>) -> Element {
    let mut sort_mode = use_signal(|| AppSort::Newest);
    let mut tag_filter = use_signal(|| Option::<String>::None);
    let mut delete_confirm_id = use_signal(|| Option::<String>::None);
    let mut booted_app = use_signal(|| Option::<SavedApp>::None);

    let apps = saved_apps();

    let mut all_tags: Vec<String> = apps
        .iter()
        .flat_map(|app| app.tags.iter().cloned())
        .collect();
    all_tags.sort_unstable();
    all_tags.dedup();

    let filter_tag = tag_filter();
    let mut display_apps = apps.clone();
    if let Some(tag) = filter_tag.as_ref() {
        let tag_lower = tag.to_lowercase();
        display_apps.retain(|app| {
            app.tags
                .iter()
                .any(|candidate| candidate.to_lowercase() == tag_lower)
        });
    }

    match sort_mode() {
        AppSort::Newest => display_apps.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
        AppSort::Oldest => display_apps.sort_by(|a, b| a.created_at.cmp(&b.created_at)),
        AppSort::Title => {
            display_apps.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
        }
    }

    rsx! {
        div { class: "main-container apps-container",
            // Booted app fullscreen view
            if let Some(app) = booted_app() {
                {
                    let theme_css = app_theme_css(theme());
                    let themed_content = inject_theme_and_sdk(&app.content, theme_css, &app.id);
                    rsx! {
                        div { class: "booted-app-overlay",
                            iframe {
                                class: "booted-app-frame",
                                srcdoc: "{themed_content}",
                            }
                            button {
                                class: "booted-app-close",
                                onclick: move |_| booted_app.set(None),
                                dangerous_inner_html: "&times;"
                            }
                        }
                    }
                }
            }

            if apps.is_empty() {
                div { class: "apps-empty",
                    h3 { "No saved apps yet" }
                    p { class: "text-muted", "Build an app in Workbench and save it to see it here." }
                }
            } else {
                div { class: "app-controls",
                    div { class: "app-control-group",
                        label { for: "app-sort", class: "control-label", "Sort" }
                        select {
                            id: "app-sort",
                            value: match sort_mode() { AppSort::Newest => "newest", AppSort::Oldest => "oldest", AppSort::Title => "title" },
                            onchange: move |evt: FormEvent| {
                                let mode = match evt.value().as_str() {
                                    "oldest" => AppSort::Oldest,
                                    "title" => AppSort::Title,
                                    _ => AppSort::Newest,
                                };
                                sort_mode.set(mode);
                            },
                            option { value: "newest", "Newest" }
                            option { value: "oldest", "Oldest" }
                            option { value: "title", "Title" }
                        }
                    }
                    div { class: "app-control-group",
                        label { for: "app-tag", class: "control-label", "Filter" }
                        select {
                            id: "app-tag",
                            value: filter_tag.clone().unwrap_or_default(),
                            onchange: {
                                move |evt: FormEvent| {
                                    let value = evt.value();
                                    if value.is_empty() {
                                        tag_filter.set(None);
                                    } else {
                                        tag_filter.set(Some(value));
                                    }
                                }
                            },
                            option { value: "", "All tags" }
                            for tag in all_tags.iter() {
                                option { value: "{tag}", "{tag}" }
                            }
                        }
                    }
                }
                if display_apps.is_empty() {
                    div { class: "apps-empty",
                        p { class: "text-muted", "No apps match the selected filters." }
                    }
                } else {
                    div { class: "apps-grid",
                        for app in display_apps.iter().cloned() {
                            div {
                                key: "{app.id}",
                                class: "app-card",
                                role: "button",
                                tabindex: "0",
                                // Tap/click boots the app directly
                                onclick: {
                                    let app_clone = app.clone();
                                    move |_| booted_app.set(Some(app_clone.clone()))
                                },
                                // Right-click/long-press shows delete option
                                oncontextmenu: {
                                    let app_id = app.id.clone();
                                    move |evt: MouseEvent| {
                                        evt.prevent_default();
                                        delete_confirm_id.set(Some(app_id.clone()));
                                    }
                                },
                                onkeydown: {
                                    let app_clone = app.clone();
                                    move |evt: KeyboardEvent| {
                                        let key = evt.key();
                                        let activate = match key {
                                            Key::Enter => true,
                                            Key::Character(ref value) if value == " " => true,
                                            _ => false,
                                        };
                                        if activate {
                                            evt.stop_propagation();
                                            evt.prevent_default();
                                            booted_app.set(Some(app_clone.clone()));
                                        }
                                    }
                                },
                                div { class: "app-card-preview",
                                    iframe {
                                        class: "app-card-iframe",
                                        srcdoc: "{app.content}",
                                        tabindex: "-1",
                                    }
                                }
                                div { class: "app-card-info",
                                    h4 { class: "app-card-title", "{app.title}" }
                                    div { class: "app-card-tags",
                                        for tag in app.tags.iter() {
                                            span { class: "tag-pill tag-pill-compact", "{tag}" }
                                        }
                                    }
                                    span { class: "app-card-date", "{app_saved_date(app.created_at)}" }
                                }
                            }
                        }
                    }
                }
            }

            // Delete confirmation overlay
            if let Some(app_id) = delete_confirm_id() {
                div { class: "confirm-overlay",
                    onclick: move |_| delete_confirm_id.set(None),
                    div { class: "confirm-dialog",
                        onclick: move |e| e.stop_propagation(),
                        p { "Delete this app?" }
                        div { class: "confirm-actions",
                            button {
                                class: "btn",
                                onclick: move |_| delete_confirm_id.set(None),
                                "Cancel"
                            }
                            button {
                                class: "btn btn-danger",
                                onclick: move |_| {
                                    // Delete the app
                                    delete_app(&app_id);
                                    saved_apps.with_mut(|apps| {
                                        apps.retain(|a| a.id != app_id);
                                    });
                                    delete_confirm_id.set(None);
                                },
                                "Delete"
                            }
                        }
                    }
                }
            }
        }
    }
}

fn app_saved_date(timestamp: u64) -> String {
    if timestamp == 0 {
        return "Unknown date".to_string();
    }

    let Ok(mut datetime) = OffsetDateTime::from_unix_timestamp(timestamp as i64) else {
        return "Unknown date".to_string();
    };

    if let Ok(offset) = UtcOffset::current_local_offset() {
        datetime = datetime.to_offset(offset);
    }

    datetime
        .format(APP_DATE_FORMAT)
        .unwrap_or_else(|_| "Unknown date".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn delete_app(app_id: &str) {
    // app_id is the file path
    if let Err(e) = std::fs::remove_file(app_id) {
        eprintln!("Failed to delete app file: {}", e);
    }
}

#[cfg(target_arch = "wasm32")]
fn delete_app(_app_id: &str) {
    // No file to delete on wasm
}
