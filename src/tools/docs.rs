use super::ToolError;
use crate::views::shared::SavedDoc;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{Arc, RwLock};

// ============================================
// SEARCH DOCS TOOL
// ============================================

/// Arguments for searching documents
#[derive(Deserialize)]
pub struct SearchDocsArgs {
    query: String,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    5
}

/// Tool for searching through saved documents
#[derive(Clone)]
pub struct SearchDocsTool {
    pub docs: Arc<RwLock<Vec<SavedDoc>>>,
}

impl SearchDocsTool {
    pub fn new(docs: Arc<RwLock<Vec<SavedDoc>>>) -> Self {
        Self { docs }
    }
}

impl Serialize for SearchDocsTool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_unit()
    }
}

impl<'de> Deserialize<'de> for SearchDocsTool {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Err(serde::de::Error::custom(
            "SearchDocsTool cannot be deserialized",
        ))
    }
}

impl Tool for SearchDocsTool {
    const NAME: &'static str = "search_docs";

    type Error = ToolError;
    type Args = SearchDocsArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "search_docs".to_string(),
            description: "Search through saved documents for keywords or phrases. Returns matching documents with relevant snippets showing where the query appears.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query to find in document titles and content"
                    },
                    "limit": {
                        "type": "number",
                        "description": "Maximum number of results to return (default: 5, max: 20)"
                    }
                },
                "required": ["query"]
            }),
        }
    }

    fn call(
        &self,
        args: Self::Args,
    ) -> impl std::future::Future<Output = Result<Self::Output, Self::Error>> + Send {
        let docs = self.docs.clone();

        async move {
            let docs = docs
                .read()
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read docs: {}", e)))?;

            let query_lower = args.query.to_lowercase();
            let limit = args.limit.min(20);

            let mut results = Vec::new();

            for doc in docs.iter() {
                let title_match = doc.title.to_lowercase().contains(&query_lower);
                let content_match = doc.content.to_lowercase().contains(&query_lower);

                if title_match || content_match {
                    let snippet = if content_match {
                        extract_snippet(&doc.content, &args.query, 100)
                    } else {
                        doc.content.chars().take(100).collect::<String>()
                    };

                    results.push(json!({
                        "id": doc.id,
                        "title": doc.title,
                        "tags": doc.tags,
                        "snippet": snippet,
                        "created_at": doc.created_at,
                    }));

                    if results.len() >= limit {
                        break;
                    }
                }
            }

            if results.is_empty() {
                Ok(format!("No documents found matching '{}'", args.query))
            } else {
                Ok(serde_json::to_string_pretty(&json!({
                    "query": args.query,
                    "count": results.len(),
                    "results": results,
                }))
                .unwrap())
            }
        }
    }
}

fn extract_snippet(content: &str, query: &str, context_chars: usize) -> String {
    let content_lower = content.to_lowercase();
    let query_lower = query.to_lowercase();

    if let Some(pos) = content_lower.find(&query_lower) {
        let start = pos.saturating_sub(context_chars / 2);
        let end = (pos + query.len() + context_chars / 2).min(content.len());

        let mut snippet = content[start..end].to_string();

        if start > 0 {
            snippet = format!("...{}", snippet);
        }
        if end < content.len() {
            snippet = format!("{}...", snippet);
        }

        snippet
    } else {
        content.chars().take(context_chars).collect::<String>()
    }
}

// ============================================
// GET DOCS LIST TOOL
// ============================================

#[derive(Deserialize)]
pub struct GetDocsListArgs {
    #[serde(default)]
    tag_filter: Option<String>,
    #[serde(default = "default_list_limit")]
    limit: usize,
}

fn default_list_limit() -> usize {
    10
}

#[derive(Clone)]
pub struct GetDocsListTool {
    pub docs: Arc<RwLock<Vec<SavedDoc>>>,
}

impl GetDocsListTool {
    pub fn new(docs: Arc<RwLock<Vec<SavedDoc>>>) -> Self {
        Self { docs }
    }
}

impl Serialize for GetDocsListTool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_unit()
    }
}

impl<'de> Deserialize<'de> for GetDocsListTool {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Err(serde::de::Error::custom(
            "GetDocsListTool cannot be deserialized",
        ))
    }
}

impl Tool for GetDocsListTool {
    const NAME: &'static str = "get_docs_list";

    type Error = ToolError;
    type Args = GetDocsListArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "get_docs_list".to_string(),
            description: "Get a list of saved documents with optional tag filtering. Returns document metadata including titles, tags, and creation dates.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "tag_filter": {
                        "type": "string",
                        "description": "Optional tag to filter documents by (case-insensitive)"
                    },
                    "limit": {
                        "type": "number",
                        "description": "Maximum number of documents to return (default: 10, max: 50)"
                    }
                }
            }),
        }
    }

    fn call(
        &self,
        args: Self::Args,
    ) -> impl std::future::Future<Output = Result<Self::Output, Self::Error>> + Send {
        let docs = self.docs.clone();

        async move {
            let docs = docs
                .read()
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read docs: {}", e)))?;

            let limit = args.limit.min(50);

            let filtered: Vec<_> = docs
                .iter()
                .filter(|doc| {
                    if let Some(ref tag) = args.tag_filter {
                        doc.tags.iter().any(|t| t.eq_ignore_ascii_case(tag))
                    } else {
                        true
                    }
                })
                .take(limit)
                .map(|doc| {
                    json!({
                        "id": doc.id,
                        "title": doc.title,
                        "tags": doc.tags,
                        "created_at": doc.created_at,
                        "preview": doc.content.chars().take(100).collect::<String>(),
                    })
                })
                .collect();

            if filtered.is_empty() {
                if let Some(tag) = args.tag_filter {
                    Ok(format!("No documents found with tag '{}'", tag))
                } else {
                    Ok("No documents found".to_string())
                }
            } else {
                Ok(serde_json::to_string_pretty(&json!({
                    "count": filtered.len(),
                    "filter": args.tag_filter,
                    "documents": filtered,
                }))
                .unwrap())
            }
        }
    }
}
