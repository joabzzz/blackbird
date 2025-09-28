use crate::views::shared::{SavedDoc, markdown_to_html};
use dioxus::{
    events::{FormEvent, Key, KeyboardEvent},
    prelude::*,
};
use time::{OffsetDateTime, UtcOffset, format_description::FormatItem, macros::format_description};

const DOC_DATE_FORMAT: &[FormatItem<'static>] =
    format_description!("[month repr:short] [day padding:zero], [year]");

#[derive(Clone, Copy, PartialEq, Eq)]
enum DocSort {
    Newest,
    Oldest,
    Title,
}

#[component]
pub fn DocsView(saved_docs: Signal<Vec<SavedDoc>>) -> Element {
    let mut selected_doc_id = use_signal(|| Option::<String>::None);
    let mut sort_mode = use_signal(|| DocSort::Newest);
    let mut tag_filter = use_signal(|| Option::<String>::None);

    {
        let saved_docs = saved_docs;
        let mut selected_doc_id = selected_doc_id;
        use_effect(move || {
            let docs = saved_docs();
            let should_clear = selected_doc_id.with(|selection| {
                selection
                    .as_ref()
                    .map(|id| !docs.iter().any(|doc| &doc.id == id))
                    .unwrap_or(false)
            });
            if should_clear {
                selected_doc_id.set(None);
            }
        });
    }

    let docs = saved_docs();

    let mut all_tags: Vec<String> = docs
        .iter()
        .flat_map(|doc| doc.tags.iter().cloned())
        .collect();
    all_tags.sort_unstable();
    all_tags.dedup();

    let filter_tag = tag_filter();
    let mut display_docs = docs.clone();
    if let Some(tag) = filter_tag.as_ref() {
        let tag_lower = tag.to_lowercase();
        display_docs.retain(|doc| {
            doc.tags
                .iter()
                .any(|candidate| candidate.to_lowercase() == tag_lower)
        });
    }

    match sort_mode() {
        DocSort::Newest => display_docs.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
        DocSort::Oldest => display_docs.sort_by(|a, b| a.created_at.cmp(&b.created_at)),
        DocSort::Title => {
            display_docs.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
        }
    }

    let current_selection = selected_doc_id();
    let selected_doc = current_selection
        .as_ref()
        .and_then(|id| display_docs.iter().find(|doc| &doc.id == id))
        .cloned();

    rsx! {
        div { class: "main-container",
            if docs.is_empty() {
                p { class: "text-muted", "No saved documents yet. Use the Save action to capture an assistant response." }
            } else {
                div { class: "doc-controls",
                    div { class: "doc-control-group",
                        label { for: "doc-sort", class: "control-label", "Sort" }
                        select {
                            id: "doc-sort",
                            value: match sort_mode() { DocSort::Newest => "newest", DocSort::Oldest => "oldest", DocSort::Title => "title" },
                            onchange: move |evt: FormEvent| {
                                let mode = match evt.value().as_str() {
                                    "oldest" => DocSort::Oldest,
                                    "title" => DocSort::Title,
                                    _ => DocSort::Newest,
                                };
                                sort_mode.set(mode);
                            },
                            option { value: "newest", "Newest" }
                            option { value: "oldest", "Oldest" }
                            option { value: "title", "Title" }
                        }
                    }
                    div { class: "doc-control-group",
                        label { for: "doc-tag", class: "control-label", "Filter" }
                        select {
                            id: "doc-tag",
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
                if display_docs.is_empty() {
                    div { class: "doc-empty",
                        p { class: "text-muted", "No documents match the selected filters." }
                    }
                } else {
                    div { class: "doc-table",
                        div { class: "doc-table-header",
                            span { class: "doc-col-title", "Title" }
                            span { class: "doc-col-tags", "Tags" }
                            span { class: "doc-col-date", "Saved" }
                        }
                        div { class: "doc-table-body",
                            for doc in display_docs.iter().cloned() {
                                div {
                                    key: "{doc.id}",
                                    class: format_args!(
                                        "doc-row {}",
                                        if selected_doc
                                            .as_ref()
                                            .is_some_and(|selected| selected.id == doc.id) { "active" } else { "" }
                                    ),
                                    role: "button",
                                    tabindex: "0",
                                    onclick: {
                                        let doc_id = doc.id.clone();
                                        move |_| selected_doc_id.set(Some(doc_id.clone()))
                                    },
                                    onkeydown: {
                                        let doc_id = doc.id.clone();
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
                                                selected_doc_id.set(Some(doc_id.clone()));
                                            }
                                        }
                                    },
                                    span { class: "doc-row-title", "{doc.title}" }
                                    div { class: "doc-row-tags",
                                        if doc.tags.is_empty() {
                                            span { class: "tag-pill tag-pill-muted", "No tags" }
                                        } else {
                                            for tag in doc.tags.iter() {
                                                span { class: "tag-pill tag-pill-compact", "{tag}" }
                                            }
                                        }
                                    }
                                    span { class: "doc-row-date", "{doc_saved_date(doc.created_at)}" }
                                }
                            }
                        }
                    }
                }
                if let Some(doc) = selected_doc {
                    div { class: "doc-overlay", role: "dialog", aria_modal: "true",
                        onclick: move |_| selected_doc_id.set(None),
                        div {
                            class: "doc-overlay-panel",
                            onclick: move |evt| evt.stop_propagation(),
                            header { class: "doc-overlay-header",
                                h2 { class: "doc-viewer-title", "{doc.title}" }
                                div { class: "doc-overlay-actions",
                                    button {
                                        class: "doc-overlay-close btn-ghost",
                                        r#type: "button",
                                        onclick: move |_| selected_doc_id.set(None),
                                        aria_label: "Close document",
                                        dangerous_inner_html: "&times;"
                                    }
                                }
                            }
                            if !doc.tags.is_empty() {
                                div { class: "doc-overlay-tags",
                                    for tag in doc.tags.iter() {
                                        span { class: "tag-pill tag-pill-compact", "{tag}" }
                                    }
                                }
                            }
                            p { class: "doc-viewer-date", "Saved {doc_saved_date(doc.created_at)}" }
                            div { class: "doc-viewer-content md", dangerous_inner_html: "{markdown_to_html(&doc.content)}" }
                        }
                    }
                }
            }
        }
    }
}

fn doc_saved_date(timestamp: u64) -> String {
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
        .format(DOC_DATE_FORMAT)
        .unwrap_or_else(|_| "Unknown date".to_string())
}
