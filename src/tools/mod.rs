/// Tools module for Blackbird AI function calling
///
/// This module contains all the tools that can be called by the AI agent
/// to interact with the application and perform actions.
pub mod calculator;
pub mod docs;
pub mod settings;

pub use calculator::CalculatorTool;
pub use docs::{GetDocsListTool, SearchDocsTool};
pub use settings::GetSettingTool;

/// Common error type for all tools
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("Invalid arguments: {0}")]
    InvalidArgs(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Not found: {0}")]
    NotFound(String),
}
