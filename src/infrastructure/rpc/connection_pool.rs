use alloy::providers::{Provider, ProviderBuilder, RootProvider};
use alloy::transports::http::{Client, Http};
use std::sync::Arc;
use std::time::Duration;
use url::Url;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub struct RpcConnectionPool {
    provider: Arc<RootProvider<Http<Client>>>,
}

impl RpcConnectionPool {
    pub async fn new(rpc_url: &str) -> Result<Self, String> {
        let url = Url::parse(rpc_url).map_err(|e| format!("Invalid RPC URL: {}", e))?;

        let provider = ProviderBuilder::new().on_http(url);

        Ok(Self {
            provider: Arc::new(provider),
        })
    }

    pub fn provider(&self) -> &RootProvider<Http<Client>> {
        &self.provider
    }
}
