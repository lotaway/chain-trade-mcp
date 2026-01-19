#[cfg(test)]
mod tool_tests {
    use chain_trade_mcp::interface::tools::balance::BalanceTool;
    use chain_trade_mcp::interface::tools::price::PriceTool;
    use chain_trade_mcp::interface::tools::swap::SwapTool;
    use chain_trade_mcp::interface::tools::tool_trait::Tool;
    use serde_json::json;

    const WALLET_ADDRESS: &str = "0x8C864D0c8E476Bf9eb9d620C10E1296fb0E2F940";
    const TOKEN_ADDRESS: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    const WETH_ADDRESS: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";

    #[test]
    fn test_balance_tool_metadata() {
        let tool = BalanceTool;

        assert_eq!(tool.name(), "get_balance");
        assert!(!tool.description().is_empty());

        let schema = tool.input_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["address"].is_object());
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .contains(&json!("address")));
    }

    #[test]
    fn test_price_tool_metadata() {
        let tool = PriceTool;

        assert_eq!(tool.name(), "get_token_price");
        assert!(!tool.description().is_empty());

        let schema = tool.input_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["token_address"].is_object());
    }

    #[test]
    fn test_swap_tool_metadata() {
        let tool = SwapTool;

        assert_eq!(tool.name(), "swap_tokens");
        assert!(!tool.description().is_empty());

        let schema = tool.input_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["from_token"].is_object());
        assert!(schema["properties"]["to_token"].is_object());
        assert!(schema["properties"]["amount"].is_object());

        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&json!("from_token")));
        assert!(required.contains(&json!("to_token")));
        assert!(required.contains(&json!("amount")));
    }

    #[test]
    fn test_balance_tool_input_validation() {
        let tool = BalanceTool;
        let schema = tool.input_schema();

        // Check that address is required
        let required = schema["required"].as_array().unwrap();
        assert_eq!(required.len(), 1);
        assert_eq!(required[0], "address");

        // Check that token_address is optional
        assert!(!required.contains(&json!("token_address")));
    }

    #[test]
    fn test_swap_tool_slippage_optional() {
        let tool = SwapTool;
        let schema = tool.input_schema();

        // Check that slippage is optional
        let required = schema["required"].as_array().unwrap();
        assert!(!required.contains(&json!("slippage")));

        // But slippage property should exist
        assert!(schema["properties"]["slippage"].is_object());
    }

    // Integration tests that make actual EVM RPC calls
    // These tests use a public RPC URL that doesn't require API key
    // Using Cloudflare's Ethereum RPC endpoint
    const PUBLIC_RPC_URL: &str = "https://cloudflare-eth.com";

    fn create_test_config() -> chain_trade_mcp::config::Config {
        use chain_trade_mcp::config::Config;

        let mut config = Config::load().expect("Failed to load base config");

        // Override RPC URL with a public endpoint that doesn't require API key
        config.rpc_urls = vec![PUBLIC_RPC_URL.to_string()];

        config
    }
    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_balance_eth() {
        use chain_trade_mcp::config::Config;
        use chain_trade_mcp::infrastructure::cache::CacheService;
        use chain_trade_mcp::infrastructure::ethereum::EthereumClient;
        use chain_trade_mcp::infrastructure::notification::NotificationService;

        let config =
            Config::load().expect("Failed to load config - ensure .env file exists with RPC_URL");
        let cache = CacheService::new(config.cache_ttl);
        let notifier = NotificationService::new(&config);
        let client = EthereumClient::new(config, cache, notifier)
            .await
            .expect("Failed to create Ethereum client");

        let tool = BalanceTool;
        let args = json!({
            "address": WALLET_ADDRESS
        });

        let result = tool.execute(&client, &args).await;
        assert!(result.is_ok(), "Failed to get ETH balance: {:?}", result);

        let response = result.unwrap();
        // Parse MCP response format: {content: [{type: "text", text: JSON_STRING}]}
        let text = response["content"][0]["text"]
            .as_str()
            .expect("response should have content[0].text");
        let balance: serde_json::Value =
            serde_json::from_str(text).expect("text should be valid JSON");

        assert!(balance["amount"].is_string());
        assert!(balance["formatted"].is_string());

        // Verify amount can be parsed as a valid number (U256)
        let amount_str = balance["amount"]
            .as_str()
            .expect("amount should be a string");
        let amount_parsed = amount_str.parse::<alloy::primitives::U256>();
        assert!(
            amount_parsed.is_ok(),
            "amount should be parseable as U256, got: {}",
            amount_str
        );

        // Verify formatted is a valid decimal number
        let formatted_str = balance["formatted"]
            .as_str()
            .expect("formatted should be a string");
        let formatted_parsed = formatted_str.parse::<f64>();
        assert!(
            formatted_parsed.is_ok(),
            "formatted should be parseable as f64, got: {}",
            formatted_str
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_balance_erc20() {
        use chain_trade_mcp::config::Config;
        use chain_trade_mcp::infrastructure::cache::CacheService;
        use chain_trade_mcp::infrastructure::ethereum::EthereumClient;
        use chain_trade_mcp::infrastructure::notification::NotificationService;

        let config =
            Config::load().expect("Failed to load config - ensure .env file exists with RPC_URL");
        let cache = CacheService::new(config.cache_ttl);
        let notifier = NotificationService::new(&config);
        let client = EthereumClient::new(config, cache, notifier)
            .await
            .expect("Failed to create Ethereum client");

        let tool = BalanceTool;
        let args = json!({
            "address": WALLET_ADDRESS,
            "token_address": TOKEN_ADDRESS
        });

        let result = tool.execute(&client, &args).await;
        assert!(result.is_ok(), "Failed to get ERC20 balance: {:?}", result);

        let response = result.unwrap();
        let text = response["content"][0]["text"]
            .as_str()
            .expect("response should have content[0].text");
        let balance: serde_json::Value =
            serde_json::from_str(text).expect("text should be valid JSON");
        assert!(balance["token"].is_object());
        assert!(balance["token"]["symbol"].is_string());
        assert!(balance["token"]["decimals"].is_number());
        assert!(balance["amount"].is_string());
        assert!(balance["formatted"].is_string());

        // Verify amount can be parsed as a valid number (U256)
        let amount_str = balance["amount"]
            .as_str()
            .expect("amount should be a string");
        let amount_parsed = amount_str.parse::<alloy::primitives::U256>();
        assert!(
            amount_parsed.is_ok(),
            "amount should be parseable as U256, got: {}",
            amount_str
        );

        // Verify formatted is a valid decimal number
        let formatted_str = balance["formatted"]
            .as_str()
            .expect("formatted should be a string");
        let formatted_parsed = formatted_str.parse::<f64>();
        assert!(
            formatted_parsed.is_ok(),
            "formatted should be parseable as f64, got: {}",
            formatted_str
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_token_price() {
        use chain_trade_mcp::config::Config;
        use chain_trade_mcp::infrastructure::cache::CacheService;
        use chain_trade_mcp::infrastructure::ethereum::EthereumClient;
        use chain_trade_mcp::infrastructure::notification::NotificationService;

        let config =
            Config::load().expect("Failed to load config - ensure .env file exists with RPC_URL");
        let cache = CacheService::new(config.cache_ttl);
        let notifier = NotificationService::new(&config);
        let client = EthereumClient::new(config, cache, notifier)
            .await
            .expect("Failed to create Ethereum client");

        let tool = PriceTool;
        let args = json!({
            "token_address": TOKEN_ADDRESS
        });

        let result = tool.execute(&client, &args).await;
        assert!(result.is_ok(), "Failed to get token price: {:?}", result);

        let response = result.unwrap();
        let text = response["content"][0]["text"]
            .as_str()
            .expect("response should have content[0].text");
        let price_data: serde_json::Value =
            serde_json::from_str(text).expect("text should be valid JSON");
        assert!(price_data["price"].is_string());
        assert!(price_data["token_address"].is_string());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_swap_tokens_simulation() {
        use chain_trade_mcp::config::Config;
        use chain_trade_mcp::infrastructure::cache::CacheService;
        use chain_trade_mcp::infrastructure::ethereum::EthereumClient;
        use chain_trade_mcp::infrastructure::notification::NotificationService;

        let config =
            Config::load().expect("Failed to load config - ensure .env file exists with RPC_URL");
        let cache = CacheService::new(config.cache_ttl);
        let notifier = NotificationService::new(&config);
        let client = EthereumClient::new(config, cache, notifier)
            .await
            .expect("Failed to create Ethereum client");

        let tool = SwapTool;
        let args = json!({
            "from_token": TOKEN_ADDRESS,
            "to_token": WETH_ADDRESS,
            "amount": "1000000", // 1 USDC (6 decimals)
            "slippage": 0.01
        });

        let result = tool.execute(&client, &args).await;

        // Note: Swap simulation may fail with STF (SafeTransferFrom) error
        // because we're using Address::ZERO which has no token balance.
        // This is expected behavior for simulation without actual funds.
        match result {
            Ok(response) => {
                // If successful, validate the response structure
                let text = response["content"][0]["text"]
                    .as_str()
                    .expect("response should have content[0].text");
                let swap_quote: serde_json::Value =
                    serde_json::from_str(text).expect("text should be valid JSON");
                assert!(swap_quote["from_token"].is_string());
                assert!(swap_quote["to_token"].is_string());
                assert!(swap_quote["input_amount"].is_string());
                assert!(swap_quote["estimated_output"].is_string());
                assert_eq!(swap_quote["simulation_success"], true);
            }
            Err(e) => {
                // Accept STF error as it's expected when simulating without balance
                assert!(
                    e.contains("STF") || e.contains("execution reverted"),
                    "Unexpected error (expected STF or execution reverted): {}",
                    e
                );
            }
        }
    }
}
