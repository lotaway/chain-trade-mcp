use crate::infrastructure::ethereum::EthereumClient;
use crate::interface::tools::tool_trait::Tool;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct SwapTool;

#[async_trait]
impl Tool for SwapTool {
    fn name(&self) -> &'static str {
        "swap_tokens"
    }

    fn description(&self) -> &'static str {
        "Simulate a token swap on Uniswap V3 to get estimated output and gas usage"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "from_token": { "type": "string", "description": "Address of the token to sell" },
                "to_token": { "type": "string", "description": "Address of the token to buy" },
                "amount": { "type": "string", "description": "Amount of from_token to sell (human readable, e.g. 1.5)" },
                "slippage": { "type": "number", "description": "Slippage tolerance (e.g. 0.005 for 0.5%)" }
            },
            "required": ["from_token", "to_token", "amount"]
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "description": "Swap simulation quote in MCP content format",
            "properties": {
                "content": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "type": {"type": "string", "const": "text"},
                            "text": {
                                "type": "string",
                                "description": "JSON string containing swap quote with fields: from_token (address), to_token (address), input_amount (string), estimated_output (string with slippage info), gas_estimate (string), simulation_success (boolean), error_message (optional string)"
                            }
                        }
                    }
                }
            }
        })
    }

    async fn execute(&self, client: &EthereumClient, args: &Value) -> Result<Value, String> {
        let from_token = args
            .get("from_token")
            .and_then(|v| v.as_str())
            .ok_or("Missing from_token")?;
        let to_token = args
            .get("to_token")
            .and_then(|v| v.as_str())
            .ok_or("Missing to_token")?;
        let amount = args
            .get("amount")
            .and_then(|v| v.as_str())
            .ok_or("Missing amount")?;
        let slippage = args.get("slippage").and_then(|v| v.as_f64());

        let quote = client
            .simulate_swap(from_token, to_token, amount, slippage)
            .await
            .map_err(|e| e.to_string())?;

        Ok(json!({
            "content": [
                {
                    "type": "text",
                    "text": serde_json::to_string_pretty(&quote).unwrap()
                }
            ]
        }))
    }
}
