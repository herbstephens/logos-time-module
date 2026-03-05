use thiserror::Error;

#[derive(Debug, Error)]
pub enum TimeModuleError {
    #[error("RPC connection failed: {0}")]
    RpcError(String),

    #[error("Waku connection failed: {0}")]
    WakuError(String),

    #[error("Message decode error: {0}")]
    MessageDecodeError(String),

    #[error("Invalid work agreement: {0}")]
    InvalidAgreement(String),

    #[error("Invalid signature: {0}")]
    SignatureError(String),

    #[error("Contract interaction failed: {0}")]
    ContractError(String),

    #[error("Logos Storage error: {0}")]
    StorageError(String),

    #[error("Internal broadcast channel closed")]
    ChannelClosed,

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Daily cap exceeded for worker {worker}: attempted {attempted}, remaining {remaining}")]
    DailyCapExceeded {
        worker:    String,
        attempted: String,
        remaining: String,
    },

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
