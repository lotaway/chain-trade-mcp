use crate::config::Config;
use crate::domain::{Balance, SwapQuote, Token};
use crate::infrastructure::cache::CacheService;
use crate::infrastructure::notification::NotificationService;
use alloy::{
    primitives::{utils::format_units, Address, U256},
    providers::{Provider, ProviderBuilder, RootProvider},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
    sol,
    transports::http::{Client, Http},
};
use anyhow::Result;
use std::str::FromStr;
use tracing::error;
use url::Url;

sol! {
    #[sol(rpc)]
    contract IERC20 {
        function balanceOf(address account) external view returns (uint256);
        function decimals() external view returns (uint8);
        function symbol() external view returns (string);
    }
}

pub struct EthereumClient {
    provider: RootProvider<Http<Client>>,
    #[allow(dead_code)]
    signer: Option<PrivateKeySigner>,
    config: Config,
    cache: CacheService,
    notifier: NotificationService,
}

impl EthereumClient {
    pub async fn new(
        config: Config,
        cache: CacheService,
        notifier: NotificationService,
    ) -> Result<Self> {
        let url = Url::parse(&config.rpc_url)?;
        let provider = ProviderBuilder::new().on_http(url);

        let signer = if let Some(pk) = &config.private_key {
            Some(PrivateKeySigner::from_str(pk)?)
        } else {
            None
        };

        Ok(Self {
            provider,
            signer,
            config,
            cache,
            notifier,
        })
    }

    pub async fn get_balance(&self, address: &str, token_address: Option<&str>) -> Result<Balance> {
        let cache_key = format!("balance:{}:{:?}", address, token_address);
        if let Some(cached) = self.cache.get(&cache_key).await {
            if let Ok(balance) = serde_json::from_str(&cached) {
                return Ok(balance);
            }
        }

        let result = self.fetch_balance(address, token_address).await;

        match result {
            Ok(balance) => {
                if let Ok(json) = serde_json::to_string(&balance) {
                    self.cache.insert(cache_key, json).await;
                }
                Ok(balance)
            }
            Err(e) => {
                let msg = format!("Failed to fetch balance for {}: {}", address, e);
                error!("{}", msg);
                self.notifier.send_alert("RPC Error: get_balance", &msg);
                Err(e)
            }
        }
    }

    async fn fetch_balance(&self, address: &str, token_address: Option<&str>) -> Result<Balance> {
        let addr = Address::from_str(address)?;

        if let Some(token_addr_str) = token_address {
            let token_addr = Address::from_str(token_addr_str)?;
            let contract = IERC20::new(token_addr, &self.provider);

            let balance = contract.balanceOf(addr).call().await?._0;
            let decimals = contract.decimals().call().await?._0;
            let symbol = contract.symbol().call().await?._0;

            let formatted = format_units(balance, decimals)?;

            Ok(Balance {
                token: Some(Token {
                    address: token_addr_str.to_string(),
                    symbol,
                    decimals,
                }),
                amount: balance.to_string(),
                formatted,
            })
        } else {
            let balance = self.provider.get_balance(addr).await?;
            let formatted = format_units(balance, 18)?;

            Ok(Balance {
                token: None,
                amount: balance.to_string(),
                formatted,
            })
        }
    }

    pub async fn get_token_price(&self, token_address: &str) -> Result<String> {
        let cache_key = format!("price:{}", token_address);
        if let Some(cached) = self.cache.get(&cache_key).await {
            return Ok(cached);
        }

        let result = self.fetch_token_price(token_address).await;

        match result {
            Ok(price) => {
                self.cache.insert(cache_key, price.clone()).await;
                Ok(price)
            }
            Err(e) => {
                let msg = format!("Failed to fetch price for {}: {}", token_address, e);
                error!("{}", msg);
                self.notifier.send_alert("RPC Error: get_token_price", &msg);
                Err(e)
            }
        }
    }

