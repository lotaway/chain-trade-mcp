use anyhow::Result;
use dotenv::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub rpc_url: String,
    pub private_key: Option<String>,
    pub port: u16,
    pub default_slippage: f64,
    pub uniswap_fee_tier: u32,
    pub cache_ttl: u64,
    pub smtp_host: Option<String>,
    pub smtp_user: Option<String>,
    pub smtp_pass: Option<String>,
    pub smtp_from: Option<String>,
    pub smtp_to: Option<String>,
    // Contract addresses
    pub usdc_address: String,
    pub uniswap_quoter_address: String,
    pub uniswap_router_address: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenv().ok();

        let rpc_url = env::var("RPC_URL")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "https://1rpc.io/eth".to_string());

        let private_key = env::var("PRIVATE_KEY").ok().filter(|k| !k.is_empty());

        let port = env::var("PORT")
            .ok()
            .filter(|s| !s.is_empty())
            .and_then(|s| s.parse().ok())
            .unwrap_or(3000);

        let default_slippage = env::var("DEFAULT_SLIPPAGE")
            .ok()
            .filter(|s| !s.is_empty())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.005);

        let uniswap_fee_tier = env::var("UNISWAP_FEE_TIER")
            .ok()
            .filter(|s| !s.is_empty())
            .and_then(|s| s.parse().ok())
            .unwrap_or(3000);

        let cache_ttl = env::var("CACHE_TTL")
            .ok()
            .filter(|s| !s.is_empty())
            .and_then(|s| s.parse().ok())
            .unwrap_or(5);

        let smtp_host = env::var("SMTP_HOST").ok().filter(|s| !s.is_empty());
        let smtp_user = env::var("SMTP_USER").ok().filter(|s| !s.is_empty());
        let smtp_pass = env::var("SMTP_PASS").ok().filter(|s| !s.is_empty());
        let smtp_from = env::var("SMTP_FROM").ok().filter(|s| !s.is_empty());
        let smtp_to = env::var("SMTP_TO").ok().filter(|s| !s.is_empty());

        // Contract addresses with mainnet defaults
        let usdc_address = env::var("USDC_ADDRESS")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string());

        let uniswap_quoter_address = env::var("UNISWAP_QUOTER_ADDRESS")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "0x61fFE014bA17989E743c5F6cB21bF9697530B21e".to_string());

        let uniswap_router_address = env::var("UNISWAP_ROUTER_ADDRESS")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45".to_string());

        Ok(Config {
            rpc_url,
            private_key,
            port,
            default_slippage,
            uniswap_fee_tier,
            cache_ttl,
            smtp_host,
            smtp_user,
            smtp_pass,
            smtp_from,
            smtp_to,
            usdc_address,
            uniswap_quoter_address,
            uniswap_router_address,
        })
    }
}
