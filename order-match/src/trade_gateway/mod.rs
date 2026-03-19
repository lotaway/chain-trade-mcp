use crate::ai_quant::client::AIClient;
use anyhow::Result;

pub struct TradeGateway {
    ai_client: AIClient,
}

impl TradeGateway {
    pub fn new(ai_client: AIClient) -> Self {
        Self { ai_client }
    }

    pub async fn process_market_data(&self, market_data: &str) -> Result<()> {
        let signal = self.ai_client.get_trade_signal(market_data).await?;
        println!("Received Trade Signal: {}", signal);
        // Here we would parse the signal and submit an order to the matching engine
        // For now, we just print it.
        Ok(())
    }
}
