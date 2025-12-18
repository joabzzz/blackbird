use crate::ai::{chat_reply_stream_poll, chat_reply_stream_start};
use crate::types::{ChatMessage, Role, ThemeMode};
use crate::views::shared::{SavedApp, persist_app};
use dioxus::events::Key;
use dioxus::prelude::*;
use time::OffsetDateTime;

/// CSS that gets injected into generated apps to match Blackbird's theme
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

const APP_BUILDER_SYSTEM_PROMPT: &str = r#"
You are Blackbird Workbench. You build interactive HTML/JS/CSS apps.

RULES:
1. Respond ONLY with HTML code. No markdown, no explanations.
2. Output a complete HTML document with embedded <style> and <script>.
3. IMPORTANT: Do NOT include any body styling (background, color, font-family, margin, padding). The parent app injects theme CSS automatically.
4. Use CSS variables: --bg, --bg-secondary, --text, --text-muted, --border, --accent, --surface
5. Include a <title> tag.
6. End with: [[app_tags: Tag1, Tag2]]

Example structure:
<!DOCTYPE html>
<html>
<head>
    <title>My App</title>
    <style>
        .container { /* your styles using var(--text), var(--bg), etc */ }
    </style>
</head>
<body>
    <div class="container">...</div>
    <script>// your code</script>
</body>
</html>
[[app_tags: Utility]]
"#;

