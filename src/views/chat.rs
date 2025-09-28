use crate::ai::{chat_reply_stream_poll, chat_reply_stream_start};
use crate::types::{ChatMessage, Role};
use crate::views::shared::{SavedDoc, markdown_to_html, persist_markdown_doc};
use dioxus::events::Key;
use dioxus::prelude::*;
use std::time::Instant;
use time::{OffsetDateTime, UtcOffset, format_description::FormatItem, macros::format_description};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct MessagePerformance {
    duration_ms: u128,
    token_count: usize,
    tokens_per_second: f64,
}

fn estimate_token_count(content: &str) -> usize {
    let words = content
        .split_whitespace()
        .filter(|segment| !segment.is_empty())
        .count();
    if words > 0 {
        words
    } else {
        content.chars().count().div_ceil(4)
    }
}

const MESSAGE_TIME_FORMAT: &[FormatItem<'static>] =
    format_description!("[hour repr:12 padding:zero]:[minute padding:zero] [period case:upper]");

fn current_time() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

fn format_message_timestamp(timestamp: Option<OffsetDateTime>) -> Option<String> {
    let mut datetime = timestamp?;
    if let Ok(offset) = UtcOffset::current_local_offset() {
        datetime = datetime.to_offset(offset);
    }
    datetime.format(MESSAGE_TIME_FORMAT).ok()
}

fn finalize_message_metrics(
    mut performances: Signal<Vec<Option<MessagePerformance>>>,
    messages: Signal<Vec<ChatMessage>>,
    mut processing_started_at: Signal<Option<Instant>>,
    index: usize,
) {
    if let Some(start) = processing_started_at() {
        let duration_ms = start.elapsed().as_millis();
        let content = messages.with(|msgs| msgs.get(index).map(|msg| msg.content.clone()));
        if let Some(content) = content {
            let token_count = estimate_token_count(&content);
            let duration_secs = duration_ms as f64 / 1000.0;
            let tokens_per_second = if duration_secs > 0.0 {
                token_count as f64 / duration_secs
            } else {
                token_count as f64
            };
            performances.with_mut(|slots| {
                if let Some(slot) = slots.get_mut(index) {
                    *slot = Some(MessagePerformance {
                        duration_ms,
                        token_count,
                        tokens_per_second,
                    });
                }
            });
        }
    }
    processing_started_at.set(None);
}

#[component]
pub fn ChatView(saved_docs: Signal<Vec<SavedDoc>>, base_font_px: Signal<i32>) -> Element {
    let mut messages = use_signal(Vec::<ChatMessage>::new);
    let mut input = use_signal(String::new);
    let mut sending = use_signal(|| false);
    let mut streaming_index = use_signal(|| Option::<usize>::None);
    let mut performances = use_signal(Vec::<Option<MessagePerformance>>::new);
    let mut processing_started_at = use_signal(|| Option::<Instant>::None);
    let scroll_to_bottom = || {};

    rsx! {
        div { class: "main-container",
            div { class: "chat-wrap",
                div { id: "chat-list", class: "chat-list",
                    for (i, msg) in messages().iter().enumerate() {
                        div { class: format_args!("message-row {}", match msg.role { Role::User => "user", Role::Assistant => "assistant" }),
                            if matches!(msg.role, Role::Assistant) { div { class: "avatar assistant", "C" } }
                            div { class: "message-stack",
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
                                            performance: performances().get(i).cloned().unwrap_or(None),
                                            is_streaming: matches!(streaming_index(), Some(idx) if idx == i),
                                            saved_docs,
                                        }
                                    } else { "{msg.content}" }
                                }
                                if let Some(ts) = format_message_timestamp(msg.created_at) {
                                    span { class: format_args!(
                                            "message-timestamp {}",
                                            match msg.role { Role::User => "align-end", Role::Assistant => "align-start" }
                                        ),
                                        "{ts}"
                                    }
                                }
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
                                    messages.with_mut(|msgs| msgs.push(ChatMessage {
                                        role: Role::User,
                                        content: text.clone(),
                                        created_at: Some(current_time()),
                                    }));
                                    performances.with_mut(|ts| ts.push(None));
                                    scroll_to_bottom();
                                    input.set(String::new());
                                    sending.set(true);
                                    let mut inserted_index: usize = 0;
                                    messages.with_mut(|msgs| {
                                        inserted_index = msgs.len();
                                        msgs.push(ChatMessage {
                                            role: Role::Assistant,
                                            content: String::new(),
                                            created_at: Some(current_time()),
                                        });
                                    });
                                    performances.with_mut(|ts| ts.push(None));
                                    streaming_index.set(Some(inserted_index));
                                    processing_started_at.set(Some(Instant::now()));
                                    let msgs_for_server = messages();
                                    let performance_signal = performances;
                                    let metrics_messages = messages;
                                    let metrics_started_at = processing_started_at;
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
                                        finalize_message_metrics(
                                            performance_signal,
                                            metrics_messages,
                                            metrics_started_at,
                                            inserted_index,
                                        );
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
                                messages.with_mut(|msgs| msgs.push(ChatMessage {
                                    role: Role::User,
                                    content: text.clone(),
                                    created_at: Some(current_time()),
                                }));
                                performances.with_mut(|ts| ts.push(None));
                                scroll_to_bottom();
                                input.set(String::new());
                                sending.set(true);
                                let mut inserted_index: usize = 0;
                                messages.with_mut(|msgs| {
                                    inserted_index = msgs.len();
                                    msgs.push(ChatMessage {
                                        role: Role::Assistant,
                                        content: String::new(),
                                        created_at: Some(current_time()),
                                    });
                                });
                                performances.with_mut(|ts| ts.push(None));
                                streaming_index.set(Some(inserted_index));
                                processing_started_at.set(Some(Instant::now()));
                                let msgs_for_server = messages();
                                let performance_signal = performances;
                                let metrics_messages = messages;
                                let metrics_started_at = processing_started_at;
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
                                    finalize_message_metrics(
                                        performance_signal,
                                        metrics_messages,
                                        metrics_started_at,
                                        inserted_index,
                                    );
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

#[component]
fn AssistantBubble(
    content: String,
    show_copy: bool,
    performance: Option<MessagePerformance>,
    is_streaming: bool,
    saved_docs: Signal<Vec<SavedDoc>>,
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

    let performance_metrics = performance.map(|perf| {
        let time_secs = (perf.duration_ms as f64) / 1000.0;
        let rate = if perf.tokens_per_second >= 100.0 {
            format!("{:.0} tok/s", perf.tokens_per_second)
        } else {
            format!("{:.1} tok/s", perf.tokens_per_second)
        };
        let duration = if time_secs >= 10.0 {
            format!("{:.0}s", time_secs)
        } else {
            format!("{:.1}s", time_secs)
        };
        (format!("{} tok", perf.token_count), rate, duration)
    });
    let metrics = performance_metrics;

    let save_payload = content.clone();
    let on_save = move |_| {
        let raw = save_payload.clone();
        if let Some(doc) = persist_markdown_doc(&raw) {
            saved_docs.with_mut(|docs| {
                docs.retain(|existing| existing.id != doc.id);
            });
            saved_docs.with_mut(|docs| docs.insert(0, doc));
        }
    };

    rsx! {
        if show_copy {
            div { class: "bubble-controls",
                if let Some((tokens, rate, duration)) = metrics.as_ref() {
                    div { class: "bubble-metrics",
                        span { class: "metric-pill", "{tokens}" }
                        span { class: "metric-pill", "{rate}" }
                        span { class: "metric-pill", "{duration}" }
                    }
                }
                div { class: "actions",
                    button { class: "action-btn", title: "Copy markdown", onclick: on_copy, "Copy" }
                    button { class: "action-btn", title: "Save", onclick: on_save, "Save" }
                }
            }
        }
        if is_streaming && content.is_empty() {
            div { class: "md", div { class: "shimmer-text", "Processing…" } }
        } else {
            div { class: "md", dangerous_inner_html: "{content_html}" }
        }
    }
}
