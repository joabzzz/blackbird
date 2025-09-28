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

fn summarize_performance(perf: &MessagePerformance) -> String {
    let tokens = format!("{} tok", perf.token_count);
    let rate = if perf.tokens_per_second >= 100.0 {
        format!("{:.0} tok/s", perf.tokens_per_second)
    } else {
        format!("{:.1} tok/s", perf.tokens_per_second)
    };
    let time_secs = perf.duration_ms as f64 / 1000.0;
    let duration = if time_secs >= 10.0 {
        format!("{:.0}s", time_secs)
    } else {
        format!("{:.1}s", time_secs)
    };
    format!("{tokens} • {rate} • {duration}")
}

fn is_streaming_message(stream: Option<usize>, index: usize) -> bool {
    matches!(stream, Some(idx) if idx == index)
}

fn is_pending_assistant(msg: &ChatMessage, stream: Option<usize>, index: usize) -> bool {
    matches!(msg.role, Role::Assistant)
        && is_streaming_message(stream, index)
        && msg.content.is_empty()
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

const DOC_TAG_SYSTEM_PROMPT: &str = r#"
You are Blackbird, an AI writing partner that helps users draft and organize documents.
When you respond, always include a final line in the exact format [[doc_tags: tag1, tag2, ...]].
Choose 1-3 high-level document category tags that describe the type of document the user would file this as (e.g. Proposal, Meeting Notes, Product Brief, Technical Spec, Launch Plan, Research Summary, Checklist, FAQ).
Avoid generic words from the response body; focus on the document intent.
Do not include any additional text on the tag line beyond the brackets and comma-separated tags.
If you cannot infer a type, use [[doc_tags: Notes]].
"#;

fn extract_doc_tags(content: &str) -> (String, Vec<String>) {
    if let Some(start) = content.rfind("[[doc_tags:")
        && let Some(end) = content[start..].find("]]")
    {
        let raw_tags = &content[start + "[[doc_tags:".len()..start + end];
        let cleaned_content = content[..start].trim_end().to_string();
        let tags = raw_tags
            .split(',')
            .filter_map(normalize_tag)
            .collect::<Vec<_>>();
        return (cleaned_content, tags);
    }

    (content.to_string(), Vec::new())
}

fn normalize_tag(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut chars = trimmed.chars();
    let first = chars.next()?;
    let mut tag = String::new();
    tag.extend(first.to_uppercase());
    let remainder = chars.as_str().to_lowercase();
    tag.push_str(&remainder);
    Some(tag)
}

fn fallback_doc_tags(content: &str) -> Vec<String> {
    let text = content.to_lowercase();
    let mut tags = Vec::new();
    {
        let mut add_tag = |label: &str| {
            if !tags.iter().any(|existing| existing == label) {
                tags.push(label.to_string());
            }
        };

        if text.contains("meeting") || text.contains("standup") || text.contains("notes") {
            add_tag("Meeting Notes");
        }
        if text.contains("plan") || text.contains("roadmap") || text.contains("timeline") {
            add_tag("Project Plan");
        }
        if text.contains("proposal") || text.contains("pitch") {
            add_tag("Proposal");
        }
        if text.contains("requirements") || text.contains("spec") || text.contains("specification")
        {
            add_tag("Technical Spec");
        }
        if text.contains("summary") || text.contains("overview") {
            add_tag("Summary");
        }
        if text.contains("launch") || text.contains("release") {
            add_tag("Launch Plan");
        }
        if text.contains("faq") || text.contains("question") {
            add_tag("FAQ");
        }
        if text.contains("checklist") || text.contains("steps") {
            add_tag("Checklist");
        }
        if text.contains("brief") {
            add_tag("Product Brief");
        }
    }
    if tags.is_empty() {
        tags.push("Notes".to_string());
    }
    tags
}

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
    mut messages: Signal<Vec<ChatMessage>>,
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
            let (clean_content, mut tags) = extract_doc_tags(&content);
            if tags.is_empty() {
                tags = fallback_doc_tags(&content);
            }
            performances.with_mut(|slots| {
                if let Some(slot) = slots.get_mut(index) {
                    *slot = Some(MessagePerformance {
                        duration_ms,
                        token_count,
                        tokens_per_second,
                    });
                }
            });
            messages.with_mut(|msgs| {
                if let Some(msg) = msgs.get_mut(index) {
                    msg.content = clean_content;
                    msg.tags = tags;
                }
            });
        }
    }
    processing_started_at.set(None);
}

