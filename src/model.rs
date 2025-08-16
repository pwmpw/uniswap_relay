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
    #[allow(clippy::too_many_arguments)]
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

    /// Create a new SwapEvent with a builder pattern to avoid too many arguments
    pub fn builder() -> SwapEventBuilder {
        SwapEventBuilder::default()
    }

    /// Create a SwapEvent using the builder pattern with validation
    #[allow(clippy::too_many_arguments)]
    pub fn create_with_builder(
        version: UniswapVersion,
        transaction_hash: String,
        pool_address: String,
        token_in: TokenInfo,
        token_out: TokenInfo,
        amount_in: String,
        amount_out: String,
        user_address: String,
    ) -> Result<Self, String> {
        let builder = Self::builder()
            .version(version)
            .transaction_hash(transaction_hash)
            .pool_address(pool_address)
            .token_in(token_in)
            .token_out(token_out)
            .amount_in(amount_in)
            .amount_out(amount_out)
            .user_address(user_address);

        // Validate before building
        let warnings = builder.validate();
        if !warnings.is_empty() {
            eprintln!("SwapEvent: Builder validation warnings: {}", warnings.join(", "));
        }

        builder.build()
    }

    pub fn add_pool_info(&mut self, pool_info: PoolInfo) {
        self.pool_info = Some(pool_info);
    }

    #[allow(dead_code)]
    pub fn add_enriched_data(&mut self, enriched_data: EnrichedData) {
        self.enriched_data = Some(enriched_data);
    }

    #[allow(dead_code)]
    pub fn set_block_info(&mut self, block_number: u64, timestamp: DateTime<Utc>) {
        self.block_number = block_number;
        self.timestamp = timestamp;
    }

    #[allow(dead_code)]
    pub fn set_gas_info(&mut self, gas_used: u64, gas_price: String, gas_cost_usd: f64) {
        self.gas_used = Some(gas_used);
        self.gas_price = Some(gas_price);
        self.gas_cost_usd = Some(gas_cost_usd);
    }

    #[allow(dead_code)]
    pub fn set_usd_amounts(&mut self, amount_in_usd: f64, amount_out_usd: f64) {
        self.amount_in_usd = Some(amount_in_usd);
        self.amount_out_usd = Some(amount_out_usd);
    }

    #[allow(dead_code)]
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

/// Builder for SwapEvent to avoid too many constructor arguments
#[derive(Default)]
pub struct SwapEventBuilder {
    version: Option<UniswapVersion>,
    transaction_hash: Option<String>,
    pool_address: Option<String>,
    token_in: Option<TokenInfo>,
    token_out: Option<TokenInfo>,
    amount_in: Option<String>,
    amount_out: Option<String>,
    user_address: Option<String>,
}

impl SwapEventBuilder {
    pub fn version(mut self, version: UniswapVersion) -> Self {
        eprintln!("SwapEventBuilder: Setting version to {:?}", version);
        self.version = Some(version);
        self
    }

    pub fn transaction_hash(mut self, transaction_hash: String) -> Self {
        if transaction_hash.is_empty() {
            eprintln!("SwapEventBuilder: Warning - transaction hash is empty");
        } else if transaction_hash.len() < 10 {
            eprintln!("SwapEventBuilder: Warning - transaction hash seems too short: {}", transaction_hash);
        }
        self.transaction_hash = Some(transaction_hash);
        self
    }

    pub fn pool_address(mut self, pool_address: String) -> Self {
        if pool_address.is_empty() {
            eprintln!("SwapEventBuilder: Warning - pool address is empty");
        } else if !pool_address.starts_with("0x") {
            eprintln!("SwapEventBuilder: Warning - pool address doesn't start with 0x: {}", pool_address);
        }
        self.pool_address = Some(pool_address);
        self
    }

    pub fn token_in(mut self, token_in: TokenInfo) -> Self {
        if token_in.address.is_empty() {
            eprintln!("SwapEventBuilder: Warning - token in address is empty");
        } else if !token_in.address.starts_with("0x") {
            eprintln!("SwapEventBuilder: Warning - token in address doesn't start with 0x: {}", token_in.address);
        }
        if token_in.symbol.is_empty() {
            eprintln!("SwapEventBuilder: Warning - token in symbol is empty");
        }
        self.token_in = Some(token_in);
        self
    }

    pub fn token_out(mut self, token_out: TokenInfo) -> Self {
        if token_out.address.is_empty() {
            eprintln!("SwapEventBuilder: Warning - token out address is empty");
        } else if !token_out.address.starts_with("0x") {
            eprintln!("SwapEventBuilder: Warning - token out address doesn't start with 0x: {}", token_out.address);
        }
        if token_out.symbol.is_empty() {
            eprintln!("SwapEventBuilder: Warning - token out symbol is empty");
        }
        self.token_out = Some(token_out);
        self
    }

