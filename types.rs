use serde::{Deserialize, Serialize};
use alloy_primitives::{Address, B256, U256};

/// A signed work agreement published to the Waku content topic.
///
/// Both parties (worker and payer) sign this off-chain before the payer
/// calls `TimeProtocol.payForWork()` on-chain. The hash of this struct
/// becomes the `agreementHash` stored in the WorkNFT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkAgreement {
    /// Unique identifier for this agreement
    pub id: B256,

    /// The worker's Logos address
    pub worker: Address,

    /// The payer's Logos address
    pub payer: Address,

    /// Hours worked, scaled by 1000 (e.g., 1500 = 1.5 hours)
    pub hours_worked_scaled: u64,

    /// ERC-20 token address for payment (zero address = native token)
    pub payment_token: Address,

    /// Payment amount in token's base unit
    pub payment_amount: U256,

    /// Human-readable description of the work performed
    pub description: String,

    /// Unix timestamp when agreement was created
    pub created_at: u64,

    /// Unix timestamp when agreement expires (0 = no expiry)
    pub expires_at: u64,

    /// Worker's EIP-712 signature over the agreement
    pub worker_signature: Vec<u8>,

    /// Payer's EIP-712 signature over the agreement (set when payer countersigns)
    pub payer_signature: Option<Vec<u8>>,
}

impl WorkAgreement {
    /// Compute the keccak256 hash of this agreement for on-chain storage.
    pub fn agreement_hash(&self) -> B256 {
        use alloy_primitives::keccak256;
        let encoded = serde_json::to_vec(self).unwrap_or_default();
        keccak256(&encoded)
    }

    /// Check if both parties have signed.
    pub fn is_countersigned(&self) -> bool {
        self.payer_signature.is_some()
    }
}

/// A mint event observed on the Logos blockchain.
/// Emitted by TimeProtocol.sol and indexed by this module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintEvent {
    /// Transaction hash of the payment
    pub tx_hash: B256,

    /// Block number of the mint
    pub block_number: u64,

    /// Worker address that received TIME + WorkNFT
    pub worker: Address,

    /// Payer address that triggered the mint
    pub payer: Address,

    /// TIME tokens minted (in wei, 18 decimals)
    pub time_amount: U256,

    /// ID of the WorkNFT minted
    pub work_nft_id: U256,

    /// Hours worked (scaled x1000)
    pub hours_worked_scaled: u64,

    /// Agreement hash linking this mint to the off-chain WorkAgreement
    pub agreement_hash: B256,

    /// Payment token address
    pub payment_token: Address,

    /// Payment amount
    pub payment_amount: U256,
}

/// Work NFT metadata stored in Logos decentralised storage.
/// The content URI in the WorkNFT points to this structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkNftMetadata {
    /// Standard NFT name field
    pub name: String,

    /// Human-readable description of the work
    pub description: String,

    /// The mint event that created this NFT
    pub mint_event: MintEvent,

    /// Optional: categories/tags for the work
    pub tags: Vec<String>,

    /// Schema version for forward compatibility
    pub schema_version: String,
}

/// Status of the TIME module components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleStatus {
    pub waku_connected:       bool,
    pub chain_connected:      bool,
    pub storage_connected:    bool,
    pub agreements_pending:   usize,
    pub total_mints_observed: u64,
    pub last_block_seen:      u64,
}
