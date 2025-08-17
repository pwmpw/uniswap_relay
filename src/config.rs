use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;
use tracing::info;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub subgraph: SubgraphConfig,
    pub redis: RedisConfig,
    pub application: ApplicationConfig,
    pub monitoring: MonitoringConfig,
    pub rate_limiting: RateLimitingConfig,
    pub retry: RetryConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubgraphConfig {
    pub uniswap_v2_url: String,
    pub uniswap_v3_url: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub polling_interval_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub channel: String,
    pub connection_pool_size: u32,
    pub timeout_ms: u64,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApplicationConfig {
    pub log_level: String,
    pub environment: String,
    pub health_check_port: u16,
    pub metrics_port: u16,
    pub worker_threads: usize,
    pub max_concurrent_tasks: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MonitoringConfig {
    pub enable_metrics: bool,
    pub enable_health_checks: bool,
    pub enable_structured_logging: bool,
    pub log_format: String,
    pub metrics_interval_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitingConfig {
    pub max_subgraph_requests_per_second: u32,
    pub burst_size: u32,
    pub window_size_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

        info!("Loading configuration for environment: {}", environment);

        let mut config = Config::builder();

        // Load base configuration
        config = config.add_source(File::with_name("config/config").required(true));

        // Load environment-specific configuration
        let env_config_path = format!("config/{}", environment);
        if std::path::Path::new(&format!("{}.toml", env_config_path)).exists() {
            config = config.add_source(File::with_name(&env_config_path).required(false));
        }

        // Override with environment variables
        config = config.add_source(
            Environment::default()
                .prefix("APP")
                .separator("_")
                .ignore_empty(true),
        );

        // Build and deserialize configuration
        let app_config: AppConfig = config.build()?.try_deserialize()?;

        info!(
            "Configuration loaded successfully for environment: {}",
            environment
        );
        Ok(app_config)
    }

    pub fn validate(&self) -> Result<(), String> {
        // Validate Subgraph config
        if self.subgraph.uniswap_v2_url.is_empty() {
            return Err("Uniswap V2 subgraph URL is required".to_string());
        }
        if self.subgraph.uniswap_v3_url.is_empty() {
            return Err("Uniswap V3 subgraph URL is required".to_string());
        }

        // Validate Redis config
        if self.redis.url.is_empty() {
            return Err("Redis URL is required".to_string());
        }
        if self.redis.channel.is_empty() {
            return Err("Redis channel is required".to_string());
        }

        // Validate rate limiting config
        if self.rate_limiting.max_subgraph_requests_per_second == 0 {
            return Err("Rate limiting requests per second must be greater than 0".to_string());
        }
        if self.rate_limiting.burst_size == 0 {
            return Err("Rate limiting burst size must be greater than 0".to_string());
        }

        // Validate retry config
        if self.retry.max_attempts == 0 {
            return Err("Retry max attempts must be greater than 0".to_string());
        }
        if self.retry.initial_delay_ms == 0 {
            return Err("Retry initial delay must be greater than 0".to_string());
        }

        Ok(())
    }

    pub fn is_production(&self) -> bool {
        self.application.environment.to_lowercase() == "production"
    }

    pub fn is_development(&self) -> bool {
        self.application.environment.to_lowercase() == "development"
    }

    /// Validate configuration with detailed error reporting
    pub fn validate_detailed(&self) -> Result<(), crate::error::DAppError> {
        // Validate application config
        if self.application.log_level.is_empty() {
            return Err(crate::error::DAppError::Validation(
                "Log level is required".to_string(),
            ));
        }
        if self.application.environment.is_empty() {
            return Err(crate::error::DAppError::Validation(
                "Environment is required".to_string(),
            ));
        }

        // Validate monitoring config
        if self.monitoring.log_format.is_empty() {
            return Err(crate::error::DAppError::Validation(
                "Log format is required".to_string(),
            ));
        }

        // Validate rate limiting config
        if self.rate_limiting.max_subgraph_requests_per_second == 0 {
            return Err(crate::error::DAppError::RateLimit(
                "Rate limiting requests per second must be greater than 0".to_string(),
            ));
        }

        // Validate retry config
        if self.retry.max_attempts == 0 {
            return Err(crate::error::DAppError::Validation(
                "Retry max attempts must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if configuration has timeout issues
    pub fn check_timeout_config(&self) -> Result<(), crate::error::DAppError> {
        if self.subgraph.timeout_seconds == 0 {
            return Err(crate::error::DAppError::Timeout(
                "Subgraph timeout must be greater than 0".to_string(),
            ));
        }
        if self.redis.timeout_ms == 0 {
            return Err(crate::error::DAppError::Timeout(
                "Redis timeout must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }

    /// Check Redis configuration for potential issues
    pub fn check_redis_config(&self) -> Result<(), crate::error::DAppError> {
        if self.redis.connection_pool_size == 0 {
            return Err(crate::error::DAppError::Redis(
                crate::error::RedisError::Pool(
                    "Connection pool size must be greater than 0".to_string(),
                ),
            ));
        }
        if self.redis.retry_attempts == 0 {
            return Err(crate::error::DAppError::Redis(
                crate::error::RedisError::Connection(
                    "Retry attempts must be greater than 0".to_string(),
                ),
            ));
        }

        // Test Redis connection timeout scenarios
        if self.redis.timeout_ms > 30000 {
            return Err(crate::error::DAppError::Redis(
                crate::error::RedisError::timeout_error(
                    "Redis timeout too high (>30s)".to_string(),
                ),
            ));
        }

        // Test Redis subscribe scenarios
        if self.redis.channel.is_empty() {
            return Err(crate::error::DAppError::Redis(
                crate::error::RedisError::subscribe_error(
                    "Redis channel cannot be empty".to_string(),
                ),
            ));
        }

        Ok(())
    }

    /// Check subgraph configuration for potential issues
    pub fn check_subgraph_config(&self) -> Result<(), crate::error::DAppError> {
        if self.subgraph.max_retries == 0 {
            return Err(crate::error::DAppError::Subgraph(
                crate::error::SubgraphError::InvalidResponse(
                    "Max retries must be greater than 0".to_string(),
                ),
            ));
        }
        if self.subgraph.polling_interval_seconds == 0 {
            return Err(crate::error::DAppError::Subgraph(
                crate::error::SubgraphError::Timeout(
                    "Polling interval must be greater than 0".to_string(),
                ),
            ));
        }
        Ok(())
    }

    /// Check network configuration for potential issues
    pub fn check_network_config(&self) -> Result<(), crate::error::DAppError> {
        // Check if URLs are valid
        if !self.subgraph.uniswap_v2_url.starts_with("http") {
            return Err(crate::error::DAppError::Network(
                crate::error::NetworkError::Http("Invalid V2 subgraph URL format".to_string()),
            ));
        }
        if !self.subgraph.uniswap_v3_url.starts_with("http") {
            return Err(crate::error::DAppError::Network(
                crate::error::NetworkError::Http("Invalid V3 subgraph URL format".to_string()),
            ));
        }
        if !self.redis.url.starts_with("redis://") {
            return Err(crate::error::DAppError::Network(
                crate::error::NetworkError::Http("Invalid Redis URL format".to_string()),
            ));
        }

        // Test WebSocket scenarios
        if self.subgraph.uniswap_v2_url.contains("ws://")
            || self.subgraph.uniswap_v2_url.contains("wss://")
        {
            return Err(crate::error::DAppError::Network(
                crate::error::NetworkError::websocket_error(
                    "WebSocket URLs not supported for subgraphs".to_string(),
                ),
            ));
        }

        // Test DNS resolution scenarios
        if self.subgraph.uniswap_v2_url.contains("localhost")
            && !self.application.environment.eq("development")
        {
            return Err(crate::error::DAppError::Network(
                crate::error::NetworkError::dns_resolution_error(
                    "Localhost URLs only allowed in development".to_string(),
                ),
            ));
        }

        // Test TLS scenarios
        if self.subgraph.uniswap_v2_url.contains("http://")
            && self.application.environment.eq("production")
        {
            return Err(crate::error::DAppError::Network(
                crate::error::NetworkError::tls_error(
                    "HTTP URLs not allowed in production (use HTTPS)".to_string(),
                ),
            ));
        }

        Ok(())
    }

    /// Check serialization configuration for potential issues
    pub fn check_serialization_config(&self) -> Result<(), crate::error::DAppError> {
        // Check if log format is supported
        match self.monitoring.log_format.as_str() {
            "json" | "text" => Ok(()),
            "borsh" => Err(crate::error::DAppError::Serialization(
                crate::error::SerializationError::Borsh(
                    "Borsh format not supported for logging".to_string(),
                ),
            )),
            "hex" => Err(crate::error::DAppError::Serialization(
                crate::error::SerializationError::Hex(
                    "Hex format not supported for logging".to_string(),
                ),
            )),
            "base64" => Err(crate::error::DAppError::Serialization(
                crate::error::SerializationError::Base64(
                    "Base64 format not supported for logging".to_string(),
                ),
            )),
            _ => Err(crate::error::DAppError::Serialization(
                crate::error::SerializationError::Json("Unsupported log format".to_string()),
            )),
        }
    }

    /// Comprehensive configuration validation using all error types
    pub fn validate_comprehensive(&self) -> Result<(), crate::error::DAppError> {
        // Check environment-specific settings
        if self.is_production() {
            // Production-specific validations
            if self.application.log_level == "debug" {
                return Err(crate::error::DAppError::Validation(
                    "Production environment should not use debug logging".to_string(),
                ));
            }
            if !self.monitoring.enable_metrics {
                return Err(crate::error::DAppError::Validation(
                    "Production environment should enable metrics".to_string(),
                ));
            }
            if !self.monitoring.enable_health_checks {
                return Err(crate::error::DAppError::Validation(
                    "Production environment should enable health checks".to_string(),
                ));
            }
        } else if self.is_development() {
            // Development-specific validations
            if self.application.log_level == "error" {
                return Err(crate::error::DAppError::Validation(
                    "Development environment should use more verbose logging".to_string(),
                ));
            }
        }

        // Run all validation checks
        self.validate_detailed()?;
        self.check_timeout_config()?;
        self.check_redis_config()?;
        self.check_subgraph_config()?;
        self.check_network_config()?;
        self.check_serialization_config()?;
        self.check_ethereum_config()?;
        self.check_solana_config()?;

        Ok(())
    }

    /// Check Ethereum-specific configuration issues
    pub fn check_ethereum_config(&self) -> Result<(), crate::error::DAppError> {
        // Check for chain ID mismatches
        if self.subgraph.uniswap_v2_url.contains("mainnet")
            && self.subgraph.uniswap_v3_url.contains("testnet")
        {
            return Err(crate::error::DAppError::Ethereum(
                crate::error::EthereumError::ChainIdMismatch {
                    expected: 1, // Mainnet
                    actual: 5,   // Goerli testnet
                },
            ));
        }

        if self.subgraph.uniswap_v2_url.contains("testnet")
            && self.subgraph.uniswap_v3_url.contains("mainnet")
        {
            return Err(crate::error::DAppError::Ethereum(
                crate::error::EthereumError::ChainIdMismatch {
                    expected: 5, // Goerli testnet
                    actual: 1,   // Mainnet
                },
            ));
        }

        // Check for Ethereum-specific issues
        if self.subgraph.uniswap_v2_url.contains("mainnet")
            && self.application.environment.eq("development")
        {
            return Err(crate::error::DAppError::Ethereum(
                crate::error::EthereumError::Rpc(
                    "Mainnet URLs not recommended for development".to_string(),
                ),
            ));
        }

        if self.subgraph.uniswap_v3_url.contains("testnet")
            && self.application.environment.eq("production")
        {
            return Err(crate::error::DAppError::Ethereum(
                crate::error::EthereumError::Contract(
                    "Testnet URLs not allowed in production".to_string(),
                ),
            ));
        }

        // Check for invalid addresses
        if self
            .subgraph
            .uniswap_v2_url
            .contains("0x0000000000000000000000000000000000000000")
        {
            return Err(crate::error::DAppError::Ethereum(
                crate::error::EthereumError::InvalidAddress("Zero address not allowed".to_string()),
            ));
        }

        // Check for WebSocket URLs
        if self.subgraph.uniswap_v2_url.contains("ws://")
            || self.subgraph.uniswap_v2_url.contains("wss://")
        {
            return Err(crate::error::DAppError::Network(
                crate::error::NetworkError::websocket_error(
                    "WebSocket URLs not supported for Ethereum subgraphs".to_string(),
                ),
            ));
        }

        Ok(())
    }

    /// Check Solana-specific configuration issues
    pub fn check_solana_config(&self) -> Result<(), crate::error::DAppError> {
        // Check for Solana-specific issues (placeholder for future Solana support)
        if self.subgraph.uniswap_v2_url.contains("solana") {
            return Err(crate::error::DAppError::Solana(
                crate::error::SolanaError::Program(
                    "Solana subgraphs not yet supported".to_string(),
                ),
            ));
        }

        // Check for Solana RPC endpoints
        if self.subgraph.uniswap_v2_url.contains("solana-rpc") {
            return Err(crate::error::DAppError::Solana(
                crate::error::SolanaError::Rpc("Solana RPC not yet supported".to_string()),
            ));
        }

        // Check for Solana transaction formats
        if self.subgraph.uniswap_v2_url.contains("solana-tx") {
            return Err(crate::error::DAppError::Solana(
                crate::error::SolanaError::Transaction(
                    "Solana transactions not yet supported".to_string(),
                ),
            ));
        }

        // Check for Solana account formats
        if self.subgraph.uniswap_v2_url.contains("solana-account") {
            return Err(crate::error::DAppError::Solana(
                crate::error::SolanaError::Account("Solana accounts not yet supported".to_string()),
            ));
        }

        // Check for Solana commitment levels
        if self.subgraph.uniswap_v2_url.contains("commitment") {
            let commitment = self
                .subgraph
                .uniswap_v2_url
                .split("commitment=")
                .nth(1)
                .unwrap_or("");
            if !["processed", "confirmed", "finalized"].contains(&commitment) {
                return Err(crate::error::DAppError::Solana(
                    crate::error::SolanaError::commitment_error(format!(
                        "Invalid Solana commitment level: {}",
                        commitment
                    )),
                ));
            }
        }

        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            subgraph: SubgraphConfig {
                uniswap_v2_url: "https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v2"
                    .to_string(),
                uniswap_v3_url: "https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v3"
                    .to_string(),
                timeout_seconds: 30,
                max_retries: 3,
                polling_interval_seconds: 15,
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
                channel: "swap_events".to_string(),
                connection_pool_size: 10,
                timeout_ms: 5000,
                retry_attempts: 3,
                retry_delay_ms: 1000,
            },
            application: ApplicationConfig {
                log_level: "info".to_string(),
                environment: "development".to_string(),
                health_check_port: 8080,
                metrics_port: 9090,
                worker_threads: 4,
                max_concurrent_tasks: 100,
            },
            monitoring: MonitoringConfig {
                enable_metrics: true,
                enable_health_checks: true,
                enable_structured_logging: true,
                log_format: "json".to_string(),
                metrics_interval_seconds: 15,
            },
            rate_limiting: RateLimitingConfig {
                max_subgraph_requests_per_second: 50,
                burst_size: 100,
                window_size_seconds: 60,
            },
            retry: RetryConfig {
                max_attempts: 3,
                initial_delay_ms: 1000,
                max_delay_ms: 10000,
                backoff_multiplier: 2.0,
            },
        }
    }
}
