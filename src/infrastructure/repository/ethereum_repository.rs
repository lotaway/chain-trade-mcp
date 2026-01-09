use crate::config::Config;
use crate::domain::{
    Balance, BalanceRepository, PriceRepository, SwapQuote, SwapRepository, Token,
};
use crate::infrastructure::cache::CacheService;
use crate::infrastructure::notification::NotificationService;
use crate::infrastructure::rpc::{RpcConnectionPool, RpcRateLimiter};
use alloy::primitives::utils::format_units;
use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use alloy::sol;
use anyhow::Result;
use async_trait::async_trait;
use std::str::FromStr;
use std::sync::Arc;
use tracing::error;

sol! {
    #[sol(rpc)]
    contract IERC20 {
        function balanceOf(address account) external view returns (uint256);
        function decimals() external view returns (uint8);
        function symbol() external view returns (string);
    }
}

sol! {
    #[sol(rpc)]
    contract IQuoterV2 {
        function quoteExactInputSingle(
            address tokenIn,
            address tokenOut,
            uint24 fee,
            uint256 amountIn,
            uint160 sqrtPriceLimitX96
        ) external returns (uint256 amountOut, uint160 sqrtPriceX96After, uint32 initializedTicksCrossed, uint256 gasEstimate);
    }
}

sol! {
    #[sol(rpc)]
    contract ISwapRouter {
        struct ExactInputSingleParams {
            address tokenIn;
            address tokenOut;
            uint24 fee;
            address recipient;
            uint256 amountIn;
            uint256 amountOutMinimum;
            uint160 sqrtPriceLimitX96;
        }
        function exactInputSingle(ExactInputSingleParams calldata params) external payable returns (uint256 amountOut);
    }
}

pub struct EthereumRepository {
    pool: Arc<RpcConnectionPool>,
    rate_limiter: Arc<RpcRateLimiter>,
    config: Config,
    cache: CacheService,
    notifier: NotificationService,
    signer: Option<PrivateKeySigner>,
}

impl EthereumRepository {
    pub fn new(
        pool: Arc<RpcConnectionPool>,
        rate_limiter: Arc<RpcRateLimiter>,
        config: Config,
        cache: CacheService,
        notifier: NotificationService,
    ) -> Self {
        let signer = config
            .private_key
            .as_ref()
            .map(|pk| PrivateKeySigner::from_str(pk).expect("Failed to create signer"));

        Self {
            pool,
            rate_limiter,
            config,
            cache,
            notifier,
            signer,
        }
    }

    async fn cached_balance(&self, cache_key: &str) -> Option<Balance> {
        if let Some(cached) = self.cache.get(cache_key).await {
            if let Ok(balance) = serde_json::from_str(&cached) {
                return Some(balance);
            }
        }
        None
    }

    async fn cache_balance(&self, cache_key: String, balance: &Balance) {
        if let Ok(json) = serde_json::to_string(balance) {
            self.cache.insert(cache_key, json).await;
        }
    }

    fn notify_error(&self, operation: &str, error: &str) {
        let msg = format!("{} failed: {}", operation, error);
        error!("{}", msg);
        self.notifier
            .send_alert(&format!("RPC Error: {}", operation), &msg);
    }
}

#[async_trait]
impl BalanceRepository for EthereumRepository {
    async fn get_eth_balance(&self, address: &str) -> Result<Balance, String> {
        let cache_key = format!("eth_balance:{}", address);
        if let Some(balance) = self.cached_balance(&cache_key).await {
            return Ok(balance);
        }

        if !self.rate_limiter.acquire().await {
            return Err("Rate limit exceeded".to_string());
        }

        let addr = Address::from_str(address)
            .map_err(|e| format!("Invalid address {}: {}", address, e))?;

        let result = self.pool.provider().get_balance(addr).await;
        self.rate_limiter.acquire().await; // Refill

        match result {
            Ok(balance) => {
                let formatted = format_units(balance, 18)
                    .map_err(|e| format!("Failed to format ETH: {}", e))?;
                let balance = Balance::eth(balance.to_string(), formatted);
                self.cache_balance(cache_key, &balance).await;
                Ok(balance)
            }
            Err(e) => {
                self.notify_error("get_eth_balance", &e.to_string());
                Err(e.to_string())
            }
        }
    }

    async fn get_erc20_balance(
        &self,
        address: &str,
        token_address: &str,
    ) -> Result<Balance, String> {
        let cache_key = format!("erc20_balance:{}:{}", address, token_address);
        if let Some(balance) = self.cached_balance(&cache_key).await {
            return Ok(balance);
        }

        if !self.rate_limiter.acquire().await {
            return Err("Rate limit exceeded".to_string());
        }

        let addr = Address::from_str(address)
            .map_err(|e| format!("Invalid address {}: {}", address, e))?;
        let token_addr = Address::from_str(token_address)
            .map_err(|e| format!("Invalid token address {}: {}", token_address, e))?;

        let contract = IERC20::new(token_addr, self.pool.provider());

        let result = async {
            let balance = contract
                .balanceOf(addr)
                .call()
                .await
                .map_err(|e| e.to_string())?
                ._0;
            let decimals = contract
                .decimals()
                .call()
                .await
                .map_err(|e| e.to_string())?
                ._0;
            let symbol = contract
                .symbol()
                .call()
                .await
                .map_err(|e| e.to_string())?
                ._0;
            Ok::<_, String>((balance, decimals, symbol))
        }
        .await;

        self.rate_limiter.acquire().await;

        match result {
            Ok((balance, decimals, symbol)) => {
                let formatted = format_units(balance, decimals)
                    .map_err(|e| format!("Failed to format token: {}", e))?;
                let token = Token::new(token_address.to_string(), symbol, decimals);
                let balance = Balance::erc20(token, balance.to_string(), formatted);
                self.cache_balance(cache_key, &balance).await;
                Ok(balance)
            }
            Err(e) => {
                let error_msg = e.to_string();
                self.notify_error("get_erc20_balance", &error_msg);
                Err(error_msg)
            }
        }
    }
}