    pub fn amount_in(mut self, amount_in: String) -> Self {
        if amount_in.is_empty() {
            eprintln!("SwapEventBuilder: Warning - amount in is empty");
        } else if !amount_in.chars().all(|c| c.is_ascii_digit() || c == '.') {
            eprintln!("SwapEventBuilder: Warning - amount in is not numeric: {}", amount_in);
        }
        self.amount_in = Some(amount_in);
        self
    }

    pub fn amount_out(mut self, amount_out: String) -> Self {
        if amount_out.is_empty() {
            eprintln!("SwapEventBuilder: Warning - amount out is empty");
        } else if !amount_out.chars().all(|c| c.is_ascii_digit() || c == '.') {
            eprintln!("SwapEventBuilder: Warning - amount out is not numeric: {}", amount_out);
        }
        self.amount_out = Some(amount_out);
        self
    }

    pub fn user_address(mut self, user_address: String) -> Self {
        if user_address.is_empty() {
            eprintln!("SwapEventBuilder: Warning - user address is empty");
        } else if !user_address.starts_with("0x") {
            eprintln!("SwapEventBuilder: Warning - user address doesn't start with 0x: {}", user_address);
        }
        self.user_address = Some(user_address);
        self
    }

    /// Validate the current builder state and return any warnings
    pub fn validate(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        
        // Check for missing fields
        if self.version.is_none() {
            warnings.push("Version is not set".to_string());
        }
        
        if self.transaction_hash.is_none() {
            warnings.push("Transaction hash is not set".to_string());
        } else if let Some(ref hash) = self.transaction_hash {
            if hash.is_empty() {
                warnings.push("Transaction hash is empty".to_string());
            }
        }
        
        if self.pool_address.is_none() {
            warnings.push("Pool address is not set".to_string());
        } else if let Some(ref addr) = self.pool_address {
            if addr.is_empty() {
                warnings.push("Pool address is empty".to_string());
            } else if !addr.starts_with("0x") {
                warnings.push("Pool address doesn't start with 0x".to_string());
            }
        }
        
        if self.token_in.is_none() {
            warnings.push("Token in is not set".to_string());
        } else if let Some(ref token) = self.token_in {
            if token.address.is_empty() {
                warnings.push("Token in address is empty".to_string());
            } else if !token.address.starts_with("0x") {
                warnings.push("Token in address doesn't start with 0x".to_string());
            }
            if token.symbol.is_empty() {
                warnings.push("Token in symbol is empty".to_string());
            }
        }
        
        if self.token_out.is_none() {
            warnings.push("Token out is not set".to_string());
        } else if let Some(ref token) = self.token_out {
            if token.address.is_empty() {
                warnings.push("Token out address is empty".to_string());
            } else if !token.address.starts_with("0x") {
                warnings.push("Token out address doesn't start with 0x".to_string());
            }
            if token.symbol.is_empty() {
                warnings.push("Token out symbol is empty".to_string());
            }
        }
        
        if self.amount_in.is_none() {
            warnings.push("Amount in is not set".to_string());
        } else if let Some(ref amount) = self.amount_in {
            if amount.is_empty() {
                warnings.push("Amount in is empty".to_string());
            } else if !amount.chars().all(|c| c.is_ascii_digit() || c == '.') {
                warnings.push("Amount in is not numeric".to_string());
            }
        }
        
        if self.amount_out.is_none() {
            warnings.push("Amount out is not set".to_string());
        } else if let Some(ref amount) = self.amount_out {
            if amount.is_empty() {
                warnings.push("Amount out is empty".to_string());
            } else if !amount.chars().all(|c| c.is_ascii_digit() || c == '.') {
                warnings.push("Amount out is not numeric".to_string());
            }
        }
        
        if self.user_address.is_none() {
            warnings.push("User address is not set".to_string());
        } else if let Some(ref addr) = self.user_address {
            if addr.is_empty() {
                warnings.push("User address is empty".to_string());
            } else if !addr.starts_with("0x") {
                warnings.push("User address doesn't start with 0x".to_string());
            }
        }
        
        warnings
    }

    /// Check if the builder is ready to build
    pub fn is_ready(&self) -> bool {
        self.validate().is_empty()
    }

    /// Get a summary of the current builder state
    pub fn get_summary(&self) -> String {
        let warnings = self.validate();
        if warnings.is_empty() {
            "SwapEventBuilder: All fields are set and valid".to_string()
        } else {
            format!("SwapEventBuilder: {} warnings - {}", warnings.len(), warnings.join(", "))
        }
    }

