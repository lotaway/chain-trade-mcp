use crate::domain::SwapQuote;
use async_trait::async_trait;
use std::result::Result;

#[async_trait]
pub trait SwapRepository: Send + Sync {
    async fn simulate_swap(
        &self,
        from_token: &str,
        to_token: &str,
        amount: &str,
        slippage: Option<f64>,
    ) -> Result<SwapQuote, String>;
}
