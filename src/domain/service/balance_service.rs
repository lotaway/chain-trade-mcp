use crate::domain::Balance;
use crate::domain::BalanceRepository;
use async_trait::async_trait;
use std::result::Result;

#[async_trait]
pub trait BalanceService: Send + Sync {
    async fn get_balance(
        &self,
        address: &str,
        token_address: Option<&str>,
    ) -> Result<Balance, String>;
}

#[async_trait]
impl<T: BalanceRepository + Send + Sync> BalanceService for T {
    async fn get_balance(
        &self,
        address: &str,
        token_address: Option<&str>,
    ) -> Result<Balance, String> {
        match token_address {
            Some(token) => self.get_erc20_balance(address, token).await,
            None => self.get_eth_balance(address).await,
        }
    }
}
