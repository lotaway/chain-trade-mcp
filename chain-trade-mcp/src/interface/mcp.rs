use crate::infrastructure::ethereum::EthereumClient;
use crate::interface::tools::{
    balance::BalanceTool, price::PriceTool, swap::SwapTool, tool_trait::ToolRegistry,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use tracing::error;

// Create a global tool registry
fn create_tool_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(BalanceTool));
    registry.register(Box::new(PriceTool));
    registry.register(Box::new(SwapTool));
    registry
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<JsonRpcError>,
    id: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

pub async fn run(eth_client: EthereumClient) -> anyhow::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut lines = stdin.lock().lines();

    while let Some(line) = lines.next() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to parse JSON-RPC request: {}", e);
                continue;
            }
        };

        let response = handle_request(&req, &eth_client).await;
        let response_str = serde_json::to_string(&response)?;

        writeln!(stdout, "{}", response_str)?;
        stdout.flush()?;
    }

    Ok(())
}

async fn handle_request(req: &JsonRpcRequest, client: &EthereumClient) -> JsonRpcResponse {
    let registry = create_tool_registry();

    let result = match req.method.as_str() {
        "tools/list" => list_tools(&registry),
        "tools/call" => call_tool(req.params.as_ref(), client, &registry).await,
        _ => Err(JsonRpcError {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        }),
    };

    match result {
        Ok(res) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(res),
            error: None,
            id: req.id.clone(),
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(e),
            id: req.id.clone(),
        },
    }
}

fn list_tools(registry: &ToolRegistry) -> Result<Value, JsonRpcError> {
    Ok(registry.list_tools())
}

async fn call_tool(
    params: Option<&Value>,
    client: &EthereumClient,
    registry: &ToolRegistry,
) -> Result<Value, JsonRpcError> {
    let params = params.ok_or(JsonRpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: None,
    })?;

    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or(JsonRpcError {
            code: -32602,
            message: "Missing tool name".to_string(),
            data: None,
        })?;

    let default_args = json!({});
    let args = params.get("arguments").unwrap_or(&default_args);

    match registry.get_tool(name) {
        Some(tool) => tool.execute(client, args).await.map_err(|e| JsonRpcError {
            code: -32000,
            message: e,
            data: None,
        }),
        None => Err(JsonRpcError {
            code: -32601,
            message: format!("Tool not found: {}", name),
            data: None,
        }),
    }
}
