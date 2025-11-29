use crate::infrastructure::ethereum::EthereumClient;
use rmcp::{
    handler::server::wrapper::Parameters,
    handler::server::{tool::ToolRouter, ServerHandler},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Main MCP server for Ethereum chain trading operations
#[derive(Clone)]
pub struct ChainTradeServer {
    eth_client: Arc<EthereumClient>,
    tool_router: ToolRouter<Self>,
}

/// Balance query request parameters
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BalanceRequest {
    /// The wallet address to check
    pub address: String,
    /// Optional ERC20 token contract address. If omitted, returns ETH balance.
    pub token_address: Option<String>,
}

/// Token price query request parameters
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PriceRequest {
    /// Token contract address or symbol
    pub token: String,
}

/// Token swap simulation request parameters
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SwapRequest {
    /// Source token address (use native token address for ETH)
    pub from_token: String,
    /// Destination token address
    pub to_token: String,
    /// Amount to swap (in human-readable format, e.g., "1.0")
    pub amount: String,
    /// Optional slippage tolerance (e.g., 0.5 for 0.5%). Defaults to configured value.
    pub slippage: Option<f64>,
}

#[tool_router]
impl ChainTradeServer {
    /// Create a new ChainTradeServer instance
    pub fn new(eth_client: EthereumClient) -> Self {
        Self {
            eth_client: Arc::new(eth_client),
            tool_router: Self::tool_router(),
        }
    }

    /// Get the balance of ETH or an ERC20 token for a specific address
    #[tool(description = "Get the balance of ETH or an ERC20 token for a specific address")]
    async fn get_balance(&self, params: Parameters<BalanceRequest>) -> Result<String, String> {
        let req = params.0;
        let token_address = req.token_address.as_deref();

        let balance = self
            .eth_client
            .get_balance(&req.address, token_address)
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&balance)
            .map_err(|e| format!("Failed to serialize balance: {}", e))
    }

    /// Get current token price in USD or ETH
    #[tool(description = "Get current token price in USDC (via Uniswap V3 Quoter)")]
    async fn get_token_price(&self, params: Parameters<PriceRequest>) -> Result<String, String> {
        let req = params.0;

        let price = self
            .eth_client
            .get_token_price(&req.token)
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&price)
            .map_err(|e| format!("Failed to serialize price: {}", e))
    }

    /// Simulate a token swap on Uniswap V3
    #[tool(
        description = "Simulate a token swap on Uniswap V3 to estimate output and gas costs (does not execute)"
    )]
    async fn swap_tokens(&self, params: Parameters<SwapRequest>) -> Result<String, String> {
        let req = params.0;

        let swap_result = self
            .eth_client
            .simulate_swap(&req.from_token, &req.to_token, &req.amount, req.slippage)
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&swap_result)
            .map_err(|e| format!("Failed to serialize swap result: {}", e))
    }
}

// Implement ServerHandler trait to enable .serve() method
impl ServerHandler for ChainTradeServer {}
