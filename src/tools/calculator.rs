use super::ToolError;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Arguments for calculator tool
#[derive(Deserialize)]
pub struct CalculatorArgs {
    expression: String,
}

/// Calculator tool for evaluating mathematical expressions
///
/// This tool allows the AI to perform calculations using a safe math evaluator.
/// Supports basic arithmetic, common functions (sqrt, sin, cos, etc.), and constants (pi, e).
#[derive(Deserialize, Serialize)]
pub struct CalculatorTool;

impl Tool for CalculatorTool {
    const NAME: &'static str = "calculate";

    type Error = ToolError;
    type Args = CalculatorArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "calculate".to_string(),
            description: "Evaluate a mathematical expression. Supports basic arithmetic (+, -, *, /), exponents (^), parentheses, and common functions (sqrt, sin, cos, tan, ln, log, abs, etc.). Also supports constants like pi and e.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "Math expression to evaluate. Examples: '2 + 2 * 3', 'sqrt(16)', 'sin(pi/2)', '(5 + 3) * 2'"
                    }
                },
                "required": ["expression"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Use fasteval crate for safe math evaluation with custom function support
        let mut cb = |name: &str, args: Vec<f64>| -> Option<f64> {
            match (name, args.as_slice()) {
                // Square root
                ("sqrt", [x]) => Some(x.sqrt()),
                // Trigonometric functions (already built-in, but being explicit)
                ("sin", [x]) => Some(x.sin()),
                ("cos", [x]) => Some(x.cos()),
                ("tan", [x]) => Some(x.tan()),
                // Inverse trig
                ("asin", [x]) => Some(x.asin()),
                ("acos", [x]) => Some(x.acos()),
                ("atan", [x]) => Some(x.atan()),
                // Logarithms
                ("ln", [x]) => Some(x.ln()),
                ("log", [x]) => Some(x.log10()),
                ("log2", [x]) => Some(x.log2()),
                // Other functions
                ("abs", [x]) => Some(x.abs()),
                ("floor", [x]) => Some(x.floor()),
                ("ceil", [x]) => Some(x.ceil()),
                ("round", [x]) => Some(x.round()),
                // Constants
                ("pi", []) => Some(std::f64::consts::PI),
                ("e", []) => Some(std::f64::consts::E),
                _ => None,
            }
        };

        match fasteval::ez_eval(&args.expression, &mut cb) {
            Ok(result) => {
                // Format the result nicely
                if result.fract() == 0.0 && result.abs() < 1e10 {
                    // Integer result
                    Ok(format!("{}", result as i64))
                } else {
                    // Floating point result
                    Ok(format!("{}", result))
                }
            }
            Err(e) => Err(ToolError::ExecutionFailed(format!(
                "Cannot evaluate '{}': {}",
                args.expression, e
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_arithmetic() {
        let tool = CalculatorTool;
        let args = CalculatorArgs {
            expression: "2 + 2".to_string(),
        };

        let result = tool.call(args).await.unwrap();
        assert_eq!(result, "4");
    }

    #[tokio::test]
    async fn test_order_of_operations() {
        let tool = CalculatorTool;
        let args = CalculatorArgs {
            expression: "2 + 2 * 3".to_string(),
        };

        let result = tool.call(args).await.unwrap();
        assert_eq!(result, "8");
    }

    #[tokio::test]
    async fn test_sqrt() {
        let tool = CalculatorTool;
        let args = CalculatorArgs {
            expression: "sqrt(16)".to_string(),
        };

        let result = tool.call(args).await.unwrap();
        assert_eq!(result, "4");
    }

    #[tokio::test]
    async fn test_invalid_expression() {
        let tool = CalculatorTool;
        let args = CalculatorArgs {
            expression: "2 + * 3".to_string(),
        };

        let result = tool.call(args).await;
        assert!(result.is_err());
    }
}
