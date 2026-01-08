use alloy::providers::{ProviderBuilder, RootProvider};
use alloy::transports::http::{Client, Http};
use std::sync::Arc;
use std::time::Duration;
use url::Url;

#[derive(Clone)]
pub struct RpcConnectionPool {
    provider: Arc<RootProvider<Http<Client>>>,
    timeout: Duration,
}

impl RpcConnectionPool {
    pub async fn new(rpc_url: &str, timeout: Duration) -> Result<Self, String> {
        let url = Url::parse(rpc_url).map_err(|e| format!("Invalid RPC URL: {}", e))?;

        let provider = ProviderBuilder::new().on_http(url);

        Ok(Self {
            provider: Arc::new(provider),
            timeout,
        })
    }

    pub fn provider(&self) -> &RootProvider<Http<Client>> {
        &self.provider
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}
