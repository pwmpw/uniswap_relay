use uniswap_relay::{
    config::AppConfig,
    error::Result,
    model::{SwapEvent, SwapEventBuilder, TokenInfo, UniswapVersion},
};

/// Test SwapEventBuilder functionality
#[test]
fn test_swap_event_builder() -> Result<()> {
    let event = SwapEventBuilder::default()
        .version(UniswapVersion::V2)
        .transaction_hash("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string())
        .pool_address("0x8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8".to_string())
        .token_in(TokenInfo {
            address: "0xA0b86a33E6441b8c4C3B1b1ef4F2faD6244b51a".to_string(),
            symbol: "USDC".to_string(),
            name: "USD Coin".to_string(),
            decimals: 6,
            logo_uri: None,
            price_usd: None,
            market_cap: None,
        })
        .token_out(TokenInfo {
            address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
            symbol: "WETH".to_string(),
            name: "Wrapped Ether".to_string(),
            decimals: 18,
            logo_uri: None,
            price_usd: None,
            market_cap: None,
        })
        .amount_in("1000000".to_string())
        .amount_out("0.0005".to_string())
        .user_address("0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6".to_string())
        .build()
        .map_err(|e| uniswap_relay::error::DAppError::Internal(format!("Builder failed: {}", e)))?;

    assert_eq!(event.version, UniswapVersion::V2);
    assert_eq!(event.pool_address, "0x8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8");
    assert_eq!(event.token_in.symbol, "USDC");
    assert_eq!(event.token_out.symbol, "WETH");

    Ok(())
}

/// Test configuration validation
#[test]
fn test_config_validation() -> Result<()> {
    let config = AppConfig::default();
    
    // Test basic validation
    let result = config.validate();
    assert!(result.is_ok());
    
    // Test comprehensive validation
    let result = config.validate_comprehensive();
    assert!(result.is_ok());
    
    Ok(())
}

/// Test environment detection
#[test]
fn test_environment_detection() {
    let config = AppConfig::default();
    
    // Test environment methods
    assert!(!config.is_production());
    assert!(config.is_development());
}

/// Test error types
#[test]
fn test_error_types() {
    use uniswap_relay::error::{DAppError, EthereumError, SolanaError};
    
    // Test Ethereum error creation
    let eth_error = EthereumError::EventParsing("test error".to_string());
    let dapp_error = DAppError::Ethereum(eth_error);
    
    match dapp_error {
        DAppError::Ethereum(EthereumError::EventParsing(msg)) => {
            assert_eq!(msg, "test error");
        }
        _ => panic!("Expected Ethereum EventParsing error"),
    }
    
    // Test Solana error creation
    let sol_error = SolanaError::Instruction("test instruction error".to_string());
    let dapp_error = DAppError::Solana(sol_error);
    
    match dapp_error {
        DAppError::Solana(SolanaError::Instruction(msg)) => {
            assert_eq!(msg, "test instruction error");
        }
        _ => panic!("Expected Solana Instruction error"),
    }
}

/// Test SwapEvent serialization
#[test]
fn test_swap_event_serialization() -> Result<()> {
    let event = SwapEventBuilder::default()
        .version(UniswapVersion::V3)
        .transaction_hash("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string())
        .pool_address("0x8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8".to_string())
        .token_in(TokenInfo {
            address: "0xA0b86a33E6441b8c4C3B1b1ef4F2faD6244b51a".to_string(),
            symbol: "USDC".to_string(),
            name: "USD Coin".to_string(),
            decimals: 6,
            logo_uri: None,
            price_usd: None,
            market_cap: None,
        })
        .token_out(TokenInfo {
            address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
            symbol: "WETH".to_string(),
            name: "Wrapped Ether".to_string(),
            decimals: 18,
            logo_uri: None,
            price_usd: None,
            market_cap: None,
        })
        .amount_in("500000".to_string())
        .amount_out("0.00025".to_string())
        .user_address("0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6".to_string())
        .build()
        .map_err(|e| uniswap_relay::error::DAppError::Internal(format!("Builder failed: {}", e)))?;

    // Test JSON serialization
    let json = serde_json::to_string(&event)?;
    assert!(!json.is_empty());
    assert!(json.contains("USDC"));
    assert!(json.contains("WETH"));
    
    // Test JSON deserialization
    let deserialized: SwapEvent = serde_json::from_str(&json)?;
    assert_eq!(deserialized.id, event.id);
    assert_eq!(deserialized.version, event.version);
    
    Ok(())
}

/// Test configuration scenarios
#[test]
fn test_config_scenarios() -> Result<()> {
    let mut config = AppConfig::default();
    
    // Test rate limiting configuration
    config.rate_limiting.max_subgraph_requests_per_second = 100;
    config.rate_limiting.burst_size = 200;
    assert_eq!(config.rate_limiting.max_subgraph_requests_per_second, 100);
    assert_eq!(config.rate_limiting.burst_size, 200);
    
    // Test retry configuration
    config.retry.max_attempts = 5;
    config.retry.initial_delay_ms = 1000;
    config.retry.max_delay_ms = 30000;
    config.retry.backoff_multiplier = 2.5;
    assert_eq!(config.retry.max_attempts, 5);
    assert_eq!(config.retry.initial_delay_ms, 1000);
    assert_eq!(config.retry.max_delay_ms, 30000);
    assert_eq!(config.retry.backoff_multiplier, 2.5);
    
    // Test monitoring configuration
    config.monitoring.enable_metrics = true;
    config.monitoring.enable_health_checks = true;
    config.monitoring.metrics_interval_seconds = 30;
    assert!(config.monitoring.enable_metrics);
    assert!(config.monitoring.enable_health_checks);
    assert_eq!(config.monitoring.metrics_interval_seconds, 30);
    
    Ok(())
} 