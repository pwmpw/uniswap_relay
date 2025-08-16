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

        Ok(())
    }

    pub fn is_production(&self) -> bool {
        self.application.environment.to_lowercase() == "production"
    }

    pub fn is_development(&self) -> bool {
        self.application.environment.to_lowercase() == "development"
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
