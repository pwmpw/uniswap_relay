use std::time::Duration;
use testcontainers::{clients::Cli, *};
use testcontainers_modules::redis::Redis;
use tokio::time::sleep;
use tracing::info;

use uniswap_relay::{
    config::AppConfig,
    error::Result,
    model::{SwapEvent, SwapEventBuilder, TokenInfo, UniswapVersion},
    redis::RedisPublisher,
    service::SwapEventCollector,
    subgraph::SubgraphClient,
    telemetry::MetricsCollector,
};

/// Test configuration for integration tests
struct TestConfig {
    redis_url: String,
    config: AppConfig,
}

impl TestConfig {
    fn new(redis_url: String) -> Self {
        let mut config = AppConfig::default();
        config.redis.url = redis_url.clone();
        config.redis.channel = "test_swaps".to_string();
        config.redis.timeout_ms = 5000;
        config.subgraph.polling_interval_seconds = 1;
        config.rate_limiting.max_subgraph_requests_per_second = 10;
        config.retry.max_attempts = 3;
        config.retry.initial_delay_ms = 100;
        config.monitoring.enable_metrics = true;
        config.monitoring.enable_health_checks = true;

        Self { redis_url, config }
    }
}

/// Test container manager for integration tests
struct TestContainers {
    docker: Cli,
    redis_container: Container<'static, Redis>,
}

impl TestContainers {
    async fn new() -> Result<Self> {
        let docker = Cli::default();
        let redis_container = docker.run(Redis::default());
        
        // Wait for Redis to be ready
        sleep(Duration::from_millis(2000)).await;
        
        Ok(Self {
            docker,
            redis_container,
        })
    }

    fn get_redis_url(&self) -> String {
        let host_port = self.redis_container.get_host_port_ipv4(6379);
        format!("redis://localhost:{}", host_port)
    }

    async fn cleanup(self) {
        drop(self.redis_container);
        drop(self.docker);
    }
}

/// Integration test for Redis connectivity and publishing
#[tokio::test]
async fn test_redis_integration() -> Result<()> {
    let containers = TestContainers::new().await?;
    let test_config = TestConfig::new(containers.get_redis_url());
    
    info!("Testing Redis integration with URL: {}", test_config.redis_url);
    
    // Test Redis publisher creation
    let redis_publisher = RedisPublisher::new(&test_config.config.redis)?;
    
    // Test connection
    redis_publisher.test_connection().await?;
    info!("Redis connection test passed");
    
    // Test publishing a single event
    let test_event = create_test_swap_event();
    redis_publisher.publish_event(&test_event).await?;
    info!("Single event publishing test passed");
    
    // Test batch publishing
    let test_events = vec![
        create_test_swap_event(),
        create_test_swap_event(),
        create_test_swap_event(),
    ];
    redis_publisher.publish_batch(&test_events).await?;
    info!("Batch event publishing test passed");
    
    // Cleanup
    containers.cleanup().await;
    Ok(())
}

/// Integration test for SwapEventCollector with Redis
#[tokio::test]
async fn test_swap_collector_integration() -> Result<()> {
    let containers = TestContainers::new().await?;
    let test_config = TestConfig::new(containers.get_redis_url());
    
    info!("Testing SwapEventCollector integration");
    
    // Create components
    let redis_publisher = RedisPublisher::new(&test_config.config.redis)?;
    let subgraph_client = SubgraphClient::new(&test_config.config.subgraph)?;
    let metrics_collector = MetricsCollector::new(&test_config.config.monitoring);
    
    let mut collector = SwapEventCollector::new(
        test_config.config,
        subgraph_client,
        redis_publisher,
        metrics_collector,
    );
    
    // Test collector initialization
    info!("Testing collector initialization");
    assert!(!collector.is_running());
    
    // Test health check
    let health_status = collector.health_check().await?;
    assert!(health_status);
    info!("Health check test passed");
    
    // Test builder method integration
    info!("Testing builder method integration");
    collector.run_all_builder_tests()?;
    info!("Builder method tests passed");
    
    // Test JSON event creation
    let json_test = collector.test_json_event_creation();
    assert!(json_test.is_ok());
    info!("JSON event creation test passed");
    
    // Test raw data event creation
    let raw_test = collector.test_raw_data_event_creation();
    assert!(raw_test.is_ok());
    info!("Raw data event creation test passed");
    
    // Test create with builder
    let builder_test = collector.test_create_with_builder();
    assert!(builder_test.is_ok());
    info!("Create with builder test passed");
    
    // Cleanup
    containers.cleanup().await;
    Ok(())
}

