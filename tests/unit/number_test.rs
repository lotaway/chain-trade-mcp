#[cfg(test)]
mod number_formatting_tests {
    use alloy::primitives::{utils::format_units, utils::parse_units, U256};

    #[test]
    fn test_format_eth_balance() {
        let balance = U256::from(1_000_000_000_000_000_000u64); // 1 ETH
        let formatted = format_units(balance, 18).unwrap();
        assert_eq!(formatted, "1.000000000000000000");
    }

    #[test]
    fn test_format_usdc_balance() {
        let balance = U256::from(1_000_000u64); // 1 USDC
        let formatted = format_units(balance, 6).unwrap();
        assert_eq!(formatted, "1.000000");
    }

    #[test]
    fn test_format_zero_balance() {
        let balance = U256::ZERO;
        let formatted = format_units(balance, 18).unwrap();
        assert_eq!(formatted, "0.000000000000000000");
    }

    #[test]
    fn test_format_large_balance() {
        let balance = U256::from(1_000_000_000_000_000_000_000u128); // 1000 ETH
        let formatted = format_units(balance, 18).unwrap();
        assert_eq!(formatted, "1000.000000000000000000");
    }

    #[test]
    fn test_parse_eth_amount() {
        let amount: U256 = parse_units("1.5", 18).unwrap().into();
        let expected = U256::from(1_500_000_000_000_000_000u64);
        assert_eq!(amount, expected);
    }

    #[test]
    fn test_parse_usdc_amount() {
        let amount: U256 = parse_units("100.5", 6).unwrap().into();
        let expected = U256::from(100_500_000u64);
        assert_eq!(amount, expected);
    }

    #[test]
    fn test_parse_zero_amount() {
        let amount: U256 = parse_units("0", 18).unwrap().into();
        assert_eq!(amount, U256::ZERO);
    }

    #[test]
    fn test_parse_decimal_amount() {
        let amount: U256 = parse_units("0.000001", 18).unwrap().into();
        let expected = U256::from(1_000_000_000_000u64);
        assert_eq!(amount, expected);
    }

    #[test]
    fn test_roundtrip_formatting() {
        let original = "1.5";
        let parsed: U256 = parse_units(original, 18).unwrap().into();
        let formatted = format_units(parsed, 18).unwrap();
        assert_eq!(formatted, "1.500000000000000000");
    }
}