#[component]
pub fn ChatView(saved_docs: Signal<Vec<SavedDoc>>, base_font_px: Signal<i32>) -> Element {
    let messages = use_signal(Vec::<ChatMessage>::new);
    let mut input = use_signal(String::new);
    let sending = use_signal(|| false);
    let streaming_index = use_signal(|| Option::<usize>::None);
    let performances = use_signal(Vec::<Option<MessagePerformance>>::new);
    let processing_started_at = use_signal(|| Option::<Instant>::None);
    let scroll_to_bottom = || {};

    let mut send_message = {
        let mut messages = messages;
        let mut performances = performances;
        let mut streaming_index = streaming_index;
        let mut processing_started_at = processing_started_at;
        let mut sending_signal = sending;
        let mut input_signal = input;
        move |text: String| {
            let trimmed = text.trim();
            if trimmed.is_empty() || sending_signal() {
                return;
            }

            messages.with_mut(|msgs| {
                msgs.push(ChatMessage {
                    role: Role::User,
                    content: trimmed.to_string(),
                    created_at: Some(current_time()),
                    tags: Vec::new(),
                });
            });
            performances.with_mut(|slots| slots.push(None));
            scroll_to_bottom();
            input_signal.set(String::new());

            let conversation_snapshot = messages();

            sending_signal.set(true);
            let mut inserted_index = 0;
            messages.with_mut(|msgs| {
                inserted_index = msgs.len();
                msgs.push(ChatMessage {
                    role: Role::Assistant,
                    content: String::new(),
                    created_at: Some(current_time()),
                    tags: Vec::new(),
                });
            });
            performances.with_mut(|slots| slots.push(None));
            streaming_index.set(Some(inserted_index));
            processing_started_at.set(Some(Instant::now()));

            let mut server_messages = Vec::with_capacity(conversation_snapshot.len() + 1);
            server_messages.push(ChatMessage {
                role: Role::User,
                content: DOC_TAG_SYSTEM_PROMPT.to_string(),
                created_at: None,
                tags: Vec::new(),
            });
            server_messages.extend(conversation_snapshot);

            let performance_signal = performances;
            let metrics_messages = messages;
            let metrics_started_at = processing_started_at;
            spawn(async move {
                match chat_reply_stream_start(server_messages).await {
                    Ok(stream_id) => loop {
                        match chat_reply_stream_poll(stream_id).await {
                            Ok((content, done)) => {
                                messages.with_mut(|msgs| {
                                    if let Some(msg) = msgs.get_mut(inserted_index) {
                                        msg.content = content.clone();
                                    }
                                });
                                scroll_to_bottom();
                                if done {
                                    break;
                                }
                            }
                            Err(err) => {
                                eprintln!("stream poll error: {}", err);
                                break;
                            }
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(80)).await;
                    },
                    Err(err) => eprintln!("chat start error: {}", err),
                }
                finalize_message_metrics(
                    performance_signal,
                    metrics_messages,
                    metrics_started_at,
                    inserted_index,
                );
                streaming_index.set(None);
                sending_signal.set(false);
            });
        }
    };

    let messages_snapshot = messages();
    let performance_snapshot = performances();
    let current_stream = streaming_index();

    rsx! {
        div { class: "main-container",
            div { class: "chat-wrap",
                div { id: "chat-list", class: "chat-list",
                    for (i, msg) in messages_snapshot.iter().enumerate() {
                        div { class: format_args!("message-row {}", match msg.role { Role::User => "user", Role::Assistant => "assistant" }),
                            if matches!(msg.role, Role::Assistant) { div { class: "avatar assistant", "C" } }
                            div { class: "message-stack",
                                if is_pending_assistant(msg, current_stream, i) {
                                    div { class: "shimmer-line",
                                        span { class: "shimmer-text", "Processing…" }
                                    }
                                } else {
                                    div { class: format_args!(
                                            "bubble {}",
                                            match msg.role { Role::User => "user", Role::Assistant => "assistant" },
                                        ),
                                        if matches!(msg.role, Role::Assistant) {
                                            AssistantBubble {
                                                content: msg.content.clone(),
                                                show_copy: match current_stream { Some(idx) => idx != i, None => true },
                                                is_streaming: is_streaming_message(current_stream, i),
                                                tags: msg.tags.clone(),
                                                saved_docs,
                                            }
                                        } else { "{msg.content}" }
                                    }
                                }
                                if let Some(ts) = format_message_timestamp(msg.created_at) {
                                    div { class: format_args!(
                                            "message-meta {}",
                                            match msg.role { Role::User => "align-end", Role::Assistant => "align-start" }
                                        ),
                                        span { class: "message-timestamp", "{ts}" }
                                        if matches!(msg.role, Role::Assistant) {
                                            if let Some(perf) = performance_snapshot.get(i).copied().flatten() {
                                                span { class: "message-metrics", "{summarize_performance(&perf)}" }
                                            }
                                        }
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
                                    let text = input();
                                    send_message(text);
                                }
                            },
                            disabled: sending(), autofocus: true,
                        }
                        button {
                            class: "btn btn-primary", r#type: "button",
                            disabled: sending() || input().trim().is_empty(),
                            onclick: move |_| {
                                let text = input();
                                send_message(text);
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
    is_streaming: bool,
    tags: Vec<String>,
    saved_docs: Signal<Vec<SavedDoc>>,
) -> Element {
    let content_html = markdown_to_html(&content);
    let copy_payload = content.clone();
    let display_tags = tags.clone();
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

    let save_payload = content.clone();
    let save_tags = tags.clone();
    let on_save = move |_| {
        let raw = save_payload.clone();
        if let Some(doc) = persist_markdown_doc(&raw, Some(&save_tags)) {
            saved_docs.with_mut(|docs| {
                docs.retain(|existing| existing.id != doc.id);
            });
            saved_docs.with_mut(|docs| docs.insert(0, doc));
        }
    };

    rsx! {
        if show_copy {
            div { class: "bubble-controls",
                div { class: "actions",
                    button { class: "action-btn", title: "Copy markdown", onclick: on_copy, "Copy" }
                    button { class: "action-btn", title: "Save", onclick: on_save, "Save" }
                }
            }
        }
        if !display_tags.is_empty() {
            div { class: "bubble-tags",
                for tag in display_tags.iter() {
                    span { class: "tag-pill tag-pill-compact", "{tag}" }
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
