use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub address: String,
    pub symbol: String,
    pub decimals: u8,
}

impl Token {
    pub fn new(address: String, symbol: String, decimals: u8) -> Self {
        Self {
            address,
            symbol,
            decimals,
        }
    }
}
