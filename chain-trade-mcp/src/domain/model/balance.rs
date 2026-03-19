use serde::{Deserialize, Serialize};

use super::token::Token;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub token: Option<Token>,
    pub amount: String,
    pub formatted: String,
}

impl Balance {
    pub fn eth(amount: String, formatted: String) -> Self {
        Self {
            token: None,
            amount,
            formatted,
        }
    }

    pub fn erc20(token: Token, amount: String, formatted: String) -> Self {
        Self {
            token: Some(token),
            amount,
            formatted,
        }
    }
}
