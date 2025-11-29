mod config;
mod domain;
mod infrastructure;
mod server;

use config::Config;
use infrastructure::cache::CacheService;
use infrastructure::notification::NotificationService;
use rmcp::ServiceExt;
use tokio::io::{stdin, stdout};
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

    // Create the MCP server using rmcp SDK
    let server = server::ChainTradeServer::new(eth_client);

    // Use stdin/stdout as transport (standard MCP pattern)
    let transport = (stdin(), stdout());

    info!("MCP server initialized, starting to serve requests...");

    // Start serving MCP requests
    server.serve(transport).await?;

    Ok(())
}
