use crate::ai::{chat_reply_stream_poll, chat_reply_stream_start};
use crate::types::{ChatMessage, Role};
use crate::views::shared::{SavedDoc, markdown_to_html, persist_markdown_doc};
use dioxus::events::Key;
use dioxus::prelude::*;
use std::time::Instant;
use time::{OffsetDateTime, UtcOffset, format_description::FormatItem, macros::format_description};

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

#[component]
pub fn ChatView(saved_docs: Signal<Vec<SavedDoc>>, base_font_px: Signal<i32>) -> Element {
    let state = use_chat_state();

    rsx! {
        div { class: "main-container",
            ChatHistory { state, saved_docs }
            ChatComposer { state, base_font_px }
        }
    }
}

#[component]
fn ChatHistory(state: ChatState, saved_docs: Signal<Vec<SavedDoc>>) -> Element {
    let messages = state.messages();
    let performances = state.performances();
    let streaming_index = state.streaming_index();

    rsx! {
        div { class: "chat-wrap",
            div { id: "chat-list", class: "chat-list",
                for (index, message) in messages.into_iter().enumerate() {
                    ChatMessageRow {
                        index,
                        message,
                        metrics: performances.get(index).copied().flatten(),
                        streaming_index,
                        saved_docs,
                    }
                }
            }
        }
    }
}

