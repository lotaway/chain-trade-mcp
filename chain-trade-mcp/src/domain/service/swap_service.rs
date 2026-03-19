use crate::domain::SwapQuote;
use crate::domain::SwapRepository;
use async_trait::async_trait;
use std::result::Result;

#[async_trait]
pub trait SwapService: Send + Sync {
    async fn simulate_swap(
        &self,
        from_token: &str,
        to_token: &str,
        amount: &str,
        slippage: Option<f64>,
    ) -> Result<SwapQuote, String>;
}

#[async_trait]
impl<T: SwapRepository + Send + Sync> SwapService for T {
    async fn simulate_swap(
        &self,
        from_token: &str,
        to_token: &str,
        amount: &str,
        slippage: Option<f64>,
    ) -> Result<SwapQuote, String> {
        self.simulate_swap(from_token, to_token, amount, slippage)
            .await
    }
}
