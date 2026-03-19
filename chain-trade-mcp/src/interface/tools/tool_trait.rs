use crate::infrastructure::ethereum::EthereumClient;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn input_schema(&self) -> Value;

    /// Returns the schema describing the output format of this tool
    /// This helps AI models understand what data structure to expect
    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "description": "MCP response format",
            "properties": {
                "content": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "type": {"type": "string", "enum": ["text"]},
                            "text": {"type": "string", "description": "JSON string containing the result"}
                        }
                    }
                }
            }
        })
    }

    async fn execute(&self, client: &EthereumClient, args: &Value) -> Result<Value, String>;
}

pub struct ToolRegistry {
    tools: HashMap<&'static str, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name(), tool);
    }

    pub fn get_tool(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    pub fn list_tools(&self) -> Value {
        let tools: Vec<Value> = self
            .tools
            .values()
            .map(|tool| {
                json!({
                    "name": tool.name(),
                    "description": tool.description(),
                    "inputSchema": tool.input_schema(),
                    "outputSchema": tool.output_schema()
                })
            })
            .collect();

        json!({ "tools": tools })
    }
}
