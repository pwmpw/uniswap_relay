//! Uniswap Relay DApp Library
//! 
//! This library provides the core functionality for monitoring Uniswap events
//! via subgraphs and Redis pub/sub.

pub mod config;
pub mod error;
pub mod model;
pub mod redis;
pub mod service;
pub mod subgraph;
pub mod telemetry;
pub mod utils;

// Re-export commonly used types
pub use config::AppConfig;
pub use error::{DAppError, Result};
pub use model::{SwapEvent, SwapEventBuilder, TokenInfo, UniswapVersion};
pub use redis::RedisPublisher;
pub use service::swap_collector::SwapEventCollector;
pub use subgraph::SubgraphClient;
pub use telemetry::MetricsCollector; 