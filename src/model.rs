use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a normalized Uniswap swap event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapEvent {
    pub id: String,
    pub version: UniswapVersion,
    pub timestamp: DateTime<Utc>,
    pub block_number: u64,
    pub transaction_hash: String,
    pub pool_address: String,
    pub token_in: TokenInfo,
    pub token_out: TokenInfo,
    pub amount_in: String,
    pub amount_out: String,
    pub amount_in_usd: Option<f64>,
    pub amount_out_usd: Option<f64>,
    pub fee_amount: Option<String>,
    pub fee_usd: Option<f64>,
    pub user_address: String,
    pub gas_used: Option<u64>,
    pub gas_price: Option<String>,
    pub gas_cost_usd: Option<f64>,
    pub pool_info: Option<PoolInfo>,
    pub enriched_data: Option<EnrichedData>,
}

/// Uniswap version identifier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UniswapVersion {
    V2,
    V3,
}

/// Token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub logo_uri: Option<String>,
    pub price_usd: Option<f64>,
    pub market_cap: Option<f64>,
}

/// Pool information from subgraphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolInfo {
    pub address: String,
    pub token0: String,
    pub token1: String,
    pub fee_tier: Option<u32>,
    pub liquidity: Option<String>,
    pub volume_24h: Option<String>,
    pub fees_24h: Option<String>,
    pub apy: Option<f64>,
    pub created_at: Option<DateTime<Utc>>,
}

/// Enriched data from additional sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedData {
    pub token_metadata: HashMap<String, TokenMetadata>,
    pub market_data: Option<MarketData>,
    pub risk_metrics: Option<RiskMetrics>,
}

/// Token metadata from subgraphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetadata {
    pub total_supply: Option<String>,
    pub circulating_supply: Option<String>,
    pub holders_count: Option<u64>,
    pub transfers_count_24h: Option<u64>,
    pub volume_24h: Option<String>,
}

/// Market data information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub price_change_24h: Option<f64>,
    pub price_change_7d: Option<f64>,
    pub volume_change_24h: Option<f64>,
    pub market_cap_rank: Option<u32>,
    pub fully_diluted_valuation: Option<f64>,
}

/// Risk metrics for the swap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub impermanent_loss_risk: Option<f64>,
    pub volatility_score: Option<f64>,
    pub liquidity_score: Option<f64>,
    pub smart_contract_risk: Option<f64>,
}

/// GraphQL query result for pool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolQueryResult {
    pub data: Option<serde_json::Value>,
    pub errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLError {
    pub message: String,
    pub locations: Option<Vec<GraphQLLocation>>,
    pub path: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLLocation {
    pub line: u32,
    pub column: u32,
}

/// Swap event from Uniswap V2 subgraph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniswapV2SwapEvent {
    pub id: String,
    pub timestamp: String,
    pub pair: GraphQLPair,
    pub sender: String,
    pub amount0_in: String,
    pub amount1_in: String,
    pub amount0_out: String,
    pub amount1_out: String,
    pub to: String,
    pub log_index: u32,
    pub amount_usd: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLPair {
    pub id: String,
    pub token0: GraphQLToken,
    pub token1: GraphQLToken,
    pub reserve0: String,
    pub reserve1: String,
    pub total_supply: String,
    pub reserve_usd: Option<String>,
    pub tracked_reserve_eth: Option<String>,
    pub token0_price: Option<String>,
    pub token1_price: Option<String>,
    pub volume_usd: Option<String>,
    pub untracked_volume_usd: Option<String>,
    pub tx_count: Option<String>,
    pub created_at_timestamp: Option<String>,
    pub created_at_block_number: Option<String>,
}

/// Swap event from Uniswap V3 subgraph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniswapV3SwapEvent {
    pub id: String,
    pub timestamp: String,
    pub pool: GraphQLV3Pool,
    pub token0: String,
    pub token1: String,
    pub sender: String,
    pub recipient: String,
    pub origin: String,
    pub amount0: String,
    pub amount1: String,
    pub amount_usd: Option<String>,
    pub sqrt_price_x96: String,
    pub liquidity: String,
    pub tick: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLV3Pool {
    pub id: String,
    pub token0: GraphQLToken,
    pub token1: GraphQLToken,
    pub fee_tier: u32,
    pub liquidity: String,
    pub sqrt_price: Option<String>,
    pub token0_price: Option<String>,
    pub token1_price: Option<String>,
    pub volume_usd: Option<String>,
    pub fees_usd: Option<String>,
    pub total_value_locked_usd: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLToken {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub total_supply: Option<String>,
    pub volume: Option<String>,
    pub volume_usd: Option<String>,
}

/// Health check status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub uptime_seconds: u64,
    pub checks: HashMap<String, CheckStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckStatus {
    pub status: String,
    pub message: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub response_time_ms: Option<u64>,
}

/// Metrics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub events_processed_total: u64,
    pub events_processed_rate: f64,
    pub errors_total: u64,
    pub errors_rate: f64,
    pub latency_p50_ms: f64,
    pub latency_p95_ms: f64,
    pub latency_p99_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub timestamp: DateTime<Utc>,
}

impl SwapEvent {
    pub fn new(
        version: UniswapVersion,
        transaction_hash: String,
        pool_address: String,
        token_in: TokenInfo,
        token_out: TokenInfo,
        amount_in: String,
        amount_out: String,
        user_address: String,
    ) -> Self {
        Self {
            id: format!("{}_{}", version, transaction_hash),
            version,
            timestamp: Utc::now(),
            block_number: 0, // Will be set by the collector
            transaction_hash,
            pool_address,
            token_in,
            token_out,
            amount_in,
            amount_out,
            amount_in_usd: None,
            amount_out_usd: None,
            fee_amount: None,
            fee_usd: None,
            user_address,
            gas_used: None,
            gas_price: None,
            gas_cost_usd: None,
            pool_info: None,
            enriched_data: None,
        }
    }

    pub fn add_pool_info(&mut self, pool_info: PoolInfo) {
        self.pool_info = Some(pool_info);
    }

    pub fn add_enriched_data(&mut self, enriched_data: EnrichedData) {
        self.enriched_data = Some(enriched_data);
    }

    pub fn set_block_info(&mut self, block_number: u64, timestamp: DateTime<Utc>) {
        self.block_number = block_number;
        self.timestamp = timestamp;
    }

    pub fn set_gas_info(&mut self, gas_used: u64, gas_price: String, gas_cost_usd: f64) {
        self.gas_used = Some(gas_used);
        self.gas_price = Some(gas_price);
        self.gas_cost_usd = Some(gas_cost_usd);
    }

    pub fn set_usd_amounts(&mut self, amount_in_usd: f64, amount_out_usd: f64) {
        self.amount_in_usd = Some(amount_in_usd);
        self.amount_out_usd = Some(amount_out_usd);
    }

    pub fn set_fee_info(&mut self, fee_amount: String, fee_usd: f64) {
        self.fee_amount = Some(fee_amount);
        self.fee_usd = Some(fee_usd);
    }
}

impl std::fmt::Display for UniswapVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UniswapVersion::V2 => write!(f, "v2"),
            UniswapVersion::V3 => write!(f, "v3"),
        }
    }
}
