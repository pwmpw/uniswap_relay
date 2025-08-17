use std::sync::Once;
use tracing_subscriber::{fmt, EnvFilter};

use uniswap_relay::{
    config::AppConfig,
    error::Result,
    model::{SwapEvent, SwapEventBuilder, TokenInfo, UniswapVersion},
    redis::RedisPublisher,
    service::SwapEventCollector,
    subgraph::SubgraphClient,
    telemetry::MetricsCollector,
};

static INIT: Once = Once::new();

/// Initialize test logging once
pub fn init_test_logging() {
    INIT.call_once(|| {
        let filter = EnvFilter::from_default_env()
            .add_directive("uniswap_relay=debug".parse().unwrap())
            .add_directive("testcontainers=info".parse().unwrap());

        fmt::Subscriber::builder()
            .with_env_filter(filter)
            .with_target(false)
            .with_thread_ids(true)
            .with_thread_names(true)
            .init();
    });
}

/// Test configuration builder
pub struct TestConfigBuilder {
    config: AppConfig,
}

impl TestConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: AppConfig::default(),
        }
    }

    pub fn with_redis_url(mut self, url: &str) -> Self {
        self.config.redis.url = url.to_string();
        self
    }

    pub fn with_redis_channel(mut self, channel: &str) -> Self {
        self.config.redis.channel = channel.to_string();
        self
    }

    pub fn with_redis_timeout(mut self, timeout_ms: u64) -> Self {
        self.config.redis.timeout_ms = timeout_ms;
        self
    }

    pub fn with_subgraph_polling_interval(mut self, seconds: u64) -> Self {
        self.config.subgraph.polling_interval_seconds = seconds;
        self
    }

    pub fn with_rate_limiting(mut self, max_requests: u64, burst_size: u64) -> Self {
        self.config.rate_limiting.max_subgraph_requests_per_second = max_requests;
        self.config.rate_limiting.burst_size = burst_size;
        self
    }

    pub fn with_retry_config(mut self, max_attempts: u32, initial_delay: u64) -> Self {
        self.config.retry.max_attempts = max_attempts;
        self.config.retry.initial_delay_ms = initial_delay;
        self
    }

    pub fn with_monitoring(mut self, enable_metrics: bool, enable_health_checks: bool) -> Self {
        self.config.monitoring.enable_metrics = enable_metrics;
        self.config.monitoring.enable_health_checks = enable_health_checks;
        self
    }

    pub fn build(self) -> AppConfig {
        self.config
    }
}

impl Default for TestConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Test data generator
pub struct TestDataGenerator;

impl TestDataGenerator {
    /// Generate a valid V2 swap event
    pub fn v2_swap_event() -> SwapEvent {
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
            .expect("Failed to build V2 test event")
    }

    /// Generate a valid V3 swap event
    pub fn v3_swap_event() -> SwapEvent {
        SwapEventBuilder::default()
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
            .expect("Failed to build V3 test event")
    }

    /// Generate multiple swap events
    pub fn multiple_swap_events(count: usize) -> Vec<SwapEvent> {
        (0..count)
            .map(|i| {
                let mut event = Self::v2_swap_event();
                event.transaction_hash = format!("0x{:064x}", i);
                event
            })
            .collect()
    }

    /// Generate invalid swap event data
    pub fn invalid_swap_event() -> SwapEvent {
        SwapEventBuilder::default()
            .version(UniswapVersion::V2)
            .transaction_hash("".to_string()) // Invalid: empty transaction hash
            .pool_address("invalid_address".to_string()) // Invalid: not 0x prefixed
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
            .amount_in("invalid_amount".to_string()) // Invalid: not numeric
            .amount_out("0.0005".to_string())
            .user_address("0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6".to_string())
            .build()
            .expect("Failed to build invalid test event")
    }
}

/// Test component factory
pub struct TestComponentFactory;

impl TestComponentFactory {
    /// Create a Redis publisher for testing
    pub fn redis_publisher(config: &AppConfig) -> Result<RedisPublisher> {
        RedisPublisher::new(&config.redis)
    }