    async fn fetch_token_price(&self, token_address: &str) -> Result<String> {
        let usdc = Address::from_str(&self.config.usdc_address)?;
        let token = Address::from_str(token_address)?;

        if token == usdc {
            return Ok("1.0".to_string());
        }

        let quoter_addr = Address::from_str(&self.config.uniswap_quoter_address)?;

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

        let quoter = IQuoterV2::new(quoter_addr, &self.provider);
        let token_contract = IERC20::new(token, &self.provider);
        let decimals = token_contract.decimals().call().await?._0;
        let amount_in = U256::from(10).pow(U256::from(decimals));

        let fee = self.config.uniswap_fee_tier as u32; // Use config
                                                       // Note: alloy sol! types might expect uint24, so we cast.
        let fee_24 = fee as u32;

        let quote = quoter
            .quoteExactInputSingle(token, usdc, fee_24, amount_in, U256::ZERO)
            .call()
            .await?;
        let amount_out = quote.amountOut;
        let price = format_units(amount_out, 6)?;

        Ok(price)
    }

    pub async fn simulate_swap(
        &self,
        from_token: &str,
        to_token: &str,
        amount: &str,
        slippage: Option<f64>,
    ) -> Result<SwapQuote> {
        let result = self
            .perform_simulation(from_token, to_token, amount, slippage)
            .await;

        if let Err(e) = &result {
            let msg = format!("Swap simulation failed: {}", e);
            error!("{}", msg);
            self.notifier.send_alert("RPC Error: simulate_swap", &msg);
        }

        result
    }

    async fn perform_simulation(
        &self,
        from_token: &str,
        to_token: &str,
        amount: &str,
        slippage: Option<f64>,
    ) -> Result<SwapQuote> {
        let from = Address::from_str(from_token)?;
        let to = Address::from_str(to_token)?;

        let from_contract = IERC20::new(from, &self.provider);
        let decimals = from_contract.decimals().call().await?._0;

        let amount_in = if let Ok(u) = U256::from_str(amount) {
            u
        } else {
            let f: f64 = amount.parse().unwrap_or(0.0);
            let u = (f * 10f64.powi(decimals as i32)).round();
            U256::from(u as u128)
        };

        let router_addr = Address::from_str(&self.config.uniswap_router_address)?;

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

        let recipient = Address::ZERO;

        let params = ISwapRouter::ExactInputSingleParams {
            tokenIn: from,
            tokenOut: to,
            fee: self.config.uniswap_fee_tier as u32,
            recipient,
            amountIn: amount_in,
            amountOutMinimum: U256::ZERO,
            sqrtPriceLimitX96: U256::ZERO,
        };

        let router = ISwapRouter::new(router_addr, &self.provider);
        let calldata = router.exactInputSingle(params).calldata().to_owned();

        let tx = TransactionRequest::default()
            .from(Address::ZERO)
            .to(router_addr)
            .input(calldata.into());

        let output = self.provider.call(&tx).await?;
        let amount_out = U256::from_be_slice(&output);

        let to_contract = IERC20::new(to, &self.provider);
        let to_decimals = to_contract.decimals().call().await?._0;
        let formatted_out = format_units(amount_out, to_decimals)?;

        // Use slippage to calculate minimum acceptable output
        let slippage_tolerance = slippage.unwrap_or(self.config.default_slippage);
        let min_output = amount_out
            .saturating_mul(U256::from((10000.0 * (1.0 - slippage_tolerance)) as u64))
            / U256::from(10000);
        let formatted_min_out = format_units(min_output, to_decimals)?;

        Ok(SwapQuote {
            from_token: from_token.to_string(),
            to_token: to_token.to_string(),
            input_amount: amount.to_string(),
            estimated_output: format!(
                "{} (min: {} with {}% slippage)",
                formatted_out,
                formatted_min_out,
                slippage_tolerance * 100.0
            ),
            gas_estimate: "Unknown".to_string(),
            simulation_success: true,
            error_message: None,
        })
    }
}
