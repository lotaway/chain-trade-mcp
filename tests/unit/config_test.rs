#[cfg(test)]
mod config_tests {
    use serial_test::serial;
    use std::env;

    #[test]
    #[serial]
    fn test_config_load_with_defaults() {
        // Set minimal required env vars
        env::set_var("RPC_URL", "https://1rpc.io/eth");

        // Clear optional vars to test defaults
        env::remove_var("PORT");
        env::remove_var("DEFAULT_SLIPPAGE");
        env::remove_var("UNISWAP_FEE_TIER");
        env::remove_var("CACHE_TTL");

        let config = chain_trade_mcp::config::Config::load();
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.rpc_urls, vec!["https://1rpc.io/eth"]);
        assert_eq!(config.port, 3000);
        assert_eq!(config.default_slippage, 0.005);
        assert_eq!(config.uniswap_fee_tier, 3000);
        assert_eq!(config.cache_ttl, 60);
    }

    #[test]
    #[serial]
    fn test_config_load_with_multiple_rpc_urls() {
        env::set_var("RPC_URL", "https://1rpc.io/eth,https://eth.llamarpc.com");
        env::remove_var("PORT");
        env::remove_var("DEFAULT_SLIPPAGE");
        env::remove_var("UNISWAP_FEE_TIER");
        env::remove_var("CACHE_TTL");

        let config = chain_trade_mcp::config::Config::load();
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.rpc_urls.len(), 2);
        assert_eq!(config.rpc_urls[0], "https://1rpc.io/eth");
        assert_eq!(config.rpc_urls[1], "https://eth.llamarpc.com");
    }

    #[test]
    #[serial]
    fn test_config_load_with_custom_values() {
        env::set_var("RPC_URL", "https://1rpc.io/eth");
        env::set_var("PORT", "8080");
        env::set_var("DEFAULT_SLIPPAGE", "0.01");
        env::set_var("UNISWAP_FEE_TIER", "500");
        env::set_var("CACHE_TTL", "120");
        env::set_var("USDC_ADDRESS", "0x1234567890123456789012345678901234567890");

        let config = chain_trade_mcp::config::Config::load();
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.rpc_urls, vec!["https://1rpc.io/eth"]);
        assert_eq!(config.port, 8080);
        assert_eq!(config.default_slippage, 0.01);
        assert_eq!(config.uniswap_fee_tier, 500);
        assert_eq!(config.cache_ttl, 120);
        assert_eq!(
            config.usdc_address,
            "0x1234567890123456789012345678901234567890"
        );
    }

    #[test]
    #[serial]
    fn test_config_contract_addresses_defaults() {
        env::set_var("RPC_URL", "https://1rpc.io/eth");
        env::remove_var("USDC_ADDRESS");
        env::remove_var("UNISWAP_QUOTER_ADDRESS");
        env::remove_var("UNISWAP_ROUTER_ADDRESS");

        let config = chain_trade_mcp::config::Config::load().unwrap();

        // Check mainnet defaults
        assert_eq!(
            config.usdc_address,
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        );
        assert_eq!(
            config.uniswap_quoter_address,
            "0x61fFE014bA17989E743c5F6cB21bF9697530B21e"
        );
        assert_eq!(
            config.uniswap_router_address,
            "0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45"
        );
    }

    #[test]
    #[serial]
    fn test_config_optional_smtp_settings() {
        env::set_var("RPC_URL", "https://1rpc.io/eth");
        env::remove_var("SMTP_HOST");
        env::remove_var("SMTP_USER");
        env::remove_var("SMTP_PASS");

        let config = chain_trade_mcp::config::Config::load().unwrap();

        assert!(config.smtp_host.is_none());
        assert!(config.smtp_user.is_none());
        assert!(config.smtp_pass.is_none());

        // Now set them
        env::set_var("SMTP_HOST", "smtp.example.com");
        env::set_var("SMTP_USER", "user@example.com");
        env::set_var("SMTP_PASS", "password123");

        let config = chain_trade_mcp::config::Config::load().unwrap();

        assert_eq!(config.smtp_host, Some("smtp.example.com".to_string()));
        assert_eq!(config.smtp_user, Some("user@example.com".to_string()));
        assert_eq!(config.smtp_pass, Some("password123".to_string()));
    }
}
