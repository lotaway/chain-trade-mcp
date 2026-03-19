use crate::domain::Balance;
use async_trait::async_trait;
use std::result::Result;

#[async_trait]
pub trait BalanceRepository: Send + Sync {
    async fn get_eth_balance(&self, address: &str) -> Result<Balance, String>;
    async fn get_erc20_balance(
        &self,
        address: &str,
        token_address: &str,
    ) -> Result<Balance, String>;
}
