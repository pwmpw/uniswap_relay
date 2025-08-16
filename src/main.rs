mod config;
mod error;
mod model;
mod redis;
mod service;
mod subgraph;
mod telemetry;
mod utils;

use crate::config::AppConfig;
use crate::error::Result;
use crate::redis::publisher::RedisPublisher;
use crate::service::swap_collector::SwapEventCollector;
use crate::subgraph::SubgraphClient;
use crate::telemetry::metrics::MetricsCollector;

use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging()?;

    info!("Starting Uniswap Relay DApp (Subgraph-only)...");

    // Load configuration
    let config = AppConfig::load().map_err(|e| {
        error!("Failed to load configuration: {}", e);
        crate::error::DAppError::Config(e.to_string())
    })?;

    // Validate configuration
    if let Err(e) = config.validate() {
        error!("Configuration validation failed: {}", e);
        std::process::exit(1);
    }

    info!("Configuration loaded successfully");

    // Initialize subgraph client
    let subgraph_client = SubgraphClient::new(config.clone());

    // Test subgraph connectivity
    subgraph_client.test_connectivity().await?;
    info!("Subgraph connectivity verified");

    // Initialize Redis publisher
    let redis_publisher = RedisPublisher::new(config.clone()).await?;

    // Test Redis connection
    redis_publisher.test_connection().await?;
    info!("Redis connection established");

    // Initialize metrics collector
    let metrics_collector = MetricsCollector::new(config.clone());

    // Initialize swap event collector
    let mut swap_collector = SwapEventCollector::new(
        config.clone(),
        subgraph_client,
        redis_publisher,
        metrics_collector,
    );

    // Start collecting events from subgraphs
    swap_collector.start_collecting().await?;

    info!("Uniswap Relay DApp started successfully");

    // Wait for shutdown signal
    wait_for_shutdown().await;

    info!("Shutting down Uniswap Relay DApp...");

    // Graceful shutdown
    swap_collector.shutdown().await?;

    info!("Uniswap Relay DApp shutdown complete");
    Ok(())
}

/// Initialize logging with structured JSON output
fn init_logging() -> Result<()> {
    let env_filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());

    let formatting_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .json();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(formatting_layer)
        .init();

    info!("Logging initialized");
    Ok(())
}

/// Wait for shutdown signal (Ctrl+C)
async fn wait_for_shutdown() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to listen for SIGTERM")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, initiating shutdown...");
        }
        _ = terminate => {
            info!("Received SIGTERM, initiating shutdown...");
        }
    }
}

/// Handle graceful shutdown
#[allow(dead_code)]
async fn handle_shutdown() {
    info!("Initiating graceful shutdown...");

    // Give components time to finish current operations
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    info!("Graceful shutdown complete");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_loading() {
        // Test that we can create a default configuration
        let config = AppConfig::default();
        assert!(!config.subgraph.uniswap_v2_url.is_empty());
        assert!(!config.subgraph.uniswap_v3_url.is_empty());
        assert!(!config.redis.url.is_empty());
    }

    #[tokio::test]
    async fn test_config_validation() {
        let config = AppConfig::default();
        let validation = config.validate();
        assert!(validation.is_ok());
    }
}
