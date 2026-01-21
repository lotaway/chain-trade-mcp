use crate::domain::service::news_service::{NewsRepository, NewsService};
use crate::infrastructure::ethereum::EthereumClient;
use crate::infrastructure::swap_executor::SwapExecutor;
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

/// Token swap request parameters
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
    /// Optional maximum amount to spend
    pub max_spend: Option<String>,
    /// Optional minimum amount to receive
    pub min_receive: Option<String>,
    /// Set to true to execute real swap; default false for simulation
    pub execute: Option<bool>,
    /// Router address (required for execute=true; only Uniswap supported)
    pub router_address: Option<String>,
}

/// News search request parameters
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct NewsRequest {
    /// Search query (e.g., 'ethereum', 'bitcoin')
    pub query: String,
    /// Maximum number of articles to return (default: 10, max: 20)
    pub limit: Option<u32>,
    /// Source: 'rss' or 'cryptopanic' (default: rss)
    pub source: Option<String>,
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

    #[tool(
        description = "Swap tokens on Uniswap V3. Set execute=true for real tx, otherwise simulate."
    )]
    async fn swap_tokens(&self, params: Parameters<SwapRequest>) -> Result<String, String> {
        let req = params.0;
        let do_execute = req.execute.unwrap_or(false);
        if do_execute {
            // Enforce input constraints for real execution
            if req.slippage.is_none() {
                return Err("slippage is required for real execution".to_string());
            }
            if req.max_spend.is_none() && req.min_receive.is_none() {
                return Err("either max_spend or min_receive is required".to_string());
            }
            let router_address = req
                .router_address
                .ok_or_else(|| "router_address is required (only Uniswap supported)".to_string())?;
            if router_address != self.eth_client.get_config().uniswap_router_address {
                return Err("unsupported router_address".to_string());
            }

            let signer = self
                .eth_client
                .get_signer()
                .ok_or_else(|| "PRIVATE_KEY not configured".to_string())?;
            let executor = SwapExecutor::new(signer.clone(), self.eth_client.get_config().clone());
            let result = executor
                .execute_swap(
                    &req.from_token,
                    &req.to_token,
                    &req.amount,
                    req.slippage,
                    req.max_spend.as_deref(),
                    req.min_receive.as_deref(),
                    self.eth_client.get_provider(),
                )
                .await
                .map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&result)
                .map_err(|e| format!("Failed to serialize swap result: {}", e))
        } else {
            let result = self
                .eth_client
                .simulate_swap(&req.from_token, &req.to_token, &req.amount, req.slippage)
                .await
                .map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&result)
                .map_err(|e| format!("Failed to serialize swap result: {}", e))
        }
    }

    /// Search for cryptocurrency news (returns raw articles without summarization)
    #[tool(description = "Search for cryptocurrency news from RSS feeds and public APIs")]
    async fn news_search(&self, params: Parameters<NewsRequest>) -> Result<String, String> {
        let req = params.0;
        let service = NewsService::new();
        let result = service
            .search_news(&req.query, req.limit, req.source.as_deref())
            .await
            .map_err(|e| e)?;
        serde_json::to_string_pretty(&result)
            .map_err(|e| format!("Failed to serialize news result: {}", e))
    }
}

// Implement ServerHandler trait to enable .serve() method
impl ServerHandler for ChainTradeServer {}
