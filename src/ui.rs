use crate::ai::{chat_reply_stream_poll, chat_reply_stream_start};
use crate::types::{ChatMessage, Role};
use dioxus::events::Key;
use dioxus::prelude::*;
use once_cell::sync::Lazy;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag, html};
use std::time::Instant;
use syntect::{highlighting::ThemeSet, html::highlighted_html_for_string, parsing::SyntaxSet};

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

pub fn markdown_to_html(md: &str) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(md, opts);

    // Transform code blocks to syntax-highlighted HTML
    let mut in_code = false;
    let mut code_lang: Option<String> = None;
    let mut code_buf = String::new();
    let mut events: Vec<Event> = Vec::new();

    for ev in parser {
        match ev {
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code = true;
                code_buf.clear();
                code_lang = match kind {
                    CodeBlockKind::Fenced(lang) => Some(lang.to_string()),
                    _ => None,
                };
            }
            Event::Text(text) if in_code => {
                code_buf.push_str(&text);
            }
            Event::End(Tag::CodeBlock(_)) if in_code => {
                // Highlight and inject as raw HTML
                let ss = &*SYNTAX_SET;
                let ts = &*THEME_SET;
                let theme = ts
                    .themes
                    .get("base16-ocean.dark")
                    .or_else(|| ts.themes.values().next());
                let lang = code_lang.as_deref().unwrap_or("");
                let html_block: String = if let Some(theme) = theme {
                    let syntax = ss
                        .find_syntax_by_token(lang)
                        .unwrap_or_else(|| ss.find_syntax_plain_text());
                    match highlighted_html_for_string(&code_buf, ss, syntax, theme) {
                        Ok(s) => s,
                        Err(_) => {
                            let escaped = code_buf
                                .replace('&', "&amp;")
                                .replace('<', "&lt;")
                                .replace('>', "&gt;");
                            format!("<pre><code>{}</code></pre>", escaped)
                        }
                    }
                } else {
                    let escaped = code_buf
                        .replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;");
                    format!("<pre><code>{}</code></pre>", escaped)
                };
                events.push(Event::Html(CowStr::from(html_block)));
                in_code = false;
                code_lang = None;
                code_buf.clear();
            }
            other if in_code => {
                if let Event::Text(t) = other {
                    code_buf.push_str(&t);
                }
            }
            other => events.push(other),
        }
    }

    let mut out = String::new();
    html::push_html(&mut out, events.into_iter());
    out
}

#[component]
pub fn AssistantBubble(
    content: String,
    show_copy: bool,
    processed_time_ms: Option<u128>,
    is_streaming: bool,
    saved_msgs: Signal<Vec<String>>,
) -> Element {
    let content_html = markdown_to_html(&content);
    let copy_payload = content.clone();
    let on_copy = move |_| {
        let raw = copy_payload.clone();
        spawn(async move {
            #[cfg(any(feature = "desktop", feature = "mobile"))]
            {
                if let Ok(mut cb) = arboard::Clipboard::new() {
                    let _ = cb.set_text(raw);
                }
            }
        });
    };

    // Convert ms to seconds with one decimal and build label
    let processed_label = processed_time_ms
        .map(|ms| (ms as f64) / 1000.0)
        .map(|s| format!("Processed in {:.1}s", s));

    // Save handler: push markdown into in-memory list
    let save_payload = content.clone();
    let on_save = move |_| {
        let raw = save_payload.clone();
        saved_msgs.with_mut(|arr| arr.push(raw));
    };

    rsx! {
        if show_copy {
            div { class: "bubble-controls",
                if let Some(lbl) = processed_label { span { class: "processed-time", "{lbl}" } }
                button { class: "action-btn", title: "Copy markdown", onclick: on_copy, "Copy" }
                button { class: "action-btn", title: "Save", onclick: on_save, "Save" }
            }
        }
        if is_streaming && content.is_empty() {
            div { class: "md", div { class: "shimmer-text", "Processing…" } }
        } else {
            div { class: "md", dangerous_inner_html: "{content_html}" }
        }
    }
}