#[component]
fn ChatMessageRow(
    index: usize,
    message: ChatMessage,
    metrics: Option<metrics::MessagePerformance>,
    streaming_index: Option<usize>,
    saved_docs: Signal<Vec<SavedDoc>>,
) -> Element {
    let role_class = match message.role {
        Role::User => "user",
        Role::Assistant => "assistant",
    };

    let is_streaming = is_streaming_message(streaming_index, index);
    let is_pending = is_pending_assistant(&message, streaming_index, index);
    let timestamp = format_message_timestamp(message.created_at);
    let metrics_label = metrics.map(|perf| metrics::summarize(&perf));

    let bubble: Element = match message.role {
        Role::Assistant => rsx!(AssistantBubble {
            content: message.content.clone(),
            show_copy: match streaming_index {
                Some(idx) => idx != index,
                None => true,
            },
            is_streaming,
            tags: message.tags.clone(),
            saved_docs,
        }),
        Role::User => rsx!("{message.content}"),
    };

    rsx! {
        div { class: format_args!("message-row {}", role_class),
            if matches!(message.role, Role::Assistant) {
                div { class: "avatar assistant", "C" }
            }
            div { class: "message-stack",
                if is_pending {
                    PendingMessageIndicator {}
                } else {
                    div { class: format_args!("bubble {}", role_class),
                        {bubble}
                    }
                }
                if let Some(ts) = timestamp {
                    div { class: format_args!(
                            "message-meta {}",
                            if matches!(message.role, Role::User) { "align-end" } else { "align-start" }
                        ),
                        span { class: "message-timestamp", "{ts}" }
                        if matches!(message.role, Role::Assistant) {
                            if let Some(perf) = metrics_label {
                                span { class: "message-metrics", "{perf}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PendingMessageIndicator() -> Element {
    rsx! {
        div { class: "shimmer-line",
            span { class: "shimmer-text", "Processing…" }
        }
    }
}

#[component]
fn ChatComposer(state: ChatState, base_font_px: Signal<i32>) -> Element {
    let chat = state;
    let mut font_size = base_font_px;

    let sending = chat.sending();
    let input_value = chat.input();
    let send_disabled = sending || input_value.trim().is_empty();

    rsx! {
        form { class: "composer no-divider",
            div { class: "composer-inner",
                div { class: "hstack", style: "gap: 0.5rem; width: 100%; align-items: flex-end;",
                    textarea {
                        rows: "1",
                        placeholder: "What can I help you with?",
                        value: "{input_value}",
                        oninput: move |ev| chat.set_input(ev.value()),
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
                                chat.submit_input();
                            }
                        },
                        disabled: sending,
                        autofocus: true,
                    }
                    button {
                        class: "btn btn-primary",
                        r#type: "button",
                        disabled: send_disabled,
                        onclick: move |_| chat.submit_input(),
                        "Send"
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

#[derive(Clone, Copy)]
struct ChatState {
    messages: Signal<Vec<ChatMessage>>,
    input: Signal<String>,
    sending: Signal<bool>,
    streaming_index: Signal<Option<usize>>,
    performances: Signal<Vec<Option<metrics::MessagePerformance>>>,
    processing_started_at: Signal<Option<Instant>>,
}

impl PartialEq for ChatState {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

fn use_chat_state() -> ChatState {
    ChatState {
        messages: use_signal(Vec::<ChatMessage>::new),
        input: use_signal(String::new),
        sending: use_signal(|| false),
        streaming_index: use_signal(|| None),
        performances: use_signal(Vec::<Option<metrics::MessagePerformance>>::new),
        processing_started_at: use_signal(|| None),
    }
}

impl ChatState {
    fn messages(&self) -> Vec<ChatMessage> {
        (self.messages)()
    }

    fn performances(&self) -> Vec<Option<metrics::MessagePerformance>> {
        (self.performances)()
    }

    fn streaming_index(&self) -> Option<usize> {
        (self.streaming_index)()
    }

    fn input(&self) -> String {
        (self.input)()
    }

    fn set_input(&self, value: String) {
        let mut input = self.input;
        input.set(value);
    }

    fn sending(&self) -> bool {
        (self.sending)()
    }

    fn set_sending(&self, value: bool) {
        let mut sending = self.sending;
        sending.set(value);
    }

    fn submit_input(&self) {
        let text = self.input();
        self.submit_text(text);
    }

    fn submit_text(&self, text: String) {
        let trimmed = text.trim();
        if trimmed.is_empty() || self.sending() {
            return;
        }

        self.push_user_message(trimmed);
        let mut performances = self.performances;
        performances.with_mut(|slots| slots.push(None));
        self.set_input(String::new());

        let conversation_snapshot = self.messages();

        self.set_sending(true);
        let assistant_index = self.insert_assistant_placeholder();
        performances.with_mut(|slots| slots.push(None));
        let mut streaming_index = self.streaming_index;
        streaming_index.set(Some(assistant_index));
        let mut started_at = self.processing_started_at;
        started_at.set(Some(Instant::now()));

        let mut server_messages = Vec::with_capacity(conversation_snapshot.len() + 1);
        server_messages.push(system_prompt_message());
        server_messages.extend(conversation_snapshot);

        self.spawn_stream(assistant_index, server_messages);
    }

    fn push_user_message(&self, content: &str) {
        let mut messages = self.messages;
        messages.with_mut(|msgs| {
            msgs.push(ChatMessage {
                role: Role::User,
                content: content.to_string(),
                created_at: Some(current_time()),
                tags: Vec::new(),
            });
        });
    }

    fn insert_assistant_placeholder(&self) -> usize {
        let mut index = 0;
        let mut messages = self.messages;
        messages.with_mut(|msgs| {
            index = msgs.len();
            msgs.push(ChatMessage {
                role: Role::Assistant,
                content: String::new(),
                created_at: Some(current_time()),
                tags: Vec::new(),
            });
        });
        index
    }

    fn spawn_stream(&self, index: usize, server_messages: Vec<ChatMessage>) {
        let state = *self;
        spawn(async move {
            let mut stream_failed = false;
            match chat_reply_stream_start(server_messages).await {
                Ok(stream_id) => loop {
                    match chat_reply_stream_poll(stream_id).await {
                        Ok((content, done)) => {
                            state.update_assistant_content(index, content);
                            if done {
                                break;
                            }
                        }
                        Err(err) => {
                            eprintln!("stream poll error: {}", err);
                            stream_failed = true;
                            break;
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(80)).await;
                },
                Err(err) => {
                    eprintln!("chat start error: {}", err);
                    stream_failed = true;
                }
            }

            if stream_failed {
                state.update_assistant_content(index, "Unable to generate a response.".to_string());
            }

            state.finalize_response(index);
            let mut streaming_index = state.streaming_index;
            streaming_index.set(None);
            state.set_sending(false);
        });
    }

    fn update_assistant_content(&self, index: usize, content: String) {
        let mut messages = self.messages;
        messages.with_mut(|msgs| {
            if let Some(message) = msgs.get_mut(index) {
                message.content = content;
            }
        });
    }

    fn finalize_response(&self, index: usize) {
        let started_at = (self.processing_started_at)();
        let content = self
            .messages
            .with(|msgs| msgs.get(index).map(|msg| msg.content.clone()));

        if let (Some(start), Some(content)) = (started_at, content)
            && let Some(enrichment) = metrics::enrich_response(Some(start), &content)
        {
            let mut performances = self.performances;
            performances.with_mut(|slots| {
                if let Some(slot) = slots.get_mut(index) {
                    *slot = Some(enrichment.performance);
                }
            });
            let mut messages = self.messages;
            messages.with_mut(|msgs| {
                if let Some(msg) = msgs.get_mut(index) {
                    msg.content = enrichment.content;
                    msg.tags = enrichment.tags;
                }
            });
        }

        let mut started_at_signal = self.processing_started_at;
        started_at_signal.set(None);
    }
}

fn system_prompt_message() -> ChatMessage {
    ChatMessage {
        role: Role::User,
        content: DOC_TAG_SYSTEM_PROMPT.to_string(),
        created_at: None,
        tags: Vec::new(),
    }
}

fn is_streaming_message(stream: Option<usize>, index: usize) -> bool {
    matches!(stream, Some(idx) if idx == index)
}

fn is_pending_assistant(msg: &ChatMessage, stream: Option<usize>, index: usize) -> bool {
    matches!(msg.role, Role::Assistant)
        && is_streaming_message(stream, index)
        && msg.content.is_empty()
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

mod metrics {
    use super::*;

    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct MessagePerformance {
        pub duration_ms: u128,
        pub token_count: usize,
        pub tokens_per_second: f64,
    }

    pub struct ResponseEnrichment {
        pub content: String,
        pub tags: Vec<String>,
        pub performance: MessagePerformance,
    }

    pub fn enrich_response(
        started_at: Option<Instant>,
        content: &str,
    ) -> Option<ResponseEnrichment> {
        let start = started_at?;
        let duration_ms = start.elapsed().as_millis();
        let token_count = estimate_token_count(content);
        let duration_secs = duration_ms as f64 / 1000.0;
        let tokens_per_second = if duration_secs > 0.0 {
            token_count as f64 / duration_secs
        } else {
            token_count as f64
        };

        let (clean_content, mut tags) = extract_doc_tags(content);
        if tags.is_empty() {
            tags = fallback_doc_tags(content);
        }

        Some(ResponseEnrichment {
            content: clean_content,
            tags,
            performance: MessagePerformance {
                duration_ms,
                token_count,
                tokens_per_second,
            },
        })
    }

    pub fn summarize(perf: &MessagePerformance) -> String {
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
        if tags.is_empty() {
            tags.push("Notes".to_string());
        }
        tags
    }
}
