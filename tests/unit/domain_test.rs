#[cfg(test)]
mod domain_tests {
    use chain_trade_mcp::domain::{Balance, SwapQuote, Token};
    use serde_json;

    #[test]
    fn test_token_serialization() {
        let token = Token {
            address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
            symbol: "USDC".to_string(),
            decimals: 6,
        };

        let json = serde_json::to_string(&token).unwrap();
        assert!(json.contains("USDC"));
        assert!(json.contains("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"));

        let deserialized: Token = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.symbol, "USDC");
        assert_eq!(deserialized.decimals, 6);
    }

    #[test]
    fn test_balance_with_token() {
        let token = Token {
            address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
            symbol: "USDC".to_string(),
            decimals: 6,
        };

        let balance = Balance {
            token: Some(token.clone()),
            amount: "1000000".to_string(),
            formatted: "1.0 USDC".to_string(),
        };

        assert_eq!(balance.formatted, "1.0 USDC");
        assert!(balance.token.is_some());
        assert_eq!(balance.token.unwrap().symbol, "USDC");
    }

    #[test]
    fn test_balance_eth() {
        let balance = Balance {
            token: None,
            amount: "1000000000000000000".to_string(),
            formatted: "1.0 ETH".to_string(),
        };

        assert_eq!(balance.formatted, "1.0 ETH");
        assert!(balance.token.is_none());
    }

    #[test]
    fn test_swap_quote_success() {
        let quote = SwapQuote {
            from_token: "ETH".to_string(),
            to_token: "USDC".to_string(),
            input_amount: "1.0".to_string(),
            estimated_output: "3000.0".to_string(),
            gas_estimate: "150000".to_string(),
            simulation_success: true,
            error_message: None,
        };

        assert!(quote.simulation_success);
        assert!(quote.error_message.is_none());
        assert_eq!(quote.from_token, "ETH");
        assert_eq!(quote.to_token, "USDC");
    }

    #[test]
    fn test_swap_quote_failure() {
        let quote = SwapQuote {
            from_token: "ETH".to_string(),
            to_token: "USDC".to_string(),
            input_amount: "1.0".to_string(),
            estimated_output: "0".to_string(),
            gas_estimate: "0".to_string(),
            simulation_success: false,
            error_message: Some("Insufficient liquidity".to_string()),
        };

        assert!(!quote.simulation_success);
        assert!(quote.error_message.is_some());
        assert_eq!(quote.error_message.unwrap(), "Insufficient liquidity");
    }

    #[test]
    fn test_swap_quote_serialization() {
        let quote = SwapQuote {
            from_token: "ETH".to_string(),
            to_token: "USDC".to_string(),
            input_amount: "1.0".to_string(),
            estimated_output: "3000.0".to_string(),
            gas_estimate: "150000".to_string(),
            simulation_success: true,
            error_message: None,
        };

        let json = serde_json::to_string(&quote).unwrap();
        let deserialized: SwapQuote = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.from_token, quote.from_token);
        assert_eq!(deserialized.to_token, quote.to_token);
        assert_eq!(deserialized.simulation_success, quote.simulation_success);
    }
}
