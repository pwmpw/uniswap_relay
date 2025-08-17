use std::time::Duration;
use testcontainers::{clients::Cli, *};
use testcontainers_modules::redis::Redis;
use tokio::time::sleep;

use uniswap_relay::{
    config::AppConfig,
    error::Result,
    model::{SwapEvent, SwapEventBuilder, TokenInfo, UniswapVersion},
    redis::RedisPublisher,
    telemetry::MetricsCollector,
};

/// Simple test configuration for integration tests
fn create_test_config(redis_url: String) -> AppConfig {
    let mut config = AppConfig::default();
    config.redis.url = redis_url;
    config.redis.channel = "test_swaps".to_string();
    config.redis.timeout_ms = 5000;
    config.monitoring.enable_metrics = true;
    config.monitoring.enable_health_checks = true;
    config
}

/// Test container manager for integration tests
struct TestContainers {
    _docker: Cli,
    redis_container: Container<'static, Redis>,
}

impl TestContainers {
    async fn new() -> Result<Self> {
        let docker = Cli::default();
        let redis_container = docker.run(Redis::default());
        
        // Wait for Redis to be ready
        sleep(Duration::from_millis(2000)).await;
        
        Ok(Self {
            _docker: docker,
            redis_container,
        })
    }

    fn get_redis_url(&self) -> String {
        let host_port = self.redis_container.get_host_port_ipv4(6379);
        format!("redis://localhost:{}", host_port)
    }
}

/// Helper function to create a test swap event
fn create_test_swap_event() -> SwapEvent {
    SwapEventBuilder::default()
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
        .expect("Failed to build test event")
}

/// Basic integration test for Redis connectivity
#[tokio::test]
async fn test_redis_basic_integration() -> Result<()> {
    let containers = TestContainers::new().await?;
    let test_config = create_test_config(containers.get_redis_url());
    
    println!("Testing Redis integration with URL: {}", test_config.redis.url);
    
    // Test Redis publisher creation
    let redis_publisher = RedisPublisher::new(test_config).await?;
    
    // Test connection
    redis_publisher.test_connection().await?;
    println!("Redis connection test passed");
    
    // Test publishing a single event
    let test_event = create_test_swap_event();
    redis_publisher.publish_event(&test_event).await?;
    println!("Single event publishing test passed");
    
    // Test batch publishing
    let test_events = vec![
        create_test_swap_event(),
        create_test_swap_event(),
        create_test_swap_event(),
    ];
    redis_publisher.publish_batch(&test_events).await?;
    println!("Batch event publishing test passed");
    
    Ok(())
}

/// Basic integration test for metrics collection
#[tokio::test]
async fn test_metrics_basic_integration() -> Result<()> {
    let test_config = create_test_config("redis://localhost:6379".to_string());
    
    println!("Testing metrics integration");
    
    let metrics_collector = MetricsCollector::new(test_config.monitoring);
    
    // Test metrics collection
    metrics_collector.record_events_processed(10);
    metrics_collector.record_events_dropped(2);
    metrics_collector.record_error();
    
    let metrics = metrics_collector.get_metrics();
    assert_eq!(metrics.events_processed_total, 10);
    assert_eq!(metrics.errors_total, 1);
    
    println!("Metrics integration test passed");
    
    Ok(())
}

/// Basic integration test for configuration validation
#[tokio::test]
async fn test_config_basic_integration() -> Result<()> {
    let test_config = create_test_config("redis://localhost:6379".to_string());
    
    println!("Testing configuration validation integration");
    
    // Test valid configuration
    let validation_result = test_config.validate_comprehensive();
    assert!(validation_result.is_ok());
    println!("Valid configuration validation test passed");
    
    // Test invalid configuration
    let mut invalid_config = test_config.clone();
    invalid_config.redis.channel = "".to_string(); // Empty channel should fail validation
    
    let validation_result = invalid_config.validate_comprehensive();
    assert!(validation_result.is_err());
    println!("Invalid configuration validation test passed");
    
    Ok(())
}

/// Basic integration test for SwapEventBuilder
#[tokio::test]
async fn test_swap_event_builder_integration() -> Result<()> {
    println!("Testing SwapEventBuilder integration");
    
    // Test V2 event creation
    let v2_event = create_test_swap_event();
    assert_eq!(v2_event.version, UniswapVersion::V2);
    assert!(!v2_event.transaction_hash.is_empty());
    println!("V2 event creation test passed");
    
    // Test V3 event creation
    let v3_event = SwapEventBuilder::default()
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
        .build()?;
    
    assert_eq!(v3_event.version, UniswapVersion::V3);
    assert!(!v3_event.transaction_hash.is_empty());
    println!("V3 event creation test passed");
    
    Ok(())
} 