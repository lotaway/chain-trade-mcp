use crate::infrastructure::ethereum::EthereumClient;
use crate::interface::tools::tool_trait::Tool;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct PriceTool;

#[async_trait]
impl Tool for PriceTool {
    fn name(&self) -> &'static str {
        "get_token_price"
    }

    fn description(&self) -> &'static str {
        "Get the current price of a token in USDC"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "token_address": { "type": "string", "description": "The token contract address" }
            },
            "required": ["token_address"]
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "description": "Token price information in MCP content format",
            "properties": {
                "content": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "type": {"type": "string", "const": "text"},
                            "text": {
                                "type": "string",
                                "description": "JSON string containing price data with fields: token_address (string), price (string in USDC)"
                            }
                        }
                    }
                }
            }
        })
    }

    async fn execute(&self, client: &EthereumClient, args: &Value) -> Result<Value, String> {
        let token_address = args
            .get("token_address")
            .and_then(|v| v.as_str())
            .ok_or("Missing token_address")?;

        let price = client
            .get_token_price(token_address)
            .await
            .map_err(|e| e.to_string())?;

        let price_data = json!({
            "token_address": token_address,
            "price": price
        });

        Ok(json!({
            "content": [
                {
                    "type": "text",
                    "text": serde_json::to_string_pretty(&price_data).unwrap()
                }
            ]
        }))
    }
}
