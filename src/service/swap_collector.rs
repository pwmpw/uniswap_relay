use crate::error::Result;
use crate::model::{SwapEvent, UniswapVersion, TokenInfo, PoolInfo};
use crate::config::AppConfig;
use crate::redis::RedisPublisher;
use crate::telemetry::MetricsCollector;
use crate::subgraph::SubgraphClient;
use tokio::time::{Duration, interval};
use tracing::{info, warn, error, debug};

/// Service for collecting swap events from Uniswap subgraphs
pub struct SwapEventCollector {
    config: AppConfig,
    subgraph_client: SubgraphClient,
    redis_publisher: RedisPublisher,
    metrics_collector: MetricsCollector,
    is_running: bool,
    last_v2_block: u64,
    last_v3_block: u64,
}

impl SwapEventCollector {
    /// Create a new swap event collector
    pub fn new(
        config: AppConfig,
        subgraph_client: SubgraphClient,
        redis_publisher: RedisPublisher,
        metrics_collector: MetricsCollector,
    ) -> Self {
        Self {
            config,
            subgraph_client,
            redis_publisher,
            metrics_collector,
            is_running: false,
            last_v2_block: 0,
            last_v3_block: 0,
        }
    }

    /// Start collecting events from subgraphs
    pub async fn start_collecting(&mut self) -> Result<()> {
        if self.is_running {
            warn!("Swap event collector already running");
            return Ok(());
        }

        info!("Starting Uniswap swap event collection...");

        // Start background collection tasks
        self.start_v2_collection().await?;
        self.start_v3_collection().await?;

        self.is_running = true;
        info!("Uniswap swap event collection started successfully");

        Ok(())
    }

    /// Stop collecting events
    pub async fn stop_collecting(&mut self) -> Result<()> {
        if !self.is_running {
            warn!("Swap event collector not running");
            return Ok(());
        }

        info!("Stopping Uniswap swap event collection...");
        self.is_running = false;
        info!("Uniswap swap event collection stopped");

        Ok(())
    }

    /// Start V2 collection task
    async fn start_v2_collection(&mut self) -> Result<()> {
        let config = self.config.clone();
        let subgraph_client = self.subgraph_client.clone();
        let redis_publisher = self.redis_publisher.clone();
        let metrics_collector = self.metrics_collector.clone();
        let mut interval_timer = interval(Duration::from_secs(config.subgraph.polling_interval_seconds));

        tokio::spawn(async move {
            loop {
                interval_timer.tick().await;
                
                if let Err(e) = Self::collect_v2_events(&subgraph_client, &redis_publisher, &metrics_collector).await {
                    error!("Error collecting V2 events: {}", e);
                    metrics_collector.record_error();
                }
            }
        });

        Ok(())
    }

    /// Start V3 collection task
    async fn start_v3_collection(&mut self) -> Result<()> {
        let config = self.config.clone();
        let subgraph_client = self.subgraph_client.clone();
        let redis_publisher = self.redis_publisher.clone();
        let metrics_collector = self.metrics_collector.clone();
        let mut interval_timer = interval(Duration::from_secs(config.subgraph.polling_interval_seconds));

        tokio::spawn(async move {
            loop {
                interval_timer.tick().await;
                
                if let Err(e) = Self::collect_v3_events(&subgraph_client, &redis_publisher, &metrics_collector).await {
                    error!("Error collecting V3 events: {}", e);
                    metrics_collector.record_error();
                }
            }
        });

        Ok(())
    }