    /// Test the builder with sample data
    #[cfg(test)]
    pub fn test_builder() -> Result<SwapEvent, String> {
        let token_in = TokenInfo {
            address: "0xA0b86a33E6441b8c4C3B1b1ef4F2faD6244b51a".to_string(),
            symbol: "USDC".to_string(),
            decimals: 6,
            name: "USD Coin".to_string(),
            logo_uri: None,
            price_usd: None,
            market_cap: None,
        };

        let token_out = TokenInfo {
            address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
            symbol: "WETH".to_string(),
            decimals: 18,
            name: "Wrapped Ether".to_string(),
            logo_uri: None,
            price_usd: None,
            market_cap: None,
        };

        SwapEvent::builder()
            .version(UniswapVersion::V3)
            .transaction_hash("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string())
            .pool_address("0x8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8".to_string())
            .token_in(token_in)
            .token_out(token_out)
            .amount_in("1000000".to_string()) // 1 USDC
            .amount_out("0.0005".to_string()) // 0.0005 WETH
            .user_address("0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6".to_string())
            .build()
    }

    /// Demonstrate error handling scenarios
    #[cfg(test)]
    pub fn demonstrate_errors() -> Vec<String> {
        let mut errors = Vec::new();
        
        // Test missing version
        let builder = SwapEvent::builder()
            .transaction_hash("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string())
            .pool_address("0x8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8".to_string());
        
        if let Err(e) = builder.build() {
            errors.push(format!("Missing version error: {}", e));
        }
        
        // Test invalid transaction hash
        let builder = SwapEvent::builder()
            .version(UniswapVersion::V2)
            .transaction_hash("".to_string()) // Empty hash
            .pool_address("0x8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8".to_string());
        
        if let Err(e) = builder.build() {
            errors.push(format!("Invalid transaction hash error: {}", e));
        }
        
        // Test invalid pool address
        let builder = SwapEvent::builder()
            .version(UniswapVersion::V2)
            .transaction_hash("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string())
            .pool_address("invalid_address".to_string()); // Invalid address
        
        if let Err(e) = builder.build() {
            errors.push(format!("Invalid pool address error: {}", e));
        }
        
        errors
    }

    pub fn build(self) -> Result<SwapEvent, String> {
        // Validate required fields with detailed error messages
        let version = self.version.ok_or_else(|| {
            eprintln!("SwapEventBuilder: Version field is missing");
            "Version is required for SwapEvent"
        })?;
        
        let transaction_hash = self.transaction_hash.ok_or_else(|| {
            eprintln!("SwapEventBuilder: Transaction hash field is missing");
            "Transaction hash is required for SwapEvent"
        })?;
        
        let pool_address = self.pool_address.ok_or_else(|| {
            eprintln!("SwapEventBuilder: Pool address field is missing");
            "Pool address is required for SwapEvent"
        })?;
        
        let token_in = self.token_in.ok_or_else(|| {
            eprintln!("SwapEventBuilder: Token in field is missing");
            "Token in is required for SwapEvent"
        })?;
        
        let token_out = self.token_out.ok_or_else(|| {
            eprintln!("SwapEventBuilder: Token out field is missing");
            "Token out is required for SwapEvent"
        })?;
        
        let amount_in = self.amount_in.ok_or_else(|| {
            eprintln!("SwapEventBuilder: Amount in field is missing");
            "Amount in is required for SwapEvent"
        })?;
        
        let amount_out = self.amount_out.ok_or_else(|| {
            eprintln!("SwapEventBuilder: Amount out field is missing");
            "Amount out is required for SwapEvent"
        })?;
        
        let user_address = self.user_address.ok_or_else(|| {
            eprintln!("SwapEventBuilder: User address field is missing");
            "User address is required for SwapEvent"
        })?;

        // Validate field values
        if transaction_hash.is_empty() {
            return Err("Transaction hash cannot be empty".to_string());
        }
        
        if pool_address.is_empty() {
            return Err("Pool address cannot be empty".to_string());
        }
        
        if amount_in.is_empty() || amount_out.is_empty() {
            return Err("Amount fields cannot be empty".to_string());
        }
        
        if user_address.is_empty() {
            return Err("User address cannot be empty".to_string());
        }

        // Validate token addresses
        if token_in.address.is_empty() || token_out.address.is_empty() {
            return Err("Token addresses cannot be empty".to_string());
        }

        // Validate amounts are numeric
        if !amount_in.chars().all(|c| c.is_ascii_digit() || c == '.') {
            return Err("Amount in must be a valid numeric value".to_string());
        }
        
        if !amount_out.chars().all(|c| c.is_ascii_digit() || c == '.') {
            return Err("Amount out must be a valid numeric value".to_string());
        }

        eprintln!("SwapEventBuilder: Successfully built SwapEvent for transaction {}", transaction_hash);
        
        Ok(SwapEvent {
            id: format!("{}_{}", version, transaction_hash),
            version,
            timestamp: Utc::now(),
            block_number: 0,
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
        })
    }
}
