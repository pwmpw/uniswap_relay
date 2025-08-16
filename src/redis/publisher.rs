use crate::config::AppConfig;
use crate::error::{RedisError, Result};
use crate::model::SwapEvent;
use redis::{aio::ConnectionManager, AsyncCommands, RedisResult};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// Redis publisher for swap events
#[derive(Clone)]
pub struct RedisPublisher {
    connection_manager: Arc<ConnectionManager>,
    channel: String,
    config: AppConfig,
}

impl RedisPublisher {
    /// Create a new Redis publisher
    pub async fn new(config: AppConfig) -> Result<Self> {
        let client = redis::Client::open(config.redis.url.clone())
            .map_err(|e| RedisError::Connection(e.to_string()))?;

        let connection_manager = ConnectionManager::new(client)
            .await
            .map_err(|e| RedisError::Connection(e.to_string()))?;

        info!("Redis publisher initialized successfully");

        Ok(Self {
            connection_manager: Arc::new(connection_manager),
            channel: config.redis.channel.clone(),
            config,
        })
    }

    /// Publish a single swap event
    #[allow(dead_code)]
    pub async fn publish_event(&self, event: &SwapEvent) -> Result<()> {
        // Check for Solana-style public keys in event addresses
        if event.token_in.address.len() == 44 && event.token_in.address.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(crate::error::DAppError::Solana(crate::error::SolanaError::invalid_public_key(
                "Solana-style public key found in token_in address".to_string()
            )));
        }
        
