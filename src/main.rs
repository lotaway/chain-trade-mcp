mod config;
mod domain;
mod infrastructure;
mod interface;

use config::Config;
use infrastructure::cache::CacheService;
use infrastructure::notification::NotificationService;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Set up file appender with daily rotation
    let file_appender = tracing_appender::rolling::daily("./logs", "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Initialize tracing with both console and file output
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stdout)) // Console output
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false)) // File output (no ANSI colors)
        .init();

    info!("Starting Ethereum Trading MCP Server...");

    let config = Config::load()?;
    info!("Configuration loaded. RPC URL: {}", config.rpc_url);
    info!(
        "Server configured for port: {} (MCP uses stdio)",
        config.port
    );

    let cache = CacheService::new(config.cache_ttl);
    let notifier = NotificationService::new(&config);

    let eth_client = infrastructure::ethereum::EthereumClient::new(config, cache, notifier).await?;

    interface::mcp::run(eth_client).await?;

    Ok(())
}
