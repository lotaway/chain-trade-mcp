use async_trait::async_trait;
use std::result::Result;

#[async_trait]
pub trait PriceRepository: Send + Sync {
    async fn get_price(&self, token_address: &str) -> Result<String, String>;
}