        if event.token_out.address.len() == 44 && event.token_out.address.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(crate::error::DAppError::Solana(crate::error::SolanaError::invalid_public_key(
                "Solana-style public key found in token_out address".to_string()
            )));
        }
        
        // Check for Solana account-related issues
        if event.user_address.len() == 44 && event.user_address.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(crate::error::DAppError::Solana(crate::error::SolanaError::account_error(
                "Solana-style account address found in user_address".to_string()
            )));
        }
        
        let event_json =
            serde_json::to_string(event).map_err(|e| RedisError::Serialization(e.to_string()))?;

        debug!(
            "Publishing event to Redis channel {}: {}",
            self.channel, event.id
        );

        let mut conn = (*self.connection_manager).clone();
        
        let mut conn = (*self.connection_manager).clone();
        let result: RedisResult<()> = conn.publish(&self.channel, event_json).await;

        match result {
            Ok(_) => {
                debug!("Event published successfully: {}", event.id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to publish event {}: {}", event.id, e);
                
                // Check if this is a timeout error
                if e.to_string().contains("timeout") || e.to_string().contains("timed out") {
                    Err(crate::error::DAppError::Redis(crate::error::RedisError::timeout_error(
                        format!("Redis publish timeout for event {}: {}", event.id, e)
                    )))
                } else {
                    Err(RedisError::Publish(e.to_string()).into())
                }
            }
        }
    }

    /// Publish multiple events in a batch
    pub async fn publish_batch(&self, events: &[SwapEvent]) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        debug!("Publishing batch of {} events to Redis", events.len());

        let mut conn = (*self.connection_manager).clone();

        // Use pipeline for batch publishing
        let mut pipe = redis::pipe();
        for event in events {
            let event_json = serde_json::to_string(event)
                .map_err(|e| {
                    // Use EventParsing error for JSON serialization failures
                    crate::error::DAppError::Ethereum(crate::error::EthereumError::EventParsing(format!("Failed to serialize event to JSON: {}", e)))
                })?;
            pipe.publish(&self.channel, event_json);
        }

        let result: RedisResult<()> = pipe.query_async(&mut conn).await;

        match result {
            Ok(_) => {
                debug!("Batch of {} events published successfully", events.len());
                Ok(())
            }
            Err(e) => {
                error!("Failed to publish batch: {}", e);
                
                // Check if this is a timeout error
                if e.to_string().contains("timeout") || e.to_string().contains("timed out") {
                    Err(crate::error::DAppError::Redis(crate::error::RedisError::timeout_error(
                        format!("Redis batch publish timeout: {}", e)
                    )))
                } else {
                    Err(RedisError::Publish(e.to_string()).into())
                }
            }
        }
    }

    /// Start publishing events from a channel receiver
    #[allow(dead_code)]
    pub async fn start_publishing(
        self,
        mut event_receiver: mpsc::Receiver<SwapEvent>,
    ) -> Result<()> {
        info!("Starting Redis publisher for channel: {}", self.channel);

        // Test if we can access the channel by checking if it exists
        let mut conn = (*self.connection_manager).clone();
        let channel_exists: RedisResult<bool> = conn.exists(&self.channel).await;
        
        if let Err(e) = channel_exists {
            return Err(crate::error::DAppError::Redis(crate::error::RedisError::subscribe_error(
                format!("Failed to check Redis channel {}: {}", self.channel, e)
            )));
        }

        while let Some(event) = event_receiver.recv().await {
            if let Err(e) = self.publish_event(&event).await {
                error!("Failed to publish event: {}", e);
                // Continue processing other events
            }
        }

        info!("Redis publisher stopped");
        Ok(())
    }

    /// Test Redis connection
    pub async fn test_connection(&self) -> Result<()> {
        let mut conn = (*self.connection_manager).clone();

        // Use a simple command to test connection
        let result: RedisResult<()> = conn.set("test_connection", "ok").await;

        match result {
            Ok(_) => {
                debug!("Redis connection test successful");
                Ok(())
            }
            Err(e) => {
                error!("Redis connection test failed: {}", e);
                
                // Check if this is a timeout error
                if e.to_string().contains("timeout") || e.to_string().contains("timed out") {
                    Err(crate::error::DAppError::Redis(crate::error::RedisError::timeout_error(
                        format!("Redis connection test timeout: {}", e)
                    )))
                } else {
                    Err(RedisError::Connection(e.to_string()).into())
                }
            }
        }
    }

    /// Get Redis server info
    #[allow(dead_code)]
    pub async fn get_info(&self) -> Result<String> {
        let mut _conn = (*self.connection_manager).clone();

        // Use a simple command to get basic info
        let result: RedisResult<String> = _conn.get("redis_version").await;

        match result {
            Ok(version) => Ok(format!("Redis version: {}", version)),
            Err(e) => {
                error!("Failed to get Redis info: {}", e);
                Err(RedisError::Connection(e.to_string()).into())
            }
        }
    }

    /// Get subscriber count for the channel
    #[allow(dead_code)]
    pub async fn get_subscriber_count(&self) -> Result<u64> {
        let _conn = (*self.connection_manager).clone();

        // For now, return a default value since pubsub commands are complex
        // In a real implementation, you might want to use a different approach
        Ok(0)
    }

    /// Publish with retry logic
    #[allow(dead_code)]
    async fn publish_with_retry(&self, event: &SwapEvent, max_retries: u32) -> Result<()> {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < max_retries {
            match self.publish_event(event).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    last_error = Some(e);
                    attempts += 1;

                    if attempts < max_retries {
                        let delay = tokio::time::Duration::from_millis(
                            self.config.redis.retry_delay_ms * attempts as u64,
                        );
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| RedisError::Publish("Max retries exceeded".to_string()).into()))
    }
}

/// Pool of Redis publishers for load balancing
#[allow(dead_code)]
pub struct RedisPublisherPool {
    publishers: Vec<RedisPublisher>,
    current_index: usize,
}

impl RedisPublisherPool {
    /// Create a new publisher pool
    #[allow(dead_code)]
    pub async fn new(config: &AppConfig, pool_size: usize) -> Result<Self> {
        let mut publishers = Vec::with_capacity(pool_size);

        for _ in 0..pool_size {
            let publisher = RedisPublisher::new(config.clone()).await?;
            publishers.push(publisher);
        }

        Ok(Self {
            publishers,
            current_index: 0,
        })
    }

    /// Get next publisher from the pool (round-robin)
    #[allow(dead_code)]
    pub fn get_publisher(&mut self) -> &RedisPublisher {
        let publisher = &self.publishers[self.current_index];
        self.current_index = (self.current_index + 1) % self.publishers.len();
        publisher
    }

    /// Get all publishers
    #[allow(dead_code)]
    pub fn get_all_publishers(&self) -> &[RedisPublisher] {
        &self.publishers
    }
}
