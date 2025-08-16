use uniswap_relay_dapp::config::AppConfig;
use uniswap_relay_dapp::error::Result;

#[tokio::test]
async fn test_config_loading() -> Result<()> {
    // Test that we can load the default configuration
    let config = AppConfig::default();
    
    // Verify basic config structure
    assert!(!config.subgraph.uniswap_v2_url.is_empty());
    assert!(!config.subgraph.uniswap_v3_url.is_empty());
    assert!(!config.redis.url.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_config_validation() -> Result<()> {
    let config = AppConfig::default();
    
    // Test validation
    let validation = config.validate();
    assert!(validation.is_ok());
    
    Ok(())
}

#[test]
fn test_model_creation() {
    use uniswap_relay_dapp::model::{SwapEvent, UniswapVersion, TokenInfo, PoolInfo};
    
    let token_in = TokenInfo {
        address: "0x1234".to_string(),
        symbol: "TOKEN".to_string(),
        name: "Test Token".to_string(),
        decimals: 18,
        logo_uri: None,
        price_usd: None,
        market_cap: None,
    };
    
    let token_out = TokenInfo {
        address: "0x5678".to_string(),
        symbol: "OTHER".to_string(),
        name: "Other Token".to_string(),
        decimals: 18,
        logo_uri: None,
        price_usd: None,
        market_cap: None,
    };
    
    let event = SwapEvent::new(
        UniswapVersion::V3,
        "0xabcd".to_string(),
        "0xpool".to_string(),
        token_in,
        token_out,
        "1000000000000000000".to_string(),
        "500000000000000000".to_string(),
        "0xuser".to_string(),
    );
    
    assert_eq!(event.version, UniswapVersion::V3);
    assert_eq!(event.transaction_hash, "0xabcd");
    assert_eq!(event.pool_address, "0xpool");
}

#[test]
fn test_uniswap_version_display() {
    use uniswap_relay_dapp::model::UniswapVersion;
    
    assert_eq!(UniswapVersion::V2.to_string(), "v2");
    assert_eq!(UniswapVersion::V3.to_string(), "v3");
} 