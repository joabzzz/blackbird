use crate::views::shared::{SavedDoc, markdown_to_html};
use dioxus::prelude::*;
use time::{OffsetDateTime, UtcOffset, format_description::FormatItem, macros::format_description};

const DOC_DATE_FORMAT: &[FormatItem<'static>] =
    format_description!("[month repr:short] [day padding:zero], [year]");

#[component]
pub fn DocsView(saved_docs: Signal<Vec<SavedDoc>>) -> Element {
    let mut selected_doc_id = use_signal(|| Option::<String>::None);

    {
        let saved_docs = saved_docs;
        let mut selected_doc_id = selected_doc_id;
        use_effect(move || {
            let docs = saved_docs();
            let needs_selection = selected_doc_id.with(|selection| match selection {
                Some(id) => !docs.iter().any(|doc| &doc.id == id),
                None => !docs.is_empty(),
            });
            if needs_selection && let Some(first) = docs.first() {
                selected_doc_id.set(Some(first.id.clone()));
            }
        });
    }

    let docs = saved_docs();
    let current_selection = selected_doc_id();
    let selected_doc = current_selection
        .as_ref()
        .and_then(|id| docs.iter().find(|doc| &doc.id == id))
        .cloned();

    rsx! {
        div { class: "main-container",
            if docs.is_empty() {
                p { class: "text-muted", "No saved documents yet. Use the Save action to capture an assistant response." }
            } else {
                div { class: "doc-layout",
                    div { class: "doc-list",
                        for doc in docs.iter().cloned() {
                            div {
                                key: "{doc.id}",
                                class: format_args!(
                                    "doc-card {}",
                                    if current_selection
                                        .as_ref()
                                        .is_some_and(|id| id == &doc.id) { "active" } else { "" }
                                ),
                                role: "button",
                                tabindex: "0",
                                onclick: {
                                    let doc_id = doc.id.clone();
                                    move |_| selected_doc_id.set(Some(doc_id.clone()))
                                },
                                h3 { class: "doc-title", "{doc.title}" }
                                p { class: "doc-date", "Saved {doc_saved_date(doc.created_at)}" }
                            }
                        }
                    }
                    if let Some(doc) = selected_doc {
                        div { class: "doc-viewer",
                            h2 { class: "doc-viewer-title", "{doc.title}" }
                            p { class: "doc-viewer-date", "Saved {doc_saved_date(doc.created_at)}" }
                            div { class: "doc-viewer-content md", dangerous_inner_html: "{markdown_to_html(&doc.content)}" }
                        }
                    } else {
                        div { class: "doc-viewer empty",
                            p { class: "text-muted", "Select a document to view its contents." }
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