/// Integration test for metrics collection
#[tokio::test]
async fn test_metrics_integration() -> Result<()> {
    let containers = TestContainers::new().await?;
    let test_config = TestConfig::new(containers.get_redis_url());
    
    info!("Testing metrics integration");
    
    let metrics_collector = MetricsCollector::new(&test_config.config.monitoring);
    
    // Test metrics collection
    metrics_collector.record_events_processed(10);
    metrics_collector.record_events_dropped(2);
    metrics_collector.record_error();
    
    let metrics = metrics_collector.get_metrics();
    assert_eq!(metrics.events_processed, 10);
    assert_eq!(metrics.events_dropped, 2);
    assert_eq!(metrics.error_count, 1);
    
    info!("Metrics integration test passed");
    
    // Cleanup
    containers.cleanup().await;
    Ok(())
}

/// Integration test for error handling scenarios
#[tokio::test]
async fn test_error_handling_integration() -> Result<()> {
    let containers = TestContainers::new().await?;
    let test_config = TestConfig::new(containers.get_redis_url());
    
    info!("Testing error handling integration");
    
    // Test with invalid Redis URL (should fail gracefully)
    let mut invalid_config = test_config.config.clone();
    invalid_config.redis.url = "redis://invalid:6379".to_string();
    
    let redis_result = RedisPublisher::new(&invalid_config.redis);
    assert!(redis_result.is_err());
    info!("Invalid Redis URL error handling test passed");
    
    // Test with invalid subgraph URL
    let mut invalid_subgraph_config = test_config.config.clone();
    invalid_subgraph_config.subgraph.uniswap_v2_url = "http://invalid-url".to_string();
    
    let subgraph_result = SubgraphClient::new(&invalid_subgraph_config.subgraph);
    assert!(subgraph_result.is_err());
    info!("Invalid subgraph URL error handling test passed");
    
    // Cleanup
    containers.cleanup().await;
    Ok(())
}

/// Integration test for configuration validation
#[tokio::test]
async fn test_config_validation_integration() -> Result<()> {
    let containers = TestContainers::new().await?;
    let test_config = TestConfig::new(containers.get_redis_url());
    
    info!("Testing configuration validation integration");
    
    // Test valid configuration
    let validation_result = test_config.config.validate_comprehensive();
    assert!(validation_result.is_ok());
    info!("Valid configuration validation test passed");
    
    // Test invalid configuration
    let mut invalid_config = test_config.config.clone();
    invalid_config.redis.channel = "".to_string(); // Empty channel should fail validation
    
    let validation_result = invalid_config.validate_comprehensive();
    assert!(validation_result.is_err());
    info!("Invalid configuration validation test passed");
    
    // Cleanup
    containers.cleanup().await;
    Ok(())
}

