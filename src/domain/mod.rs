use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub address: String,
    pub symbol: String,
    pub decimals: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub token: Option<Token>, // None for ETH
    pub amount: String,
    pub formatted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapQuote {
    pub from_token: String,
    pub to_token: String,
    pub input_amount: String,
    pub estimated_output: String,
    pub gas_estimate: String,
    pub simulation_success: bool,
    pub error_message: Option<String>,
}
