#[cfg(test)]
mod address_validation_tests {
    use alloy::primitives::Address;
    use std::str::FromStr;

    #[test]
    fn test_valid_ethereum_address() {
        let addr_str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
        let result = Address::from_str(addr_str);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_ethereum_address_too_short() {
        let addr_str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB4";
        let result = Address::from_str(addr_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_ethereum_address_empty() {
        let addr_str = "";
        let result = Address::from_str(addr_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_ethereum_address_invalid_chars() {
        let addr_str = "0xG0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
        let result = Address::from_str(addr_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_lowercase_ethereum_address() {
        let addr_str = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";
        let result = Address::from_str(addr_str);
        assert!(result.is_ok());
    }

    #[test]
    fn test_uppercase_ethereum_address() {
        let addr_str = "0xA0B86991C6218B36C1D19D4A2E9EB0CE3606EB48";
        let result = Address::from_str(addr_str);
        assert!(result.is_ok());
    }

    #[test]
    fn test_zero_address() {
        let addr_str = "0x0000000000000000000000000000000000000000";
        let result = Address::from_str(addr_str);
        assert!(result.is_ok());
    }
}
