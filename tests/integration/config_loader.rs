use std::path::Path;
use uniswap_relay::{
    config::AppConfig,
    error::{DAppError, Result},
};

/// Test configuration loader
pub struct TestConfigLoader;

impl TestConfigLoader {
    /// Load test configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<AppConfig> {
        let config_content = std::fs::read_to_string(path)
            .map_err(|e| DAppError::Config(format!("Failed to read test config: {}", e)))?;

        let config: AppConfig = toml::from_str(&config_content)
            .map_err(|e| DAppError::Config(format!("Failed to parse test config: {}", e)))?;

        Ok(config)
    }

    /// Load test configuration with environment overrides
    pub fn from_file_with_env<P: AsRef<Path>>(path: P, env: &str) -> Result<AppConfig> {
        let mut config = Self::from_file(path)?;
        
        // Override environment
        config.application.environment = env.to_string();
        
        // Override Redis URL for test environment
        if env == "test" {
            config.redis.url = "redis://localhost:6379".to_string();
            config.redis.channel = "test_swaps".to_string();
        }
        
        Ok(config)
    }

    /// Create a minimal test configuration
    pub fn minimal() -> AppConfig {
        let mut config = AppConfig::default();
        
        // Set minimal test values
        config.application.name = "uniswap_relay_test".to_string();
        config.application.version = "0.1.0".to_string();
        config.application.log_level = "debug".to_string();
        config.application.environment = "test".to_string();
        
        config.redis.url = "redis://localhost:6379".to_string();
        config.redis.channel = "test_swaps".to_string();
        config.redis.timeout_ms = 5000;
        
        config.subgraph.polling_interval_seconds = 1;
        config.subgraph.request_timeout_ms = 5000;
        
        config.rate_limiting.max_subgraph_requests_per_second = 10;
        config.rate_limiting.burst_size = 20;
        
        config.retry.max_attempts = 3;
        config.retry.initial_delay_ms = 100;
        
        config.monitoring.enable_metrics = true;
        config.monitoring.enable_health_checks = true;
        config.monitoring.metrics_interval_seconds = 1;
        
        config
    }

    /// Create a production-like test configuration
    pub fn production_like() -> AppConfig {
        let mut config = Self::minimal();
        
        // Override with production-like values
        config.application.environment = "production".to_string();
        config.application.log_level = "info".to_string();
        
        config.rate_limiting.max_subgraph_requests_per_second = 100;
        config.rate_limiting.burst_size = 200;
        
        config.retry.max_attempts = 5;
        config.retry.initial_delay_ms = 1000;
        config.retry.max_delay_ms = 30000;
        
        config.monitoring.metrics_interval_seconds = 30;
        
        config
    }

    /// Create a configuration for specific test scenarios
    pub fn for_scenario(scenario: &str) -> AppConfig {
        match scenario {
            "rate_limiting" => {
                let mut config = Self::minimal();
                config.rate_limiting.max_subgraph_requests_per_second = 1;
                config.rate_limiting.burst_size = 1;
                config
            }
            "retry" => {
                let mut config = Self::minimal();
                config.retry.max_attempts = 1;
                config.retry.initial_delay_ms = 10;
                config
            }
            "timeout" => {
                let mut config = Self::minimal();
                config.redis.timeout_ms = 100;
                config.subgraph.request_timeout_ms = 100;
                config
            }
            "high_load" => {
                let mut config = Self::minimal();
                config.rate_limiting.max_subgraph_requests_per_second = 1000;
                config.rate_limiting.burst_size = 2000;
                config.retry.max_attempts = 10;
                config
            }
            _ => Self::minimal(),
        }
    }

    /// Validate test configuration
    pub fn validate_test_config(config: &AppConfig) -> Result<()> {
        // Ensure test-specific requirements are met
        if config.application.environment != "test" {
            return Err(DAppError::Config(
                "Test configuration must have environment set to 'test'".to_string(),
            ));
        }

        if config.redis.channel.is_empty() {
            return Err(DAppError::Config(
                "Test configuration must have a non-empty Redis channel".to_string(),
            ));
        }

        if config.subgraph.polling_interval_seconds < 1 {
            return Err(DAppError::Config(
                "Test configuration must have polling interval >= 1 second".to_string(),
            ));
        }

        // Run comprehensive validation
        config.validate_comprehensive()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_config() {
        let config = TestConfigLoader::minimal();
        assert_eq!(config.application.environment, "test");
        assert_eq!(config.redis.channel, "test_swaps");
        assert!(config.monitoring.enable_metrics);
    }

    #[test]
    fn test_production_like_config() {
        let config = TestConfigLoader::production_like();
        assert_eq!(config.application.environment, "production");
        assert_eq!(config.rate_limiting.max_subgraph_requests_per_second, 100);
    }

    #[test]
    fn test_scenario_configs() {
        let rate_limiting_config = TestConfigLoader::for_scenario("rate_limiting");
        assert_eq!(rate_limiting_config.rate_limiting.max_subgraph_requests_per_second, 1);

        let retry_config = TestConfigLoader::for_scenario("retry");
        assert_eq!(retry_config.retry.max_attempts, 1);

        let timeout_config = TestConfigLoader::for_scenario("timeout");
        assert_eq!(timeout_config.redis.timeout_ms, 100);
    }

    #[test]
    fn test_config_validation() {
        let valid_config = TestConfigLoader::minimal();
        assert!(TestConfigLoader::validate_test_config(&valid_config).is_ok());

        let mut invalid_config = TestConfigLoader::minimal();
        invalid_config.application.environment = "production".to_string();
        assert!(TestConfigLoader::validate_test_config(&invalid_config).is_err());
    }
} 