const MOSTRA_CSS: Asset = asset!("/assets/mostra.css");

#[component]
pub fn App() -> Element {
    // Saved messages live for the session
    let saved_msgs = use_signal(Vec::<String>::new);
    let mut base_font_px = use_signal(|| 14i32);

    rsx! {
        document::Link { rel: "stylesheet", href: MOSTRA_CSS }
        // Dynamically scale base font size
        {
            let root_style = format!(":root {{ font-size: {}px; }}", base_font_px());
            rsx!( style { dangerous_inner_html: "{root_style}" } )
        }
        // Header title
        div { class: "header no-divider",
            div { class: "header-content",
                h1 { class: "tab active", "Chat" }
            }
        }

        Chat { saved_msgs, base_font_px }
    }
}

#[component]
fn Setting() -> Element {
    rsx! {
        div { class: "main-container",
            h2 { class: "page-title text-xl", "Setting" }
            p { class: "text-muted", "Coming soon." }
        }
    }
}

#[component]
fn Chat(saved_msgs: Signal<Vec<String>>, base_font_px: Signal<i32>) -> Element {
    let mut messages = use_signal(Vec::<ChatMessage>::new);
    let mut input = use_signal(String::new);
    let mut sending = use_signal(|| false);
    let mut streaming_index = use_signal(|| Option::<usize>::None);
    let mut processed_times = use_signal(Vec::<Option<u128>>::new);
    let mut processing_started_at = use_signal(|| Option::<Instant>::None);
    let scroll_to_bottom = || {};

    rsx! {
        div { class: "main-container",
            div { class: "chat-wrap",
                div { id: "chat-list", class: "chat-list",
                    for (i, msg) in messages().iter().enumerate() {
                        div { class: format_args!("message-row {}", match msg.role { Role::User => "user", Role::Assistant => "assistant" }),
                            if matches!(msg.role, Role::Assistant) { div { class: "avatar assistant", "C" } }
                            div { class: format_args!(
                                    "bubble {} {}",
                                    match msg.role { Role::User => "user", Role::Assistant => "assistant" },
                                    if matches!(msg.role, Role::Assistant)
                                        && matches!(streaming_index(), Some(idx) if idx == i)
                                        && msg.content.is_empty() { "pending" } else { "" }
                                ),
                                if matches!(msg.role, Role::Assistant) {
                                    AssistantBubble {
                                        content: msg.content.clone(),
                                        show_copy: match streaming_index() { Some(idx) => idx != i, None => true },
                                        processed_time_ms: processed_times().get(i).cloned().unwrap_or(None),
                                        is_streaming: matches!(streaming_index(), Some(idx) if idx == i),
                                        saved_msgs,
                                    }
                                } else { "{msg.content}" }
                            }
                        }
                    }
                }
            }

            form { class: "composer no-divider",
                div { class: "composer-inner",
                    div { class: "hstack", style: "gap: 0.5rem; width: 100%; align-items: flex-end;",
                        textarea {
                            class: "", rows: "1", placeholder: "What can I help you with?",
                            value: "{input}", oninput: move |ev| input.set(ev.value()),
                            onkeydown: move |ev| {
                                // Zoom via Cmd/Ctrl + and -
                                if ev.modifiers().meta() || ev.modifiers().ctrl() {
                                    if ev.key() == Key::Character("+".into()) || ev.key() == Key::Character("=".into()) {
                                        ev.prevent_default();
                                        base_font_px.set((base_font_px() + 1).clamp(12, 22));
                                        return;
                                    }
                                    if ev.key() == Key::Character("-".into()) {
                                        ev.prevent_default();
                                        base_font_px.set((base_font_px() - 1).clamp(12, 22));
                                        return;
                                    }
                                }
                                if ev.key() == Key::Enter && !ev.modifiers().shift() {
                                    ev.prevent_default();
                                    let text = input().trim().to_string();
                                    if text.is_empty() || sending() { return; }
                                    messages.with_mut(|msgs| msgs.push(ChatMessage { role: Role::User, content: text.clone() }));
                                    processed_times.with_mut(|ts| ts.push(None));
                                    scroll_to_bottom();
                                    input.set(String::new());
                                    sending.set(true);
                                    let mut inserted_index: usize = 0;
                                    messages.with_mut(|msgs| { inserted_index = msgs.len(); msgs.push(ChatMessage { role: Role::Assistant, content: String::new() }); });
                                    processed_times.with_mut(|ts| ts.push(None));
                                    streaming_index.set(Some(inserted_index));
                                    processing_started_at.set(Some(Instant::now()));
                                    let msgs_for_server = messages();
                                    spawn(async move {
                                        match chat_reply_stream_start(msgs_for_server).await {
                                            Ok(stream_id) => {
                                                loop {
                                                    match chat_reply_stream_poll(stream_id).await {
                                                        Ok((content, done)) => {
                                                            messages.with_mut(|msgs| if let Some(msg) = msgs.get_mut(inserted_index) { msg.content = content.clone(); });
                                                            scroll_to_bottom();
                                                            if done { break; }
                                                        }
                                                        Err(err) => { eprintln!("stream poll error: {}", err); break; }
                                                    }
                                                    tokio::time::sleep(std::time::Duration::from_millis(80)).await;
                                                }
                                            }
                                            Err(err) => eprintln!("chat start error: {}", err),
                                        }
                                        // Capture processed time
                                        if let Some(start) = processing_started_at() {
                                            let ms = start.elapsed().as_millis();
                                            processed_times.with_mut(|ts| if let Some(slot) = ts.get_mut(inserted_index) { *slot = Some(ms); });
                                        }
                                        streaming_index.set(None);
                                        sending.set(false);
                                    });
                                }
                            },
                            disabled: sending(), autofocus: true,
                        }
                        button {
                            class: "btn btn-primary", r#type: "button",
                            disabled: sending() || input().trim().is_empty(),
                            onclick: move |_| {
                                let text = input().trim().to_string();
                                if text.is_empty() || sending() { return; }
                                messages.with_mut(|msgs| msgs.push(ChatMessage { role: Role::User, content: text.clone() }));
                                processed_times.with_mut(|ts| ts.push(None));
                                scroll_to_bottom();
                                input.set(String::new());
                                sending.set(true);
                                let mut inserted_index: usize = 0;
                                messages.with_mut(|msgs| { inserted_index = msgs.len(); msgs.push(ChatMessage { role: Role::Assistant, content: String::new() }); });
                                processed_times.with_mut(|ts| ts.push(None));
                                streaming_index.set(Some(inserted_index));
                                processing_started_at.set(Some(Instant::now()));
                                let msgs_for_server = messages();
                                spawn(async move {
                                    match chat_reply_stream_start(msgs_for_server).await {
                                        Ok(stream_id) => {
                                            loop {
                                                match chat_reply_stream_poll(stream_id).await {
                                                    Ok((content, done)) => {
                                                        messages.with_mut(|msgs| if let Some(msg) = msgs.get_mut(inserted_index) { msg.content = content.clone(); });
                                                        scroll_to_bottom();
                                                        if done { break; }
                                                    }
                                                    Err(err) => { eprintln!("stream poll error: {}", err); break; }
                                                }
                                                tokio::time::sleep(std::time::Duration::from_millis(80)).await;
                                            }
                                        }
                                        Err(err) => eprintln!("chat start error: {}", err),
                                    }
                                    if let Some(start) = processing_started_at() {
                                        let ms = start.elapsed().as_millis();
                                        processed_times.with_mut(|ts| if let Some(slot) = ts.get_mut(inserted_index) { *slot = Some(ms); });
                                    }
                                    streaming_index.set(None);
                                    sending.set(false);
                                });
                            },
                            "Send"
                        }
                    }
                }
            }
        }
    }
}