    /// Collect V2 swap events
    async fn collect_v2_events(
        subgraph_client: &SubgraphClient,
        redis_publisher: &RedisPublisher,
        metrics_collector: &MetricsCollector,
    ) -> Result<()> {
        let query = r#"
            query GetRecentSwaps($first: Int!) {
                swaps(
                    first: $first
                    orderBy: timestamp
                    orderDirection: desc
                ) {
                    id
                    timestamp
                    pair {
                        id
                        token0 {
                            id
                            symbol
                            name
                            decimals
                        }
                        token1 {
                            id
                            symbol
                            name
                            decimals
                        }
                        reserve0
                        reserve1
                        volume_usd
                    }
                    sender
                    amount0_in
                    amount1_in
                    amount0_out
                    amount1_out
                    to
                    log_index
                    amount_usd
                }
            }
        "#;

        let variables = serde_json::json!({
            "first": 100
        });

        let result = subgraph_client.query_uniswap_v2(query, Some(variables)).await?;
        
        if let Some(data) = result.data {
            if let Some(swaps) = data.get("swaps") {
                if let Some(swaps_array) = swaps.as_array() {
                    let mut events = Vec::new();
                    
                    for swap_data in swaps_array {
                        if let Ok(swap_event) = Self::parse_v2_swap_event(swap_data) {
                            events.push(swap_event);
                        }
                    }
                    
                    if !events.is_empty() {
                        debug!("Collected {} V2 swap events", events.len());
                        
                        // Publish events to Redis
                        redis_publisher.publish_batch(&events).await?;
                        
                        // Update metrics
                        metrics_collector.record_events_processed(events.len() as u64);
                    }
                }
            }
        }

        Ok(())
    }

    /// Collect V3 swap events
    async fn collect_v3_events(
        subgraph_client: &SubgraphClient,
        redis_publisher: &RedisPublisher,
        metrics_collector: &MetricsCollector,
    ) -> Result<()> {
        let query = r#"
            query GetRecentSwaps($first: Int!) {
                swaps(
                    first: $first
                    orderBy: timestamp
                    orderDirection: desc
                ) {
                    id
                    timestamp
                    pool {
                        id
                        token0 {
                            id
                            symbol
                            name
                            decimals
                        }
                        token1 {
                            id
                            symbol
                            name
                            decimals
                        }
                        fee_tier
                        liquidity
                        volume_usd
                        fees_usd
                        total_value_locked_usd
                    }
                    token0
                    token1
                    sender
                    recipient
                    origin
                    amount0
                    amount1
                    amount_usd
                    sqrt_price_x96
                    liquidity
                    tick
                }
            }
        "#;

        let variables = serde_json::json!({
            "first": 100
        });

        let result = subgraph_client.query_uniswap_v3(query, Some(variables)).await?;
        
        if let Some(data) = result.data {
            if let Some(swaps) = data.get("swaps") {
                if let Some(swaps_array) = swaps.as_array() {
                    let mut events = Vec::new();
                    
                    for swap_data in swaps_array {
                        if let Ok(swap_event) = Self::parse_v3_swap_event(swap_data) {
                            events.push(swap_event);
                        }
                    }
                    
                    if !events.is_empty() {
                        debug!("Collected {} V3 swap events", events.len());
                        
                        // Publish events to Redis
                        redis_publisher.publish_batch(&events).await?;
                        
                        // Update metrics
                        metrics_collector.record_events_processed(events.len() as u64);
                    }
                }
            }
        }

        Ok(())
    }

    /// Parse V2 swap event from subgraph data
    fn parse_v2_swap_event(swap_data: &serde_json::Value) -> Result<SwapEvent> {
        let pair = swap_data.get("pair").ok_or_else(|| {
            crate::error::DAppError::Internal("Missing pair data".to_string())
        })?;

        let token0 = pair.get("token0").ok_or_else(|| {
            crate::error::DAppError::Internal("Missing token0 data".to_string())
        })?;

        let token1 = pair.get("token1").ok_or_else(|| {
            crate::error::DAppError::Internal("Missing token1 data".to_string())
        })?;

        let token_in = TokenInfo {
            address: token0.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            symbol: token0.get("symbol").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            name: token0.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            decimals: token0.get("decimals").and_then(|v| v.as_u64()).unwrap_or(18) as u8,
            logo_uri: None,
            price_usd: None,
            market_cap: None,
        };

        let token_out = TokenInfo {
            address: token1.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            symbol: token1.get("symbol").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            name: token1.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            decimals: token1.get("decimals").and_then(|v| v.as_u64()).unwrap_or(18) as u8,
            logo_uri: None,
            price_usd: None,
            market_cap: None,
        };

        let amount_in = swap_data.get("amount0_in").and_then(|v| v.as_str()).unwrap_or("0").to_string();
        let amount_out = swap_data.get("amount1_out").and_then(|v| v.as_str()).unwrap_or("0").to_string();
        let user_address = swap_data.get("sender").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let pool_address = pair.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let mut swap_event = SwapEvent::new(
            UniswapVersion::V2,
            swap_data.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            pool_address,
            token_in,
            token_out,
            amount_in,
            amount_out,
            user_address,
        );

        // Add pool information
        if let Some(pool_info) = Self::extract_pool_info(pair) {
            swap_event.add_pool_info(pool_info);
        }

        Ok(swap_event)
    }

