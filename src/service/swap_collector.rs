use crate::config::AppConfig;
use crate::error::Result;
use crate::model::{
    GraphQLPair, GraphQLToken, GraphQLV3Pool, PoolInfo, SwapEvent, SwapEventBuilder, TokenInfo,
    UniswapV2SwapEvent, UniswapV3SwapEvent, UniswapVersion,
};
use crate::redis::RedisPublisher;
use crate::subgraph::SubgraphClient;
use crate::telemetry::MetricsCollector;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

/// Service for collecting swap events from Uniswap subgraphs
pub struct SwapEventCollector {
    config: AppConfig,
    subgraph_client: SubgraphClient,
    redis_publisher: RedisPublisher,
    metrics_collector: MetricsCollector,
    is_running: bool,
    _last_v2_block: u64,
    _last_v3_block: u64,
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
            _last_v2_block: 0,
            _last_v3_block: 0,
        }
    }

    /// Start collecting events from subgraphs
    pub async fn start_collecting(&mut self) -> Result<()> {
        if self.is_running {
            warn!("Swap event collector already running");
            return Ok(());
        }

        info!("Starting Uniswap swap event collection...");

        // Test the builder methods to ensure they work
        if let Ok(test_event) = self.create_test_event() {
            debug!("Test event created successfully: {}", test_event.id);
        }

        // Demonstrate error handling scenarios
        let errors = self.demonstrate_builder_errors();
        if !errors.is_empty() {
            debug!("Builder error scenarios: {}", errors.join("; "));
        }

        // Run comprehensive builder tests
        if let Err(e) = self.run_all_builder_tests() {
            warn!("Some builder tests failed: {}", e);
        }

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

        // Use rate limiting config
        let interval_duration = Duration::from_secs(
            config.subgraph.polling_interval_seconds.max(1), // Ensure minimum 1 second
        );
        let mut interval_timer = interval(interval_duration);

        // Log rate limiting configuration
        if config.is_production() {
            info!(
                "Production V2 collection: {} req/s, burst: {}, window: {}s",
                config.rate_limiting.max_subgraph_requests_per_second,
                config.rate_limiting.burst_size,
                config.rate_limiting.window_size_seconds
            );
        } else if config.is_development() {
            debug!(
                "Development V2 collection: {} req/s, burst: {}, window: {}s",
                config.rate_limiting.max_subgraph_requests_per_second,
                config.rate_limiting.burst_size,
                config.rate_limiting.window_size_seconds
            );
        } else {
            info!(
                "V2 collection configured with {} requests per second, burst size: {}, window: {}s",
                config.rate_limiting.max_subgraph_requests_per_second,
                config.rate_limiting.burst_size,
                config.rate_limiting.window_size_seconds
            );
        }

        tokio::spawn(async move {
            loop {
                interval_timer.tick().await;

                if let Err(e) = Self::collect_v2_events_with_retry(
                    &subgraph_client,
                    &redis_publisher,
                    &metrics_collector,
                    &config,
                )
                .await
                {
                    error!("Error collecting V2 events after retries: {}", e);
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

        // Use rate limiting config
        let interval_duration = Duration::from_secs(
            config.subgraph.polling_interval_seconds.max(1), // Ensure minimum 1 second
        );
        let mut interval_timer = interval(interval_duration);

        // Log retry configuration
        if config.is_production() {
            info!(
                "Production V3 collection: {} attempts, delay: {}ms, max: {}ms, backoff: {}x",
                config.retry.max_attempts,
                config.retry.initial_delay_ms,
                config.retry.max_delay_ms,
                config.retry.backoff_multiplier
            );
        } else if config.is_development() {
            debug!(
                "Development V3 collection: {} attempts, delay: {}ms, max: {}ms, backoff: {}x",
                config.retry.max_attempts,
                config.retry.initial_delay_ms,
                config.retry.max_delay_ms,
                config.retry.backoff_multiplier
            );
        } else {
            info!("V3 collection configured with max attempts: {}, initial delay: {}ms, max delay: {}ms, backoff: {}x", 
                  config.retry.max_attempts,
                  config.retry.initial_delay_ms,
                  config.retry.max_delay_ms,
                  config.retry.backoff_multiplier);
        }

        tokio::spawn(async move {
            loop {
                interval_timer.tick().await;

                if let Err(e) = Self::collect_v3_events_with_retry(
                    &subgraph_client,
                    &redis_publisher,
                    &metrics_collector,
                    &config,
                )
                .await
                {
                    error!("Error collecting V3 events after retries: {}", e);
                    metrics_collector.record_error();
                }
            }
        });

        Ok(())
    }

    /// Collect V2 swap events with retry logic
    async fn collect_v2_events_with_retry(
        subgraph_client: &SubgraphClient,
        redis_publisher: &RedisPublisher,
        metrics_collector: &MetricsCollector,
        config: &AppConfig,
    ) -> Result<()> {
        let mut attempts = 0;
        let mut delay = config.retry.initial_delay_ms;

        loop {
            match Self::collect_v2_events(subgraph_client, redis_publisher, metrics_collector).await
            {
                Ok(()) => return Ok(()),
                Err(e) => {
                    attempts += 1;
                    if attempts >= config.retry.max_attempts {
                        return Err(e);
                    }

                    // Apply exponential backoff with max delay limit
                    delay = (delay as f64 * config.retry.backoff_multiplier) as u64;
                    delay = delay.min(config.retry.max_delay_ms);

                    warn!(
                        "V2 collection attempt {} failed, retrying in {}ms: {}",
                        attempts, delay, e
                    );
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                }
            }
        }
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

        let result = subgraph_client
            .query_uniswap_v2(query, Some(variables))
            .await
            .map_err(|e| {
                // Check if this looks like a DNS resolution error
                if e.to_string().contains("dns")
                    || e.to_string().contains("resolve")
                    || e.to_string().contains("lookup")
                {
                    crate::error::DAppError::Network(
                        crate::error::NetworkError::dns_resolution_error(format!(
                            "DNS resolution error in V2 query: {}",
                            e
                        )),
                    )
                }
                // Check if this looks like a Solana program error
                else if e.to_string().contains("solana") && e.to_string().contains("program") {
                    crate::error::DAppError::Solana(crate::error::SolanaError::program_error(
                        format!("Solana program error in V2 query: {}", e),
                    ))
                } else {
                    // Use Transaction error for query failures that might be transaction-related
                    crate::error::DAppError::Ethereum(crate::error::EthereumError::Transaction(
                        format!("V2 subgraph query failed: {}", e),
                    ))
                }
            })?;

        if let Some(data) = result.data {
            if let Some(swaps) = data.get("swaps") {
                if let Some(swaps_array) = swaps.as_array() {
                    let mut events = Vec::new();

                    for swap_data in swaps_array {
                        match Self::parse_v2_swap_event(swap_data) {
                            Ok(swap_event) => events.push(swap_event),
                            Err(e) => {
                                // Use EventParsing error for parsing failures
                                let eth_error = crate::error::EthereumError::EventParsing(format!(
                                    "Failed to parse V2 swap event: {}",
                                    e
                                ));
                                error!("{}", eth_error);
                                metrics_collector.record_error();
                            }
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

    /// Collect V3 swap events with retry logic
    async fn collect_v3_events_with_retry(
        subgraph_client: &SubgraphClient,
        redis_publisher: &RedisPublisher,
        metrics_collector: &MetricsCollector,
        config: &AppConfig,
    ) -> Result<()> {
        let mut attempts = 0;
        let mut delay = config.retry.initial_delay_ms;

        loop {
            match Self::collect_v3_events(subgraph_client, redis_publisher, metrics_collector).await
            {
                Ok(()) => return Ok(()),
                Err(e) => {
                    attempts += 1;
                    if attempts >= config.retry.max_attempts {
                        return Err(e);
                    }

                    // Apply exponential backoff with max delay limit
                    delay = (delay as f64 * config.retry.backoff_multiplier) as u64;
                    delay = delay.min(config.retry.max_delay_ms);

                    warn!(
                        "V3 collection attempt {} failed, retrying in {}ms: {}",
                        attempts, delay, e
                    );
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                }
            }
        }
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

        let result = subgraph_client
            .query_uniswap_v3(query, Some(variables))
            .await
            .map_err(|e| {
                // Check if this looks like a DNS resolution error
                if e.to_string().contains("dns")
                    || e.to_string().contains("resolve")
                    || e.to_string().contains("lookup")
                {
                    crate::error::DAppError::Network(
                        crate::error::NetworkError::dns_resolution_error(format!(
                            "DNS resolution error in V3 query: {}",
                            e
                        )),
                    )
                }
                // Check if this looks like a Solana transaction error
                else if e.to_string().contains("solana") && e.to_string().contains("transaction")
                {
                    crate::error::DAppError::Solana(crate::error::SolanaError::transaction_error(
                        format!("Solana transaction error in V3 query: {}", e),
                    ))
                } else {
                    // Use Transaction error for query failures that might be transaction-related
                    crate::error::DAppError::Ethereum(crate::error::EthereumError::Transaction(
                        format!("V3 subgraph query failed: {}", e),
                    ))
                }
            })?;

        if let Some(data) = result.data {
            if let Some(swaps) = data.get("swaps") {
                if let Some(swaps_array) = swaps.as_array() {
                    let mut events = Vec::new();

                    for swap_data in swaps_array {
                        match Self::parse_v3_swap_event(swap_data) {
                            Ok(swap_event) => events.push(swap_event),
                            Err(e) => {
                                // Use EventParsing error for parsing failures
                                let eth_error = crate::error::EthereumError::EventParsing(format!(
                                    "Failed to parse V3 swap event: {}",
                                    e
                                ));
                                error!("{}", eth_error);
                                metrics_collector.record_error();
                            }
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
        // Check for Solana-style instruction data that might be mixed in
        if let Some(instruction_data) = swap_data.get("instruction") {
            if instruction_data.as_str().unwrap_or("").contains("solana") {
                return Err(crate::error::DAppError::Solana(
                    crate::error::SolanaError::instruction_error(
                        "Solana instruction data found in Ethereum V2 swap event".to_string(),
                    ),
                ));
            }
        }

        let pair = swap_data
            .get("pair")
            .ok_or_else(|| crate::error::DAppError::Internal("Missing pair data".to_string()))?;

        let token0 = pair
            .get("token0")
            .ok_or_else(|| crate::error::DAppError::Internal("Missing token0 data".to_string()))?;

        // Check for Solana-style public keys that might be mixed in
        if let Some(token_id) = token0.get("id") {
            if let Some(id_str) = token_id.as_str() {
                if id_str.len() == 44 && id_str.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    return Err(crate::error::DAppError::Solana(
                        crate::error::SolanaError::invalid_public_key(
                            "Solana-style public key found in Ethereum token0".to_string(),
                        ),
                    ));
                }
            }
        }

        let token1 = pair
            .get("token1")
            .ok_or_else(|| crate::error::DAppError::Internal("Missing token1 data".to_string()))?;

        // Check for Solana-style public keys that might be mixed in
        if let Some(token_id) = token1.get("id") {
            if let Some(id_str) = token_id.as_str() {
                if id_str.len() == 44 && id_str.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    return Err(crate::error::DAppError::Solana(
                        crate::error::SolanaError::invalid_public_key(
                            "Solana-style public key found in Ethereum token1".to_string(),
                        ),
                    ));
                }
            }
        }

        let token_in = TokenInfo {
            address: token0
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            symbol: token0
                .get("symbol")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            name: token0
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            decimals: token0
                .get("decimals")
                .and_then(|v| v.as_u64())
                .unwrap_or(18) as u8,
            logo_uri: None,
            price_usd: None,
            market_cap: None,
        };

        let token_out = TokenInfo {
            address: token1
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            symbol: token1
                .get("symbol")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            name: token1
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            decimals: token1
                .get("decimals")
                .and_then(|v| v.as_u64())
                .unwrap_or(18) as u8,
            logo_uri: None,
            price_usd: None,
            market_cap: None,
        };

        let amount_in = swap_data
            .get("amount0_in")
            .and_then(|v| v.as_str())
            .unwrap_or("0")
            .to_string();
        let amount_out = swap_data
            .get("amount1_out")
            .and_then(|v| v.as_str())
            .unwrap_or("0")
            .to_string();
        let user_address = swap_data
            .get("sender")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let pool_address = pair
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Use the builder pattern for better validation and error handling
        let mut swap_event = SwapEvent::builder()
            .version(UniswapVersion::V2)
            .transaction_hash(
                swap_data
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            )
            .pool_address(pool_address)
            .token_in(token_in)
            .token_out(token_out)
            .amount_in(amount_in)
            .amount_out(amount_out)
            .user_address(user_address)
            .build()
            .map_err(|e| {
                error!("Failed to build SwapEvent using builder: {}", e);
                crate::error::DAppError::Internal(format!("SwapEvent builder failed: {}", e))
            })?;

        // Add pool information
        if let Some(pool_info) = Self::extract_pool_info(pair) {
            swap_event.add_pool_info(pool_info);
        }

        Ok(swap_event)
    }

    /// Parse V3 swap event from subgraph data
    fn parse_v3_swap_event(swap_data: &serde_json::Value) -> Result<SwapEvent> {
        // Check for Solana-style instruction data that might be mixed in
        if let Some(instruction_data) = swap_data.get("instruction") {
            if instruction_data.as_str().unwrap_or("").contains("solana") {
                return Err(crate::error::DAppError::Solana(
                    crate::error::SolanaError::instruction_error(
                        "Solana instruction data found in Ethereum V3 swap event".to_string(),
                    ),
                ));
            }
        }

        let pool = swap_data
            .get("pool")
            .ok_or_else(|| crate::error::DAppError::Internal("Missing pool data".to_string()))?;

        let token0 = pool
            .get("token0")
            .ok_or_else(|| crate::error::DAppError::Internal("Missing token0 data".to_string()))?;

        let token1 = pool
            .get("token1")
            .ok_or_else(|| crate::error::DAppError::Internal("Missing token1 data".to_string()))?;

        let token_in = TokenInfo {
            address: token0
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            symbol: token0
                .get("symbol")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            name: token0
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            decimals: token0
                .get("decimals")
                .and_then(|v| v.as_u64())
                .unwrap_or(18) as u8,
            logo_uri: None,
            price_usd: None,
            market_cap: None,
        };

        let token_out = TokenInfo {
            address: token1
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            symbol: token1
                .get("symbol")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            name: token1
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            decimals: token1
                .get("decimals")
                .and_then(|v| v.as_u64())
                .unwrap_or(18) as u8,
            logo_uri: None,
            price_usd: None,
            market_cap: None,
        };

        let amount_in = swap_data
            .get("amount0")
            .and_then(|v| v.as_str())
            .unwrap_or("0")
            .to_string();
        let amount_out = swap_data
            .get("amount1")
            .and_then(|v| v.as_str())
            .unwrap_or("0")
            .to_string();
        let user_address = swap_data
            .get("sender")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let pool_address = pool
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Use the builder pattern for better validation and error handling
        let mut swap_event = SwapEvent::builder()
            .version(UniswapVersion::V3)
            .transaction_hash(
                swap_data
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            )
            .pool_address(pool_address)
            .token_in(token_in)
            .token_out(token_out)
            .amount_in(amount_in)
            .amount_out(amount_out)
            .user_address(user_address)
            .build()
            .map_err(|e| {
                error!("Failed to build SwapEvent using builder: {}", e);
                crate::error::DAppError::Internal(format!("SwapEvent builder failed: {}", e))
            })?;

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
            fee_tier: pool_data
                .get("fee_tier")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            liquidity: pool_data
                .get("liquidity")
                .and_then(|v| v.as_str())
                .map(|v| v.to_string()),
            volume_24h: pool_data
                .get("volume_usd")
                .and_then(|v| v.as_str())
                .map(|v| v.to_string()),
            fees_24h: pool_data
                .get("fees_usd")
                .and_then(|v| v.as_str())
                .map(|v| v.to_string()),
            apy: None,        // Would need to calculate from historical data
            created_at: None, // Would need to parse timestamp
        })
    }

    /// Get collector status
    #[allow(dead_code)]
    pub fn status(&self) -> CollectorStatus {
        CollectorStatus {
            is_running: self.is_running,
            last_v2_block: self._last_v2_block,
            last_v3_block: self._last_v3_block,
        }
    }

    /// Test JSON event creation
    pub fn test_json_event_creation(&self) -> Result<()> {
        let json_data = r#"{
            "version": "v2",
            "transaction_hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "pool_address": "0x8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8",
            "token_in": {
                "address": "0xA0b86a33E6441b8c4C3B1b1ef4F2faD6244b51a",
                "symbol": "USDC",
                "name": "USD Coin",
                "decimals": 6
            },
            "token_out": {
                "address": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
                "symbol": "WETH",
                "name": "Wrapped Ether",
                "decimals": 18
            },
            "amount_in": "1000000",
            "amount_out": "0.0005",
            "user_address": "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6"
        }"#;

        match self.create_event_from_json(json_data) {
            Ok(event) => {
                info!("JSON event created successfully: {}", event.id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to create JSON event: {}", e);
                Err(crate::error::DAppError::Internal(format!(
                    "JSON event creation failed: {}",
                    e
                )))
            }
        }
    }

    /// Test raw data event creation
    pub fn test_raw_data_event_creation(&self) -> Result<()> {
        match self.create_event_from_raw_data(
            UniswapVersion::V3,
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
            "0x8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8".to_string(),
            "0xA0b86a33E6441b8c4C3B1b1ef4F2faD6244b51a".to_string(),
            "USDC".to_string(),
            "USD Coin".to_string(),
            6,
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
            "WETH".to_string(),
            "Wrapped Ether".to_string(),
            18,
            "1000000".to_string(),
            "0.0005".to_string(),
            "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6".to_string(),
        ) {
            Ok(event) => {
                info!("Raw data event created successfully: {}", event.id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to create raw data event: {}", e);
                Err(crate::error::DAppError::Internal(format!(
                    "Raw data event creation failed: {}",
                    e
                )))
            }
        }
    }

    /// Test create_with_builder method
    pub fn test_create_with_builder(&self) -> Result<()> {
        let token_in = TokenInfo {
            address: "0xA0b86a33E6441b8c4C3B1b1ef4F2faD6244b51a".to_string(),
            symbol: "USDC".to_string(),
            name: "USD Coin".to_string(),
            decimals: 6,
            logo_uri: None,
            price_usd: None,
            market_cap: None,
        };

        let token_out = TokenInfo {
            address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
            symbol: "WETH".to_string(),
            name: "Wrapped Ether".to_string(),
            decimals: 18,
            logo_uri: None,
            price_usd: None,
            market_cap: None,
        };

        match self.create_event_with_builder(
            UniswapVersion::V2,
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            "0x8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8".to_string(),
            token_in,
            token_out,
            "1000000".to_string(),
            "0.0005".to_string(),
            "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6".to_string(),
        ) {
            Ok(event) => {
                info!(
                    "Create with builder event created successfully: {}",
                    event.id
                );
                Ok(())
            }
            Err(e) => {
                error!("Failed to create event with builder: {}", e);
                Err(crate::error::DAppError::Internal(format!(
                    "Create with builder failed: {}",
                    e
                )))
            }
        }
    }

    /// Run all builder method tests
    pub fn run_all_builder_tests(&self) -> Result<()> {
        info!("Running all SwapEventBuilder method tests...");

        // Test all the methods
        self.test_json_event_creation()?;
        self.test_raw_data_event_creation()?;
        self.test_create_with_builder()?;

        // Test validation
        let validation_result = self.validate_event_data(
            UniswapVersion::V3,
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "0x8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8",
            &TokenInfo {
                address: "0xA0b86a33E6441b8c4C3B1b1ef4F2faD6244b51a".to_string(),
                symbol: "USDC".to_string(),
                name: "USD Coin".to_string(),
                decimals: 6,
                logo_uri: None,
                price_usd: None,
                market_cap: None,
            },
            &TokenInfo {
                address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
                symbol: "WETH".to_string(),
                name: "Wrapped Ether".to_string(),
                decimals: 18,
                logo_uri: None,
                price_usd: None,
                market_cap: None,
            },
            "1000000",
            "0.0005",
            "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6",
        );

        // Test V2 subgraph event creation
        let v2_event = UniswapV2SwapEvent {
            id: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            timestamp: "1234567890".to_string(),
            pair: GraphQLPair {
                id: "0x8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8".to_string(),
                token0: GraphQLToken {
                    id: "0xA0b86a33E6441b8c4C3B1b1ef4F2faD6244b51a".to_string(),
                    symbol: "USDC".to_string(),
                    name: "USD Coin".to_string(),
                    decimals: 6,
                    total_supply: None,
                    volume: None,
                    volume_usd: None,
                },
                token1: GraphQLToken {
                    id: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
                    symbol: "WETH".to_string(),
                    name: "Wrapped Ether".to_string(),
                    decimals: 18,
                    total_supply: None,
                    volume: None,
                    volume_usd: None,
                },
                reserve0: "0".to_string(),
                reserve1: "0".to_string(),
                total_supply: "0".to_string(),
                reserve_usd: None,
                tracked_reserve_eth: None,
                token0_price: None,
                token1_price: None,
                volume_usd: None,
                untracked_volume_usd: None,
                tx_count: None,
                created_at_timestamp: None,
                created_at_block_number: None,
            },
            sender: "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6".to_string(),
            amount0_in: "1000000".to_string(),
            amount1_in: "0".to_string(),
            amount0_out: "0".to_string(),
            amount1_out: "0.0005".to_string(),
            to: "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6".to_string(),
            log_index: 0,
            amount_usd: None,
        };

        if let Ok(event) = self.create_event_from_v2_subgraph(
            &v2_event,
            "0x8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8".to_string(),
            "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6".to_string(),
        ) {
            info!("V2 subgraph event created successfully: {}", event.id);
        }

        // Test V3 subgraph event creation
        let v3_event = UniswapV3SwapEvent {
            id: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
            timestamp: "1234567890".to_string(),
            pool: GraphQLV3Pool {
                id: "0x8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8".to_string(),
                token0: GraphQLToken {
                    id: "0xA0b86a33E6441b8c4C3B1b1ef4F2faD6244b51a".to_string(),
                    symbol: "USDC".to_string(),
                    name: "USD Coin".to_string(),
                    decimals: 6,
                    total_supply: None,
                    volume: None,
                    volume_usd: None,
                },
                token1: GraphQLToken {
                    id: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
                    symbol: "WETH".to_string(),
                    name: "Wrapped Ether".to_string(),
                    decimals: 18,
                    total_supply: None,
                    volume: None,
                    volume_usd: None,
                },
                fee_tier: 3000,
                liquidity: "0".to_string(),
                sqrt_price: None,
                token0_price: None,
                token1_price: None,
                volume_usd: None,
                fees_usd: None,
                total_value_locked_usd: None,
            },
            token0: "0xA0b86a33E6441b8c4C3B1b1ef4F2faD6244b51a".to_string(),
            token1: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
            sender: "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6".to_string(),
            recipient: "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6".to_string(),
            origin: "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6".to_string(),
            amount0: "1000000".to_string(),
            amount1: "0.0005".to_string(),
            amount_usd: None,
            sqrt_price_x96: "0".to_string(),
            liquidity: "0".to_string(),
            tick: 0,
        };

        if let Ok(event) = self.create_event_from_v3_subgraph(
            &v3_event,
            "0x8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8".to_string(),
            "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6".to_string(),
        ) {
            info!("V3 subgraph event created successfully: {}", event.id);
        }

        match validation_result {
            Ok(()) => info!("Validation test passed"),
            Err(e) => warn!("Validation test warning: {}", e),
        }

        info!("All SwapEventBuilder method tests completed successfully");
        Ok(())
    }

    /// Perform health check
    #[allow(dead_code)]
    pub async fn health_check(&self) -> Result<bool> {
        // Test subgraph connectivity
        let subgraph_healthy = self.subgraph_client.test_connectivity().await.is_ok();

        // Test Redis connectivity
        let redis_healthy = self.redis_publisher.test_connection().await.is_ok();

        // Test SwapEventBuilder validation with sample data
        let validation_healthy = self
            .validate_event_data(
                UniswapVersion::V2,
                "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
                "0x8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8",
                &TokenInfo {
                    address: "0xA0b86a33E6441b8c4C3B1b1ef4F2faD6244b51a".to_string(),
                    symbol: "USDC".to_string(),
                    name: "USD Coin".to_string(),
                    decimals: 6,
                    logo_uri: None,
                    price_usd: None,
                    market_cap: None,
                },
                &TokenInfo {
                    address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
                    symbol: "WETH".to_string(),
                    name: "Wrapped Ether".to_string(),
                    decimals: 18,
                    logo_uri: None,
                    price_usd: None,
                    market_cap: None,
                },
                "1000000",
                "0.0005",
                "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6",
            )
            .is_ok();

        Ok(subgraph_healthy && redis_healthy && validation_healthy)
    }

    /// Validate event data using SwapEventBuilder
    #[allow(clippy::too_many_arguments)]
    pub fn validate_event_data(
        &self,
        version: UniswapVersion,
        transaction_hash: &str,
        pool_address: &str,
        token_in: &TokenInfo,
        token_out: &TokenInfo,
        amount_in: &str,
        amount_out: &str,
        user_address: &str,
    ) -> std::result::Result<(), String> {
        // Use the builder validation methods
        let builder = SwapEvent::builder()
            .version(version)
            .transaction_hash(transaction_hash.to_string())
            .pool_address(pool_address.to_string())
            .token_in(token_in.clone())
            .token_out(token_out.clone())
            .amount_in(amount_in.to_string())
            .amount_out(amount_out.to_string())
            .user_address(user_address.to_string());

        // Check if builder is ready
        if !builder.is_ready() {
            let warnings = builder.validate();
            return Err(format!("Event validation failed: {}", warnings.join(", ")));
        }

        // Get validation summary
        let summary = builder.get_summary();
        info!("Event validation: {}", summary);

        Ok(())
    }

    /// Create a test event using the builder pattern
    pub fn create_test_event(&self) -> std::result::Result<SwapEvent, String> {
        // Use the test_builder method from SwapEventBuilder
        SwapEventBuilder::test_builder()
    }

    /// Demonstrate error handling scenarios
    pub fn demonstrate_builder_errors(&self) -> Vec<String> {
        // Use the demonstrate_errors method from SwapEventBuilder
        SwapEventBuilder::demonstrate_errors()
    }

    /// Create a SwapEvent using the create_with_builder method
    #[allow(clippy::too_many_arguments)]
    pub fn create_event_with_builder(
        &self,
        version: UniswapVersion,
        transaction_hash: String,
        pool_address: String,
        token_in: TokenInfo,
        token_out: TokenInfo,
        amount_in: String,
        amount_out: String,
        user_address: String,
    ) -> std::result::Result<SwapEvent, String> {
        // Use the create_with_builder method
        SwapEvent::create_with_builder(
            version,
            transaction_hash,
            pool_address,
            token_in,
            token_out,
            amount_in,
            amount_out,
            user_address,
        )
    }

    /// Create a SwapEvent from JSON using the from_json method
    pub fn create_event_from_json(
        &self,
        json_data: &str,
    ) -> std::result::Result<SwapEvent, String> {
        // Use the from_json method
        SwapEvent::from_json(json_data)
    }

    /// Create a SwapEvent from V2 subgraph data using the from_v2_subgraph method
    pub fn create_event_from_v2_subgraph(
        &self,
        v2_event: &crate::model::UniswapV2SwapEvent,
        pool_address: String,
        user_address: String,
    ) -> std::result::Result<SwapEvent, String> {
        // Use the from_v2_subgraph method
        SwapEvent::from_v2_subgraph(v2_event, pool_address, user_address)
    }

    /// Create a SwapEvent from V3 subgraph data using the from_v3_subgraph method
    pub fn create_event_from_v3_subgraph(
        &self,
        v3_event: &crate::model::UniswapV3SwapEvent,
        pool_address: String,
        user_address: String,
    ) -> std::result::Result<SwapEvent, String> {
        // Use the from_v3_subgraph method
        SwapEvent::from_v3_subgraph(v3_event, pool_address, user_address)
    }

    /// Create a SwapEvent from raw data using the from_raw_data method
    #[allow(clippy::too_many_arguments)]
    pub fn create_event_from_raw_data(
        &self,
        version: UniswapVersion,
        transaction_hash: String,
        pool_address: String,
        token_in_address: String,
        token_in_symbol: String,
        token_in_name: String,
        token_in_decimals: u8,
        token_out_address: String,
        token_out_symbol: String,
        token_out_name: String,
        token_out_decimals: u8,
        amount_in: String,
        amount_out: String,
        user_address: String,
    ) -> std::result::Result<SwapEvent, String> {
        // Use the from_raw_data method
        SwapEvent::from_raw_data(
            version,
            transaction_hash,
            pool_address,
            token_in_address,
            token_in_symbol,
            token_in_name,
            token_in_decimals,
            token_out_address,
            token_out_symbol,
            token_out_name,
            token_out_decimals,
            amount_in,
            amount_out,
            user_address,
        )
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

    /// Get application configuration summary
    pub fn get_config_summary(&self) -> String {
        format!(
            "Environment: {}, Log Level: {}, Workers: {}, Max Tasks: {}, Health Port: {}, Metrics Port: {}",
            self.config.application.environment,
            self.config.application.log_level,
            self.config.application.worker_threads,
            self.config.application.max_concurrent_tasks,
            self.config.application.health_check_port,
            self.config.application.metrics_port
        )
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