#[async_trait]
impl PriceRepository for EthereumRepository {
    async fn get_price(&self, token_address: &str) -> Result<String, String> {
        let cache_key = format!("price:{}", token_address);
        if let Some(cached) = self.cache.get(&cache_key).await {
            return Ok(cached);
        }

        if !self.rate_limiter.acquire().await {
            return Err("Rate limit exceeded".to_string());
        }

        let usdc = Address::from_str(&self.config.usdc_address)
            .map_err(|e| format!("Invalid USDC address: {}", e))?;
        let token = Address::from_str(token_address)
            .map_err(|e| format!("Invalid token address: {}", e))?;

        if token == usdc {
            return Ok("1.0".to_string());
        }

        let quoter_addr = Address::from_str(&self.config.uniswap_quoter_address)
            .map_err(|e| format!("Invalid quoter address: {}", e))?;

        let quoter = IQuoterV2::new(quoter_addr, self.pool.provider());
        let token_contract = IERC20::new(token, self.pool.provider());

        let result = async {
            let decimals = token_contract
                .decimals()
                .call()
                .await
                .map_err(|e| e.to_string())?
                ._0;
            let amount_in = U256::from(10).pow(U256::from(decimals));
            let quote = quoter
                .quoteExactInputSingle(
                    token,
                    usdc,
                    self.config.uniswap_fee_tier,
                    amount_in,
                    U256::ZERO,
                )
                .call()
                .await
                .map_err(|e| e.to_string())?;
            let price = format_units(quote.amountOut, 6).map_err(|e| e.to_string())?;
            Ok::<_, String>(price)
        }
        .await;

        self.rate_limiter.acquire().await;

        match result {
            Ok(price) => {
                self.cache.insert(cache_key, price.clone()).await;
                Ok(price)
            }
            Err(e) => {
                self.notify_error("get_price", &e);
                Err(e)
            }
        }
    }
}

#[async_trait]
impl SwapRepository for EthereumRepository {
    async fn simulate_swap(
        &self,
        from_token: &str,
        to_token: &str,
        amount: &str,
        slippage: Option<f64>,
    ) -> Result<SwapQuote, String> {
        if !self.rate_limiter.acquire().await {
            return Ok(SwapQuote::failure(
                from_token.to_string(),
                to_token.to_string(),
                amount.to_string(),
                "Rate limit exceeded".to_string(),
            ));
        }

        let from =
            Address::from_str(from_token).map_err(|e| format!("Invalid from token: {}", e))?;
        let to = Address::from_str(to_token).map_err(|e| format!("Invalid to token: {}", e))?;

        let from_contract = IERC20::new(from, self.pool.provider());
        let result = async {
            let decimals = from_contract
                .decimals()
                .call()
                .await
                .map_err(|e| e.to_string())?
                ._0;
            let amount_in: U256 = match U256::from_str(amount) {
                Ok(v) => v,
                Err(_) => {
                    let f: f64 = amount.parse().unwrap_or(0.0);
                    let u = (f * 10f64.powi(decimals as i32)).round();
                    U256::from(u as u128)
                }
            };

            let router_addr = Address::from_str(&self.config.uniswap_router_address)
                .map_err(|e| format!("Invalid router address: {}", e))?;

            let params = ISwapRouter::ExactInputSingleParams {
                tokenIn: from,
                tokenOut: to,
                fee: self.config.uniswap_fee_tier,
                recipient: alloy::primitives::Address::ZERO,
                amountIn: amount_in,
                amountOutMinimum: U256::ZERO,
                sqrtPriceLimitX96: U256::ZERO,
            };

            let router = ISwapRouter::new(router_addr, self.pool.provider());
            let calldata = router.exactInputSingle(params).calldata().to_owned();

            let tx = TransactionRequest::default()
                .from(alloy::primitives::Address::ZERO)
                .to(router_addr)
                .input(calldata.into());

            let output = self
                .pool
                .provider()
                .call(&tx)
                .await
                .map_err(|e| e.to_string())?;
            let amount_out = U256::from_be_slice(&output);

            let to_contract = IERC20::new(to, self.pool.provider());
            let to_decimals = to_contract
                .decimals()
                .call()
                .await
                .map_err(|e| e.to_string())?
                ._0;
            let formatted_out = format_units(amount_out, to_decimals).map_err(|e| e.to_string())?;

            let slippage_tolerance = slippage.unwrap_or(self.config.default_slippage);
            let min_output = amount_out
                .saturating_mul(U256::from((10000.0 * (1.0 - slippage_tolerance)) as u64))
                / U256::from(10000);
            let formatted_min_out =
                format_units(min_output, to_decimals).map_err(|e| e.to_string())?;

            Ok::<_, String>((formatted_out, formatted_min_out, slippage_tolerance))
        }
        .await;

        self.rate_limiter.acquire().await;

        match result {
            Ok((formatted_out, formatted_min_out, slippage)) => Ok(SwapQuote::success(
                from_token.to_string(),
                to_token.to_string(),
                amount.to_string(),
                format!(
                    "{} (min: {} with {}% slippage)",
                    formatted_out,
                    formatted_min_out,
                    slippage * 100.0
                ),
                "Unknown".to_string(),
            )),
            Err(e) => {
                self.notify_error("simulate_swap", &e);
                Ok(SwapQuote::failure(
                    from_token.to_string(),
                    to_token.to_string(),
                    amount.to_string(),
                    e,
                ))
            }
        }
    }
}
