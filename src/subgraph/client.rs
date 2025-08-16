use crate::config::AppConfig;
use crate::error::{DAppError, Result, SubgraphError};
use crate::model::PoolQueryResult;
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;
use tracing::{debug, info, warn};

/// GraphQL client for Uniswap subgraphs
pub struct SubgraphClient {
    client: Client,
    config: AppConfig,
}

impl SubgraphClient {
    /// Create a new subgraph client
    pub fn new(config: AppConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.subgraph.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    /// Query Uniswap V2 subgraph
    pub async fn query_uniswap_v2(
        &self,
        query: &str,
        variables: Option<Value>,
    ) -> Result<PoolQueryResult> {
        self.query_subgraph(&self.config.subgraph.uniswap_v2_url, query, variables)
            .await
    }

    /// Query Uniswap V3 subgraph
    pub async fn query_uniswap_v3(
        &self,
        query: &str,
        variables: Option<Value>,
    ) -> Result<PoolQueryResult> {
        self.query_subgraph(&self.config.subgraph.uniswap_v3_url, query, variables)
            .await
    }

    /// Generic subgraph query method
    async fn query_subgraph(
        &self,
        url: &str,
        query: &str,
        variables: Option<Value>,
    ) -> Result<PoolQueryResult> {
        let request_body = json!({
            "query": query,
            "variables": variables
        });

        debug!("Querying subgraph {}: {}", url, request_body);

        let response = self
            .client
            .post(url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| DAppError::Subgraph(SubgraphError::Http(e.to_string())))?;

        if !response.status().is_success() {
            return Err(DAppError::Subgraph(SubgraphError::Http(format!(
                "HTTP error: {}",
                response.status()
            ))));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| DAppError::Subgraph(SubgraphError::Http(e.to_string())))?;

        let result: PoolQueryResult = serde_json::from_str(&response_text)
            .map_err(|e| DAppError::Subgraph(SubgraphError::Parsing(e.to_string())))?;

        // Check for GraphQL errors
        if let Some(errors) = &result.errors {
            if !errors.is_empty() {
                let error_messages: Vec<String> =
                    errors.iter().map(|e| e.message.clone()).collect();

                return Err(DAppError::Subgraph(SubgraphError::GraphQL(
                    error_messages.join("; "),
                )));
            }
        }

        Ok(result)
    }

    /// Get pool information by address from V2
    #[allow(dead_code)]
    pub async fn get_v2_pool_info(&self, pool_address: &str) -> Result<Option<Value>> {
        let query = r#"
            query GetPair($pairId: ID!) {
                pair(id: $pairId) {
                    id
                    token0 {
                        id
                        symbol
                        name
                        decimals
                        totalSupply
                        volume
                        volumeUSD
                    }
                    token1 {
                        id
                        symbol
                        name
                        decimals
                        totalSupply
                        volume
                        volumeUSD
                    }
                    reserve0
                    reserve1
                    totalSupply
                    reserveUSD
                    trackedReserveETH
                    token0Price
                    token1Price
                    volumeUSD
                    untrackedVolumeUSD
                    txCount
                    createdAtTimestamp
                    createdAtBlockNumber
                }
            }
        "#;

        let variables = json!({
            "pairId": pool_address
        });

        let result = self.query_uniswap_v2(query, Some(variables)).await?;
        Ok(result.data.and_then(|data| data.get("pair").cloned()))
    }

    /// Get pool information by address from V3
    #[allow(dead_code)]
    pub async fn get_v3_pool_info(&self, pool_address: &str) -> Result<Option<Value>> {
        let query = r#"
            query GetPool($poolId: ID!) {
                pool(id: $poolId) {
                    id
                    token0 {
                        id
                        symbol
                        name
                        decimals
                        totalSupply
                        volume
                        volumeUSD
                    }
                    token1 {
                        id
                        symbol
                        name
                        decimals
                        totalSupply
                        volume
                        volumeUSD
                    }
                    feeTier
                    liquidity
                    sqrtPrice
                    token0Price
                    token1Price
                    volumeUSD
                    feesUSD
                    totalValueLockedUSD
                }
            }
        "#;

        let variables = json!({
            "poolId": pool_address
        });

        let result = self.query_uniswap_v3(query, Some(variables)).await?;
        Ok(result.data.and_then(|data| data.get("pool").cloned()))
    }

    /// Get token information from V2
    #[allow(dead_code)]
    pub async fn get_v2_token_info(&self, token_address: &str) -> Result<Option<Value>> {
        let query = r#"
            query GetToken($tokenId: ID!) {
                token(id: $tokenId) {
                    id
                    symbol
                    name
                    decimals
                    totalSupply
                    volume
                    volumeUSD
                    totalValueLocked
                    totalValueLockedUSD
                }
            }
        "#;

        let variables = json!({
            "tokenId": token_address
        });

        let result = self.query_uniswap_v2(query, Some(variables)).await?;
        Ok(result.data.and_then(|data| data.get("token").cloned()))
    }

    /// Get token information from V3
    #[allow(dead_code)]
    pub async fn get_v3_token_info(&self, token_address: &str) -> Result<Option<Value>> {
        let query = r#"
            query GetToken($tokenId: ID!) {
                token(id: $tokenId) {
                    id
                    symbol
                    name
                    decimals
                    totalSupply
                    volume
                    volumeUSD
                    totalValueLocked
                    totalValueLockedUSD
                }
            }
        "#;

        let variables = json!({
            "tokenId": token_address
        });

        let result = self.query_uniswap_v3(query, Some(variables)).await?;
        Ok(result.data.and_then(|data| data.get("token").cloned()))
    }

    /// Get recent swaps for a V2 pool
    #[allow(dead_code)]
    pub async fn get_v2_recent_swaps(&self, pool_address: &str, limit: u32) -> Result<Vec<Value>> {
        let query = r#"
            query GetRecentSwaps($pairId: ID!, $limit: Int!) {
                swaps(
                    where: { pair: $pairId }
                    orderBy: timestamp
                    orderDirection: desc
                    first: $limit
                ) {
                    id
                    timestamp
                    pair {
                        id
                        token0 { symbol }
                        token1 { symbol }
                    }
                    sender
                    amount0In
                    amount1In
                    amount0Out
                    amount1Out
                    to
                    logIndex
                    amountUSD
                }
            }
        "#;

        let variables = json!({
            "pairId": pool_address,
            "limit": limit
        });

        let result = self.query_uniswap_v2(query, Some(variables)).await?;

        // Extract swaps from response
        if let Some(data) = result.data {
            if let Some(swaps) = data.get("swaps") {
                if let Some(swaps_array) = swaps.as_array() {
                    return Ok(swaps_array.clone());
                }
            }
        }

        Ok(Vec::new())
    }

    /// Get recent swaps for a V3 pool
    #[allow(dead_code)]
    pub async fn get_v3_recent_swaps(&self, pool_address: &str, limit: u32) -> Result<Vec<Value>> {
        let query = r#"
            query GetRecentSwaps($poolId: ID!, $limit: Int!) {
                swaps(
                    where: { pool: $poolId }
                    orderBy: timestamp
                    orderDirection: desc
                    first: $limit
                ) {
                    id
                    timestamp
                    pool {
                        id
                        token0 { symbol }
                        token1 { symbol }
                    }
                    token0
                    token1
                    sender
                    recipient
                    origin
                    amount0
                    amount1
                    amountUSD
                    sqrtPriceX96
                    liquidity
                    tick
                }
            }
        "#;

        let variables = json!({
            "poolId": pool_address,
            "limit": limit
        });

        let result = self.query_uniswap_v3(query, Some(variables)).await?;

        // Extract swaps from response
        if let Some(data) = result.data {
            if let Some(swaps) = data.get("swaps") {
                if let Some(swaps_array) = swaps.as_array() {
                    return Ok(swaps_array.clone());
                }
            }
        }

        Ok(Vec::new())
    }

    /// Test subgraph connectivity
    pub async fn test_connectivity(&self) -> Result<()> {
        let test_query = r#"
            query {
                _meta {
                    block {
                        number
                    }
                }
            }
        "#;

        // Test V2 subgraph
        match self.query_uniswap_v2(test_query, None).await {
            Ok(_) => info!("Uniswap V2 subgraph connectivity: OK"),
            Err(e) => warn!("Uniswap V2 subgraph connectivity: FAILED - {}", e),
        }

        // Test V3 subgraph
        match self.query_uniswap_v3(test_query, None).await {
            Ok(_) => info!("Uniswap V3 subgraph connectivity: OK"),
            Err(e) => warn!("Uniswap V3 subgraph connectivity: FAILED - {}", e),
        }

        Ok(())
    }
}

impl Clone for SubgraphClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            config: self.config.clone(),
        }
    }
}
