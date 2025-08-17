use uniswap_relay::{
    config::AppConfig,
    error::Result,
    model::{SwapEvent, SwapEventBuilder, TokenInfo, UniswapVersion},
};

/// Simple test for SwapEventBuilder
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

/// Simple test for configuration validation
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

/// Simple test for environment detection
#[test]
fn test_environment_detection() {
    let config = AppConfig::default();
    
    // Test environment methods
    assert!(!config.is_production());
    assert!(config.is_development());
}

/// Simple test for metrics structure
#[test]
fn test_metrics_structure() {
    use uniswap_relay::telemetry::metrics::Metrics;
    
    let metrics = Metrics::default();
    
    // Test that metrics have expected fields
    assert_eq!(metrics.events_processed_total, 0);
    assert_eq!(metrics.errors_total, 0);
    assert_eq!(metrics.latency_p50_ms, 0);
}

/// Simple test for error types
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