    /// Parse V3 swap event from subgraph data
    fn parse_v3_swap_event(swap_data: &serde_json::Value) -> Result<SwapEvent> {
        let pool = swap_data.get("pool").ok_or_else(|| {
            crate::error::DAppError::Internal("Missing pool data".to_string())
        })?;

        let token0 = pool.get("token0").ok_or_else(|| {
            crate::error::DAppError::Internal("Missing token0 data".to_string())
        })?;

        let token1 = pool.get("token1").ok_or_else(|| {
            crate::error::DAppError::Internal("Missing token1 data".to_string())
        })?;

        let token_in = TokenInfo {
            address: token0.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            symbol: token0.get("symbol").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            name: token0.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            decimals: token0.get("decimals").and_then(|v| v.as_u64()).unwrap_or(18) as u8,
            logo_uri: None,
            price_usd: None,
            market_cap: None,
        };

        let token_out = TokenInfo {
            address: token1.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            symbol: token1.get("symbol").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            name: token1.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            decimals: token1.get("decimals").and_then(|v| v.as_u64()).unwrap_or(18) as u8,
            logo_uri: None,
            price_usd: None,
            market_cap: None,
        };

        let amount_in = swap_data.get("amount0").and_then(|v| v.as_str()).unwrap_or("0").to_string();
        let amount_out = swap_data.get("amount1").and_then(|v| v.as_str()).unwrap_or("0").to_string();
        let user_address = swap_data.get("sender").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let pool_address = pool.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let mut swap_event = SwapEvent::new(
            UniswapVersion::V3,
            swap_data.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            pool_address,
            token_in,
            token_out,
            amount_in,
            amount_out,
            user_address,
        );

        // Add pool information
        if let Some(pool_info) = Self::extract_pool_info(pool) {
            swap_event.add_pool_info(pool_info);
        }

        Ok(swap_event)
    }

    /// Extract pool information from subgraph data
    fn extract_pool_info(pool_data: &serde_json::Value) -> Option<PoolInfo> {
        let token0 = pool_data.get("token0")?;
        let token1 = pool_data.get("token1")?;

        Some(PoolInfo {
            address: pool_data.get("id")?.as_str()?.to_string(),
            token0: token0.get("id")?.as_str()?.to_string(),
            token1: token1.get("id")?.as_str()?.to_string(),
            fee_tier: pool_data.get("fee_tier").and_then(|v| v.as_u64()).map(|v| v as u32),
            liquidity: pool_data.get("liquidity").and_then(|v| v.as_str()).map(|v| v.to_string()),
            volume_24h: pool_data.get("volume_usd").and_then(|v| v.as_str()).map(|v| v.to_string()),
            fees_24h: pool_data.get("fees_usd").and_then(|v| v.as_str()).map(|v| v.to_string()),
            apy: None, // Would need to calculate from historical data
            created_at: None, // Would need to parse timestamp
        })
    }

    /// Get collector status
    pub fn status(&self) -> CollectorStatus {
        CollectorStatus {
            is_running: self.is_running,
            last_v2_block: self.last_v2_block,
            last_v3_block: self.last_v3_block,
        }
    }

    /// Perform health check
    pub async fn health_check(&self) -> Result<bool> {
        // Test subgraph connectivity
        let subgraph_healthy = self.subgraph_client.test_connectivity().await.is_ok();
        
        // Test Redis connectivity
        let redis_healthy = self.redis_publisher.test_connection().await.is_ok();

        Ok(subgraph_healthy && redis_healthy)
    }

    /// Graceful shutdown
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down swap event collector...");

        if self.is_running {
            self.stop_collecting().await?;
        }

        info!("Swap event collector shutdown complete");
        Ok(())
    }
}

/// Status information for the swap event collector
#[derive(Debug, Clone)]
pub struct CollectorStatus {
    pub is_running: bool,
    pub last_v2_block: u64,
    pub last_v3_block: u64,
}

impl std::fmt::Display for CollectorStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Collector Status: running={}, v2_block={}, v3_block={}",
            self.is_running, self.last_v2_block, self.last_v3_block
        )
    }
} 