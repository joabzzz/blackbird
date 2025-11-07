use super::ToolError;
use crate::types::ThemeMode;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{Arc, RwLock};

/// Arguments for getting settings
#[derive(Deserialize)]
pub struct GetSettingArgs {
    setting: String,
}

/// Tool for retrieving application settings
#[derive(Clone)]
pub struct GetSettingTool {
    pub theme: Arc<RwLock<ThemeMode>>,
    pub base_font_px: Arc<RwLock<i32>>,
}

impl GetSettingTool {
    pub fn new(theme: Arc<RwLock<ThemeMode>>, base_font_px: Arc<RwLock<i32>>) -> Self {
        Self {
            theme,
            base_font_px,
        }
    }
}

impl Serialize for GetSettingTool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_unit()
    }
}

impl<'de> Deserialize<'de> for GetSettingTool {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Err(serde::de::Error::custom(
            "GetSettingTool cannot be deserialized",
        ))
    }
}

impl Tool for GetSettingTool {
    const NAME: &'static str = "get_setting";

    type Error = ToolError;
    type Args = GetSettingArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "get_setting".to_string(),
            description: "Get current application settings like theme mode and font size."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "setting": {
                        "type": "string",
                        "enum": ["theme", "font_size", "all"],
                        "description": "Which setting to retrieve: 'theme' for current theme mode, 'font_size' for base font size in pixels, or 'all' for all settings"
                    }
                },
                "required": ["setting"]
            }),
        }
    }

    fn call(
        &self,
        args: Self::Args,
    ) -> impl std::future::Future<Output = Result<Self::Output, Self::Error>> + Send {
        let theme = self.theme.clone();
        let font_size = self.base_font_px.clone();

        async move {
            match args.setting.as_str() {
                "theme" => {
                    let theme = theme.read().map_err(|e| {
                        ToolError::ExecutionFailed(format!("Failed to read theme: {}", e))
                    })?;
                    Ok(format!("{:?}", *theme))
                }
                "font_size" => {
                    let font_size = font_size.read().map_err(|e| {
                        ToolError::ExecutionFailed(format!("Failed to read font size: {}", e))
                    })?;
                    Ok(format!("{}px", *font_size))
                }
                "all" => {
                    let theme_value = theme.read().map_err(|e| {
                        ToolError::ExecutionFailed(format!("Failed to read theme: {}", e))
                    })?;
                    let font_size_value = font_size.read().map_err(|e| {
                        ToolError::ExecutionFailed(format!("Failed to read font size: {}", e))
                    })?;

                    Ok(serde_json::to_string_pretty(&json!({
                        "theme": format!("{:?}", *theme_value),
                        "font_size": format!("{}px", *font_size_value),
                    }))
                    .unwrap())
                }
                _ => Err(ToolError::InvalidArgs(format!(
                    "Unknown setting: '{}'. Valid options: 'theme', 'font_size', 'all'",
                    args.setting
                ))),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_theme() {
        let theme = Arc::new(RwLock::new(ThemeMode::Dark));
        let font = Arc::new(RwLock::new(16));
        let tool = GetSettingTool::new(theme, font);

        let args = GetSettingArgs {
            setting: "theme".to_string(),
        };

        let result = tool.call(args).await.unwrap();
        assert_eq!(result, "Dark");
    }

    #[tokio::test]
    async fn test_get_font_size() {
        let theme = Arc::new(RwLock::new(ThemeMode::Dark));
        let font = Arc::new(RwLock::new(18));
        let tool = GetSettingTool::new(theme, font);

        let args = GetSettingArgs {
            setting: "font_size".to_string(),
        };

        let result = tool.call(args).await.unwrap();
        assert_eq!(result, "18px");
    }

    #[tokio::test]
    async fn test_invalid_setting() {
        let theme = Arc::new(RwLock::new(ThemeMode::Dark));
        let font = Arc::new(RwLock::new(16));
        let tool = GetSettingTool::new(theme, font);

        let args = GetSettingArgs {
            setting: "invalid".to_string(),
        };

        let result = tool.call(args).await;
        assert!(result.is_err());
    }
}
