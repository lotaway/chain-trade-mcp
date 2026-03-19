use crate::infrastructure::ethereum::EthereumClient;
use crate::infrastructure::swap_executor::SwapExecutor;
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
        "Execute a token swap on Uniswap V3. Set execute=true to perform real swap."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "from_token": { "type": "string", "description": "Address of the token to sell" },
                "to_token": { "type": "string", "description": "Address of the token to buy" },
                "amount": { "type": "string", "description": "Amount of from_token to sell (human readable, e.g. 1.5)" },
                "slippage": { "type": "number", "description": "Slippage tolerance (e.g. 0.005 for 0.5%)" },
                "max_spend": { "type": "string", "description": "Maximum amount to spend (optional)" },
                "min_receive": { "type": "string", "description": "Minimum amount to receive (optional)" },
                "execute": { "type": "boolean", "description": "Set to true to execute real swap (requires PRIVATE_KEY)" }
            },
            "required": ["from_token", "to_token", "amount"]
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "type": {"type": "string", "const": "text"},
                            "text": {"type": "string"}
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
        let max_spend = args.get("max_spend").and_then(|v| v.as_str());
        let min_receive = args.get("min_receive").and_then(|v| v.as_str());
        let execute = args
            .get("execute")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let quote = if execute {
            let signer = client.get_signer().ok_or("PRIVATE_KEY not configured")?;
            let executor = SwapExecutor::new(signer.clone(), client.get_config().clone());
            executor
                .execute_swap(
                    from_token,
                    to_token,
                    amount,
                    slippage,
                    max_spend,
                    min_receive,
                    client.get_provider(),
                )
                .await
                .map_err(|e| e.to_string())?
        } else {
            client
                .simulate_swap(from_token, to_token, amount, slippage)
                .await
                .map_err(|e| e.to_string())?
        };

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
