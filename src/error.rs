use thiserror::Error;

#[derive(Error, Debug)]
pub enum DAppError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Ethereum error: {0}")]
    Ethereum(#[from] EthereumError),

    #[error("Solana error: {0}")]
    Solana(#[from] SolanaError),

    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),

    #[error("Subgraph error: {0}")]
    Subgraph(#[from] SubgraphError),

    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] SerializationError),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Error, Debug)]
pub enum EthereumError {
    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("Contract error: {0}")]
    Contract(String),

    #[error("Event parsing error: {0}")]
    EventParsing(String),

    #[error("Block error: {0}")]
    Block(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Chain ID mismatch: expected {expected}, got {actual}")]
    ChainIdMismatch { expected: u64, actual: u64 },
}

#[derive(Error, Debug)]
pub enum SolanaError {
    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("Program error: {0}")]
    Program(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Account error: {0}")]
    Account(String),

    #[error("Instruction error: {0}")]
    Instruction(String),

    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),

    #[error("Commitment error: {0}")]
    Commitment(String),
}

#[derive(Error, Debug)]
pub enum RedisError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Publish error: {0}")]
    Publish(String),

    #[error("Subscribe error: {0}")]
    Subscribe(String),

    #[error("Pool error: {0}")]
    Pool(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

#[derive(Error, Debug)]
pub enum SubgraphError {
    #[error("GraphQL error: {0}")]
    GraphQL(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Parsing error: {0}")]
    Parsing(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("HTTP error: {0}")]
    Http(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Connection timeout: {0}")]
    ConnectionTimeout(String),

    #[error("DNS resolution failed: {0}")]
    DnsResolution(String),

    #[error("TLS error: {0}")]
    Tls(String),
}

#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("JSON serialization error: {0}")]
    Json(String),

    #[error("Borsh serialization error: {0}")]
    Borsh(String),

    #[error("Hex encoding error: {0}")]
    Hex(String),

    #[error("Base64 encoding error: {0}")]
    Base64(String),
}

pub type Result<T> = std::result::Result<T, DAppError>;

impl From<std::io::Error> for DAppError {
    fn from(err: std::io::Error) -> Self {
        DAppError::Network(NetworkError::Http(err.to_string()))
    }
}

impl From<serde_json::Error> for DAppError {
    fn from(err: serde_json::Error) -> Self {
        DAppError::Serialization(SerializationError::Json(err.to_string()))
    }
}

impl From<reqwest::Error> for DAppError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            DAppError::Timeout(err.to_string())
        } else if err.is_connect() {
            DAppError::Network(NetworkError::ConnectionTimeout(err.to_string()))
        } else {
            DAppError::Network(NetworkError::Http(err.to_string()))
        }
    }
}

impl From<redis::RedisError> for DAppError {
    fn from(err: redis::RedisError) -> Self {
        DAppError::Redis(RedisError::Connection(err.to_string()))
    }
} 