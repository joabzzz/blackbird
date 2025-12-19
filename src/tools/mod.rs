/// Tools module for Blackbird AI function calling
///
/// This module contains all the tools that can be called by the AI agent
/// to interact with the application and perform actions.
pub mod apps;
pub mod calculator;
pub mod settings;

pub use apps::{GetAppsListTool, SearchAppsTool};
pub use calculator::CalculatorTool;
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
