use crate::domain::PriceRepository;
use async_trait::async_trait;
use std::result::Result;

#[async_trait]
pub trait PriceService: Send + Sync {
    async fn get_price(&self, token_address: &str) -> Result<String, String>;
}

#[async_trait]
impl<T: PriceRepository + Send + Sync> PriceService for T {
    async fn get_price(&self, token_address: &str) -> Result<String, String> {
        self.get_price(token_address).await
    }
}
