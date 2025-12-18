use super::ToolError;
use crate::views::shared::SavedApp;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{Arc, RwLock};

// ============================================
// SEARCH APPS TOOL
// ============================================

/// Arguments for searching apps
#[derive(Deserialize)]
pub struct SearchAppsArgs {
    query: String,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    5
}

/// Tool for searching through saved apps
#[derive(Clone)]
pub struct SearchAppsTool {
    pub apps: Arc<RwLock<Vec<SavedApp>>>,
}

impl SearchAppsTool {
    pub fn new(apps: Arc<RwLock<Vec<SavedApp>>>) -> Self {
        Self { apps }
    }
}

impl Serialize for SearchAppsTool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_unit()
    }
}

impl<'de> Deserialize<'de> for SearchAppsTool {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Err(serde::de::Error::custom(
            "SearchAppsTool cannot be deserialized",
        ))
    }
}

impl Tool for SearchAppsTool {
    const NAME: &'static str = "search_apps";

    type Error = ToolError;
    type Args = SearchAppsArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "search_apps".to_string(),
            description: "Search through saved apps for keywords or phrases. Returns matching apps with relevant snippets.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query to find in app titles and content"
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
        let apps = self.apps.clone();

        async move {
            let apps = apps
                .read()
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read apps: {}", e)))?;

            let query_lower = args.query.to_lowercase();
            let limit = args.limit.min(20);

            let mut results = Vec::new();

            for app in apps.iter() {
                let title_match = app.title.to_lowercase().contains(&query_lower);
                let content_match = app.content.to_lowercase().contains(&query_lower);

                if title_match || content_match {
                    let snippet = if content_match {
                        extract_snippet(&app.content, &args.query, 100)
                    } else {
                        app.content.chars().take(100).collect::<String>()
                    };

                    results.push(json!({
                        "id": app.id,
                        "title": app.title,
                        "tags": app.tags,
                        "snippet": snippet,
                        "created_at": app.created_at,
                    }));

                    if results.len() >= limit {
                        break;
                    }
                }
            }

            if results.is_empty() {
                Ok(format!("No apps found matching '{}'", args.query))
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
// GET APPS LIST TOOL
// ============================================

#[derive(Deserialize)]
pub struct GetAppsListArgs {
    #[serde(default)]
    tag_filter: Option<String>,
    #[serde(default = "default_list_limit")]
    limit: usize,
}

fn default_list_limit() -> usize {
    10
}

#[derive(Clone)]
pub struct GetAppsListTool {
    pub apps: Arc<RwLock<Vec<SavedApp>>>,
}

impl GetAppsListTool {
    pub fn new(apps: Arc<RwLock<Vec<SavedApp>>>) -> Self {
        Self { apps }
    }
}

impl Serialize for GetAppsListTool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_unit()
    }
}

impl<'de> Deserialize<'de> for GetAppsListTool {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Err(serde::de::Error::custom(
            "GetAppsListTool cannot be deserialized",
        ))
    }
}

impl Tool for GetAppsListTool {
    const NAME: &'static str = "get_apps_list";

    type Error = ToolError;
    type Args = GetAppsListArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "get_apps_list".to_string(),
            description: "Get a list of saved apps with optional tag filtering. Returns app metadata including titles, tags, and creation dates.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "tag_filter": {
                        "type": "string",
                        "description": "Optional tag to filter apps by (case-insensitive)"
                    },
                    "limit": {
                        "type": "number",
                        "description": "Maximum number of apps to return (default: 10, max: 50)"
                    }
                }
            }),
        }
    }

    fn call(
        &self,
        args: Self::Args,
    ) -> impl std::future::Future<Output = Result<Self::Output, Self::Error>> + Send {
        let apps = self.apps.clone();

        async move {
            let apps = apps
                .read()
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read apps: {}", e)))?;

            let limit = args.limit.min(50);

            let filtered: Vec<_> = apps
                .iter()
                .filter(|app| {
                    if let Some(ref tag) = args.tag_filter {
                        app.tags.iter().any(|t| t.eq_ignore_ascii_case(tag))
                    } else {
                        true
                    }
                })
                .take(limit)
                .map(|app| {
                    json!({
                        "id": app.id,
                        "title": app.title,
                        "tags": app.tags,
                        "created_at": app.created_at,
                        "preview": app.content.chars().take(100).collect::<String>(),
                    })
                })
                .collect();

            if filtered.is_empty() {
                if let Some(tag) = args.tag_filter {
                    Ok(format!("No apps found with tag '{}'", tag))
                } else {
                    Ok("No apps found".to_string())
                }
            } else {
                Ok(serde_json::to_string_pretty(&json!({
                    "count": filtered.len(),
                    "filter": args.tag_filter,
                    "apps": filtered,
                }))
                .unwrap())
            }
        }
    }
}
