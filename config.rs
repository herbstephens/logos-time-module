use serde::{Deserialize, Serialize};

/// Configuration for the TIME Protocol Logos Core module.
/// Loaded from `logos-node/config.toml` under `[modules.logos-time]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeModuleConfig {
    /// Whether this module is active
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// WebSocket RPC URL for the Logos blockchain node
    #[serde(default = "default_rpc")]
    pub logos_rpc_url: String,

    /// Address of the deployed TimeProtocol.sol contract on LSSA
    #[serde(default)]
    pub lssa_contract_address: String,

    /// Address of the World ID verifier contract (for birthright sybil resistance)
    #[serde(default)]
    pub world_id_verifier: String,

    /// Waku content topic for work agreement coordination
    /// Format: /time/1/work-agreements/proto
    #[serde(default = "default_topic")]
    pub waku_content_topic: String,

    /// Waku node WebSocket URL
    #[serde(default = "default_waku")]
    pub waku_node_url: String,

    /// Seconds between birthright clock ticks (default: 86400 = 24h)
    #[serde(default = "default_birthright_interval")]
    pub birthright_interval_secs: u64,

    /// Maximum TIME mintable per day via earned work (default: 23e18)
    #[serde(default = "default_max_earned")]
    pub max_earned_per_day_wei: String,

    /// Logos Storage node URL for Work NFT metadata
    #[serde(default = "default_storage")]
    pub logos_storage_url: String,

    /// Private key for the module's signing wallet (env var recommended)
    /// In production: use a KMS or hardware wallet integration
    #[serde(default)]
    pub signer_key_env: String,
}

impl Default for TimeModuleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            logos_rpc_url: default_rpc(),
            lssa_contract_address: String::new(),
            world_id_verifier: String::new(),
            waku_content_topic: default_topic(),
            waku_node_url: default_waku(),
            birthright_interval_secs: default_birthright_interval(),
            max_earned_per_day_wei: default_max_earned(),
            logos_storage_url: default_storage(),
            signer_key_env: "TIME_MODULE_SIGNER_KEY".to_string(),
        }
    }
}

fn default_enabled() -> bool { true }
fn default_rpc()     -> String { "ws://localhost:8546".to_string() }
fn default_topic()   -> String { "/time/1/work-agreements/proto".to_string() }
fn default_waku()    -> String { "ws://localhost:8547".to_string() }
fn default_storage() -> String { "http://localhost:8090".to_string() }
fn default_birthright_interval() -> u64 { 86_400 }
fn default_max_earned() -> String {
    // 23 * 1e18 as decimal string
    "23000000000000000000".to_string()
}
