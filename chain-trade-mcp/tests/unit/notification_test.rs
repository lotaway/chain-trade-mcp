#[cfg(test)]
mod notification_tests {
    use chain_trade_mcp::config::Config;
    use chain_trade_mcp::infrastructure::notification::NotificationService;
    use serial_test::serial;
    use std::env;

    #[test]
    #[serial]
    fn test_notification_service_creation_without_smtp() {
        env::set_var("RPC_URL", "https://1rpc.io/eth");
        env::remove_var("SMTP_HOST");
        env::remove_var("SMTP_USER");
        env::remove_var("SMTP_PASS");

        let config = Config::load().unwrap();
        let _notifier = NotificationService::new(&config);

        // Should create successfully even without SMTP config
        // The service will just skip sending emails
        assert!(config.smtp_host.is_none());
    }

    #[test]
    #[serial]
    fn test_notification_service_creation_with_smtp() {
        env::set_var("RPC_URL", "https://1rpc.io/eth");
        env::set_var("SMTP_HOST", "smtp.example.com");
        env::set_var("SMTP_USER", "user@example.com");
        env::set_var("SMTP_PASS", "password123");
        env::set_var("SMTP_FROM", "from@example.com");
        env::set_var("SMTP_TO", "to@example.com");

        let config = Config::load().unwrap();
        let _notifier = NotificationService::new(&config);

        // Should create successfully with SMTP config
        assert!(config.smtp_host.is_some());
        assert_eq!(config.smtp_host.unwrap(), "smtp.example.com");
    }
}
