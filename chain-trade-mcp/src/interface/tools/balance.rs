use crate::infrastructure::ethereum::EthereumClient;
use crate::interface::tools::tool_trait::Tool;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct BalanceTool;

#[async_trait]
impl Tool for BalanceTool {
    fn name(&self) -> &'static str {
        "get_balance"
    }

    fn description(&self) -> &'static str {
        "Get the balance of ETH or an ERC20 token for a specific address"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "address": { "type": "string", "description": "The wallet address to check" },
                "token_address": { "type": "string", "description": "Optional ERC20 token contract address. If omitted, returns ETH balance." }
            },
            "required": ["address"]
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "description": "Balance information wrapped in MCP content format",
            "properties": {
                "content": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "type": {"type": "string", "const": "text"},
                            "text": {
                                "type": "string",
                                "description": "JSON string containing balance data with fields: token (optional object with address/symbol/decimals), amount (wei/smallest unit as string), formatted (human-readable amount as string)"
                            }
                        }
                    }
                }
            }
        })
    }

    async fn execute(&self, client: &EthereumClient, args: &Value) -> Result<Value, String> {
        let address = args
            .get("address")
            .and_then(|v| v.as_str())
            .ok_or("Missing address")?;
        let token_address = args.get("token_address").and_then(|v| v.as_str());

        let balance = client
            .get_balance(address, token_address)
            .await
            .map_err(|e| e.to_string())?;

        Ok(json!({
            "content": [
                {
                    "type": "text",
                    "text": serde_json::to_string_pretty(&balance).unwrap()
                }
            ]
        }))
    }
}