/// Integration test for end-to-end event flow
#[tokio::test]
async fn test_end_to_end_event_flow() -> Result<()> {
    let containers = TestContainers::new().await?;
    let test_config = TestConfig::new(containers.get_redis_url());
    
    info!("Testing end-to-end event flow");
    
    // Create all components
    let redis_publisher = RedisPublisher::new(&test_config.config.redis)?;
    let subgraph_client = SubgraphClient::new(&test_config.config.subgraph)?;
    let metrics_collector = MetricsCollector::new(&test_config.config.monitoring);
    
    let mut collector = SwapEventCollector::new(
        test_config.config,
        subgraph_client,
        redis_publisher,
        metrics_collector,
    );
    
    // Test the complete flow
    info!("Testing complete event flow");
    
    // 1. Create test event using builder
    let test_event = create_test_swap_event();
    info!("Test event created: {}", test_event.id);
    
    // 2. Validate event data
    let validation_result = collector.validate_event_data(
        test_event.version.clone(),
        &test_event.transaction_hash,
        &test_event.pool_address,
        &test_event.token_in,
        &test_event.token_out,
        &test_event.amount_in,
        &test_event.amount_out,
        &test_event.user_address,
    );
    assert!(validation_result.is_ok());
    info!("Event validation passed");
    
    // 3. Test event serialization/deserialization
    let event_json = serde_json::to_string(&test_event)?;
    let deserialized_event: SwapEvent = serde_json::from_str(&event_json)?;
    assert_eq!(test_event.id, deserialized_event.id);
    info!("Event serialization/deserialization test passed");
    
    // 4. Test metrics recording
    let initial_metrics = collector.metrics_collector.get_metrics();
    collector.metrics_collector.record_events_processed(1);
    let updated_metrics = collector.metrics_collector.get_metrics();
    assert_eq!(updated_metrics.events_processed, initial_metrics.events_processed + 1);
    info!("Metrics recording test passed");
    
    // Cleanup
    containers.cleanup().await;
    Ok(())
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

/// Integration test for concurrent operations
#[tokio::test]
async fn test_concurrent_operations() -> Result<()> {
    let containers = TestContainers::new().await?;
    let test_config = TestConfig::new(containers.get_redis_url());
    
    info!("Testing concurrent operations");
    
    let redis_publisher = RedisPublisher::new(&test_config.config.redis)?;
    
    // Test concurrent publishing
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let publisher = redis_publisher.clone();
            let event = create_test_swap_event();
            tokio::spawn(async move {
                publisher.publish_event(&event).await
            })
        })
        .collect();
    
    // Wait for all operations to complete
    for handle in handles {
        let result = handle.await?;
        assert!(result.is_ok());
    }
    
    info!("Concurrent operations test passed");
    
    // Cleanup
    containers.cleanup().await;
    Ok(())
}

/// Integration test for Redis connection resilience
#[tokio::test]
async fn test_redis_connection_resilience() -> Result<()> {
    let containers = TestContainers::new().await?;
    let test_config = TestConfig::new(containers.get_redis_url());
    
    info!("Testing Redis connection resilience");
    
    let redis_publisher = RedisPublisher::new(&test_config.config.redis)?;
    
    // Test multiple connection attempts
    for i in 0..5 {
        let result = redis_publisher.test_connection().await;
        assert!(result.is_ok(), "Connection attempt {} failed", i);
        sleep(Duration::from_millis(100)).await;
    }
    
    info!("Redis connection resilience test passed");
    
    // Cleanup
    containers.cleanup().await;
    Ok(())
}

/// Integration test for configuration hot-reloading simulation
#[tokio::test]
async fn test_configuration_changes() -> Result<()> {
    let containers = TestContainers::new().await?;
    let test_config = TestConfig::new(containers.get_redis_url());
    
    info!("Testing configuration changes");
    
    // Test with different configurations
    let mut config1 = test_config.config.clone();
    config1.rate_limiting.max_subgraph_requests_per_second = 5;
    
    let mut config2 = test_config.config.clone();
    config2.rate_limiting.max_subgraph_requests_per_second = 20;
    
    // Both should be valid
    assert!(config1.validate_comprehensive().is_ok());
    assert!(config2.validate_comprehensive().is_ok());
    
    // Test environment detection
    assert!(!config1.is_production());
    assert!(config1.is_development());
    
    info!("Configuration changes test passed");
    
    // Cleanup
    containers.cleanup().await;
    Ok(())
} 