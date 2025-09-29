use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
    Octane,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
    #[serde(skip)]
    /// Only used for local timestamp display; upstream APIs ignore this field.
    pub created_at: Option<OffsetDateTime>,
    #[serde(skip)]
    /// Local-only tags captured for saved documents, not part of LLM requests.
    pub tags: Vec<String>,
}
