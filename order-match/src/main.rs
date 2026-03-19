use crate::config::AppConfig;

mod config;
mod order_match;
mod generated;
mod ai_quant;
mod trade_gateway;

use ai_quant::client::AIClient;
use trade_gateway::TradeGateway;

#[tokio::main]
async fn main() {
    println!("Order Book Program is running!");
    let config = match AppConfig::load() {
        Ok(config) => {
            println!("Config loaded: {:#?}", config);
            config
        }
        Err(e) => {
            println!("Config loading failed: {:#?}", e);
            return;
        }
    };

    // Initialize AI Trading Components
    let ai_client = AIClient::new(config.ai_quant.clone());
    let trade_gateway = TradeGateway::new(ai_client);

    // Spawn a task to simulate AI trading (for demonstration)
    if config.ai_quant.enabled {
        let gateway = std::sync::Arc::new(trade_gateway);
        tokio::spawn(async move {
            println!("Starting AI Trading Loop...");
            loop {
                // Mock market data
                let market_data = "BTC/USDT price: 45000, volume: 100";
                if let Err(e) = gateway.process_market_data(market_data).await {
                    println!("Error processing market data: {}", e);
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            }
        });
    }

    order_match::order_match::start().await;
}