    /// Create a subgraph client for testing
    pub fn subgraph_client(config: &AppConfig) -> Result<SubgraphClient> {
        SubgraphClient::new(&config.subgraph)
    }

    /// Create a metrics collector for testing
    pub fn metrics_collector(config: &AppConfig) -> MetricsCollector {
        MetricsCollector::new(&config.monitoring)
    }

    /// Create a complete SwapEventCollector for testing
    pub fn swap_collector(config: AppConfig) -> Result<SwapEventCollector> {
        let redis_publisher = Self::redis_publisher(&config)?;
        let subgraph_client = Self::subgraph_client(&config)?;
        let metrics_collector = Self::metrics_collector(&config);

        Ok(SwapEventCollector::new(
            config,
            subgraph_client,
            redis_publisher,
            metrics_collector,
        ))
    }
}

/// Test assertions helper
pub struct TestAssertions;

impl TestAssertions {
    /// Assert that a result is ok and contains expected value
    pub fn assert_ok<T, E>(result: Result<T, E>, expected: T)
    where
        T: PartialEq + std::fmt::Debug,
        E: std::fmt::Debug,
    {
        match result {
            Ok(value) => assert_eq!(value, expected, "Expected {:?}, got {:?}", expected, value),
            Err(e) => panic!("Expected Ok({:?}), got Err({:?})", expected, e),
        }
    }

    /// Assert that a result is an error
    pub fn assert_err<T, E>(result: Result<T, E>) -> E
    where
        T: std::fmt::Debug,
        E: std::fmt::Debug,
    {
        match result {
            Ok(value) => panic!("Expected Err, got Ok({:?})", value),
            Err(e) => e,
        }
    }

    /// Assert that a condition is true with a custom message
    pub fn assert_true(condition: bool, message: &str) {
        assert!(condition, "{}", message);
    }

    /// Assert that two values are equal with a custom message
    pub fn assert_eq<T>(left: T, right: T, message: &str)
    where
        T: PartialEq + std::fmt::Debug,
    {
        assert_eq!(left, right, "{}", message);
    }
}

/// Test cleanup utilities
pub struct TestCleanup;

impl TestCleanup {
    /// Wait for a condition to be true with timeout
    pub async fn wait_for_condition<F, Fut>(condition: F, timeout_ms: u64) -> bool
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);

        while start.elapsed() < timeout {
            if condition().await {
                return true;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        false
    }

    /// Clean up test data
    pub async fn cleanup_test_data(redis_publisher: &RedisPublisher) -> Result<()> {
        // Clear test channel
        let _: Result<()> = redis_publisher
            .redis_client
            .del("test_swaps")
            .await
            .map_err(|e| uniswap_relay::error::DAppError::Redis(e.into()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = TestConfigBuilder::new()
            .with_redis_url("redis://localhost:6379")
            .with_redis_channel("test")
            .with_rate_limiting(5, 10)
            .build();

        assert_eq!(config.redis.url, "redis://localhost:6379");
        assert_eq!(config.redis.channel, "test");
        assert_eq!(config.rate_limiting.max_subgraph_requests_per_second, 5);
        assert_eq!(config.rate_limiting.burst_size, 10);
    }

    #[test]
    fn test_data_generator() {
        let v2_event = TestDataGenerator::v2_swap_event();
        assert_eq!(v2_event.version, UniswapVersion::V2);
        assert!(!v2_event.transaction_hash.is_empty());

        let v3_event = TestDataGenerator::v3_swap_event();
        assert_eq!(v3_event.version, UniswapVersion::V3);
        assert!(!v3_event.transaction_hash.is_empty());

        let multiple_events = TestDataGenerator::multiple_swap_events(3);
        assert_eq!(multiple_events.len(), 3);
    }

    #[test]
    fn test_assertions() {
        TestAssertions::assert_true(true, "This should pass");
        TestAssertions::assert_eq(1, 1, "These should be equal");

        let ok_result: Result<i32, &str> = Ok(42);
        TestAssertions::assert_ok(ok_result, 42);

        let err_result: Result<i32, &str> = Err("error");
        let error = TestAssertions::assert_err(err_result);
        assert_eq!(error, "error");
    }
} 