#[component]
pub fn WorkbenchView(
    saved_apps: Signal<Vec<SavedApp>>,
    base_font_px: Signal<i32>,
    theme: Signal<ThemeMode>,
) -> Element {
    let state = use_workbench_state();
    let mut show_clear_confirm = use_signal(|| false);

    rsx! {
        div { class: "workbench-container",
            WorkbenchDisplay { state, saved_apps, theme, show_clear_confirm }
            WorkbenchComposer { state, base_font_px }

            // Clear confirmation overlay
            if show_clear_confirm() {
                div { class: "confirm-overlay",
                    onclick: move |_| show_clear_confirm.set(false),
                    div { class: "confirm-dialog",
                        onclick: move |e| e.stop_propagation(),
                        p { "Clear conversation?" }
                        div { class: "confirm-actions",
                            button {
                                class: "btn",
                                onclick: move |_| show_clear_confirm.set(false),
                                "Cancel"
                            }
                            button {
                                class: "btn btn-primary",
                                onclick: move |_| {
                                    state.clear();
                                    show_clear_confirm.set(false);
                                },
                                "Clear"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn WorkbenchDisplay(
    state: WorkbenchState,
    saved_apps: Signal<Vec<SavedApp>>,
    theme: Signal<ThemeMode>,
    show_clear_confirm: Signal<bool>,
) -> Element {
    let messages = state.messages();
    let is_streaming = state.streaming_index().is_some();
    let logs_expanded = state.logs_expanded();

    let latest_app = messages
        .iter()
        .rev()
        .find(|msg| matches!(msg.role, Role::Assistant) && !msg.content.is_empty())
        .map(|msg| extract_html_content(&msg.content));

    let has_content = !messages.is_empty();

    rsx! {
        div { class: "workbench-display",
            if is_streaming {
                div { class: "workbench-status",
                    span { class: "shimmer-text", "Building..." }
                }
            } else if let Some(ref html) = latest_app {
                if !html.is_empty() {
                    AppRenderer { html: html.clone(), saved_apps, state, theme }
                }
            }

            if !is_streaming && latest_app.as_ref().map(|h| h.is_empty()).unwrap_or(true) && !has_content {
                div { class: "workbench-empty",
                    span { class: "text-muted", "Describe an app to build" }
                }
            }

            if has_content {
                div { class: "workbench-controls",
                    button {
                        class: "logs-toggle",
                        onclick: move |_| state.toggle_logs(),
                        if logs_expanded { "Hide log" } else { "Log" }
                    }
                    button {
                        class: "clear-btn",
                        onclick: move |_| show_clear_confirm.set(true),
                        "Clear"
                    }
                }
                if logs_expanded {
                    div { class: "workbench-logs",
                        for msg in messages.iter() {
                            div { class: format_args!("log-entry {}", if matches!(msg.role, Role::User) { "user" } else { "assistant" }),
                                span { class: "log-role", if matches!(msg.role, Role::User) { "You" } else { "AI" } }
                                span { class: "log-content", "{truncate(&msg.content, 80)}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn AppRenderer(
    html: String,
    saved_apps: Signal<Vec<SavedApp>>,
    state: WorkbenchState,
    theme: Signal<ThemeMode>,
) -> Element {
    let html_for_save = html.clone();
    let tags = state.current_tags();
    let theme_css = app_theme_css(theme());

    // Inject theme CSS into the HTML
    let themed_html = inject_theme_css(&html, theme_css);

    let on_save = move |_| {
        let content = html_for_save.clone();
        let title = extract_app_title(&content).unwrap_or_else(|| "Untitled App".to_string());
        let tag_refs: Vec<String> = tags.clone();
        if let Some(app) = persist_app(&content, &title, Some(&tag_refs)) {
            saved_apps.with_mut(|apps| {
                apps.retain(|existing| existing.id != app.id);
                apps.insert(0, app);
            });
        }
    };

    rsx! {
        div { class: "app-container",
            button { class: "app-save-btn action-btn", onclick: on_save, "Save" }
            iframe { class: "app-frame", srcdoc: "{themed_html}" }
        }
    }
}

#[component]
fn WorkbenchComposer(state: WorkbenchState, base_font_px: Signal<i32>) -> Element {
    let sending = state.sending();
    let input_value = state.input();
    let mut font_size = base_font_px;

    rsx! {
        div { class: "workbench-composer",
            textarea {
                placeholder: "Build me a calculator...",
                value: "{input_value}",
                inputmode: "text",
                autocomplete: "off",
                spellcheck: "false",
                oninput: move |ev| state.set_input(ev.value()),
                onkeydown: move |ev| {
                    if ev.modifiers().meta() || ev.modifiers().ctrl() {
                        if ev.key() == Key::Character("+".into()) || ev.key() == Key::Character("=".into()) {
                            ev.prevent_default();
                            font_size.set((font_size() + 1).clamp(12, 22));
                            return;
                        }
                        if ev.key() == Key::Character("-".into()) {
                            ev.prevent_default();
                            font_size.set((font_size() - 1).clamp(12, 22));
                            return;
                        }
                    }
                    if ev.key() == Key::Enter && !ev.modifiers().shift() {
                        ev.prevent_default();
                        state.submit_input();
                    }
                },
                disabled: sending,
                autofocus: true,
            }
            button {
                class: "btn btn-primary",
                disabled: sending || input_value.trim().is_empty(),
                onclick: move |_| state.submit_input(),
                "Build"
            }
        }
    }
}

// ============================================
// State
// ============================================

#[derive(Clone, Copy)]
struct WorkbenchState {
    messages: Signal<Vec<ChatMessage>>,
    input: Signal<String>,
    sending: Signal<bool>,
    streaming_index: Signal<Option<usize>>,
    logs_expanded: Signal<bool>,
    current_tags: Signal<Vec<String>>,
}

impl PartialEq for WorkbenchState {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

fn use_workbench_state() -> WorkbenchState {
    WorkbenchState {
        messages: use_signal(Vec::<ChatMessage>::new),
        input: use_signal(String::new),
        sending: use_signal(|| false),
        streaming_index: use_signal(|| None),
        logs_expanded: use_signal(|| false),
        current_tags: use_signal(Vec::<String>::new),
    }
}

impl WorkbenchState {
    fn messages(&self) -> Vec<ChatMessage> {
        (self.messages)()
    }
    fn streaming_index(&self) -> Option<usize> {
        (self.streaming_index)()
    }
    fn input(&self) -> String {
        (self.input)()
    }
    fn set_input(&self, v: String) {
        let mut input = self.input;
        input.set(v);
    }
    fn sending(&self) -> bool {
        (self.sending)()
    }
    fn logs_expanded(&self) -> bool {
        (self.logs_expanded)()
    }
    fn toggle_logs(&self) {
        let mut l = self.logs_expanded;
        l.set(!l());
    }
    fn current_tags(&self) -> Vec<String> {
        (self.current_tags)()
    }

    fn clear(&self) {
        let mut messages = self.messages;
        messages.set(Vec::new());
        let mut current_tags = self.current_tags;
        current_tags.set(Vec::new());
        let mut logs = self.logs_expanded;
        logs.set(false);
    }

    fn submit_input(&self) {
        let text = self.input().trim().to_string();
        if text.is_empty() || self.sending() {
            return;
        }

        let mut messages = self.messages;
        messages.with_mut(|msgs| {
            msgs.push(ChatMessage {
                role: Role::User,
                content: text,
                created_at: Some(OffsetDateTime::now_utc()),
                tags: Vec::new(),
            });
        });

        let mut input = self.input;
        input.set(String::new());

        let snapshot = self.messages();

        let mut sending = self.sending;
        sending.set(true);

        let idx = {
            let mut index = 0;
            messages.with_mut(|msgs| {
                index = msgs.len();
                msgs.push(ChatMessage {
                    role: Role::Assistant,
                    content: String::new(),
                    created_at: Some(OffsetDateTime::now_utc()),
                    tags: Vec::new(),
                });
            });
            index
        };

        let mut streaming_index = self.streaming_index;
        streaming_index.set(Some(idx));

        let mut server_msgs = vec![ChatMessage {
            role: Role::User,
            content: APP_BUILDER_SYSTEM_PROMPT.to_string(),
            created_at: None,
            tags: Vec::new(),
        }];
        server_msgs.extend(snapshot);

        let state = *self;
        spawn(async move {
            let mut failed = false;
            match chat_reply_stream_start(server_msgs).await {
                Ok(id) => loop {
                    match chat_reply_stream_poll(id).await {
                        Ok((content, done)) => {
                            let mut messages = state.messages;
                            messages.with_mut(|msgs| {
                                if let Some(msg) = msgs.get_mut(idx) {
                                    msg.content = content;
                                }
                            });
                            if done {
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("poll error: {}", e);
                            failed = true;
                            break;
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(80)).await;
                },
                Err(e) => {
                    eprintln!("start error: {}", e);
                    failed = true;
                }
            }

            let mut messages = state.messages;
            if failed {
                messages.with_mut(|msgs| {
                    if let Some(msg) = msgs.get_mut(idx) {
                        msg.content = "Failed to build. Try again.".to_string();
                    }
                });
            } else {
                let content = state
                    .messages
                    .with(|msgs| msgs.get(idx).map(|m| m.content.clone()));
                if let Some(content) = content {
                    let (clean, tags) = extract_app_tags(&content);
                    messages.with_mut(|msgs| {
                        if let Some(msg) = msgs.get_mut(idx) {
                            msg.content = clean;
                            msg.tags = tags.clone();
                        }
                    });
                    let mut current_tags = state.current_tags;
                    current_tags.set(tags);
                }
            }

            let mut streaming_index = state.streaming_index;
            streaming_index.set(None);
            let mut sending = state.sending;
            sending.set(false);
        });
    }
}

// ============================================
// Helpers
// ============================================

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

fn extract_html_content(content: &str) -> String {
    if let Some(start) = content.find("```html") {
        let after = start + 7;
        if let Some(end) = content[after..].find("```") {
            return content[after..after + end].trim().to_string();
        }
    }
    let trimmed = content.trim();
    if trimmed.starts_with("<!DOCTYPE") || trimmed.starts_with("<html") || trimmed.starts_with("<")
    {
        if let Some(pos) = trimmed.rfind("[[app_tags:") {
            return trimmed[..pos].trim().to_string();
        }
        return trimmed.to_string();
    }
    String::new()
}

fn extract_app_title(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let start = lower.find("<title>")? + 7;
    let end = lower[start..].find("</title>")?;
    let title = html[start..start + end].trim();
    if title.is_empty() {
        None
    } else {
        Some(title.to_string())
    }
}

fn extract_app_tags(content: &str) -> (String, Vec<String>) {
    if let Some(start) = content.rfind("[[app_tags:")
        && let Some(end) = content[start..].find("]]")
    {
        let raw = &content[start + 11..start + end];
        let clean = content[..start].trim_end().to_string();
        let tags: Vec<String> = raw
            .split(',')
            .filter_map(|t| {
                let t = t.trim();
                if t.is_empty() {
                    None
                } else {
                    let mut c = t.chars();
                    Some(format!("{}{}", c.next()?.to_uppercase(), c.as_str()))
                }
            })
            .collect();
        return (clean, tags);
    }
    (content.to_string(), vec!["App".to_string()])
}

fn inject_theme_css(html: &str, theme_css: &str) -> String {
    // Insert theme CSS right after <head> or at the start if no head
    if let Some(pos) = html.to_lowercase().find("<head>") {
        let insert_pos = pos + 6;
        format!(
            "{}<style>{}</style>{}",
            &html[..insert_pos],
            theme_css,
            &html[insert_pos..]
        )
    } else if let Some(pos) = html.to_lowercase().find("<html") {
        // Find end of <html> tag
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
