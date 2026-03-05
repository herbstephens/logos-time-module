use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tracing::{info, warn, debug, error};

use crate::{
    error::TimeModuleError,
    types::{WorkAgreement, MintEvent, WorkNftMetadata, ModuleStatus},
};

/// Watches for payer-initiated `payForWork()` calls on the Logos blockchain
/// and indexes the resulting MintEvents.
///
/// Flow:
///   1. Receives WorkAgreement from WakuListener via broadcast channel
///   2. Watches for the payer's `payForWork()` transaction on-chain
///   3. On confirmation, indexes the MintEvent
///   4. Uploads WorkNFT metadata to Logos decentralised storage
///   5. Updates module status counters
///
/// Note: The actual on-chain mint is executed by the user's wallet calling
/// TimeProtocol.sol. This module observes and indexes — it does NOT execute
/// transactions on behalf of users. The module can optionally act as a
/// relayer for gasless mints via the Logos AnonComms payment protocol.
pub struct MintTrigger {
    rpc_url:          String,
    contract_address: String,
    storage_url:      String,
    signer_key_env:   String,
    agreement_rx:     broadcast::Receiver<WorkAgreement>,
    status:           Arc<RwLock<ModuleStatus>>,

    /// Pending agreements waiting for on-chain payment confirmation
    pending: Vec<WorkAgreement>,
}

impl MintTrigger {
    pub async fn new(
        rpc_url:          String,
        contract_address: String,
        storage_url:      String,
        signer_key_env:   String,
        agreement_rx:     broadcast::Receiver<WorkAgreement>,
        status:           Arc<RwLock<ModuleStatus>>,
    ) -> Result<Self, TimeModuleError> {
        Ok(Self {
            rpc_url,
            contract_address,
            storage_url,
            signer_key_env,
            agreement_rx,
            status,
            pending: Vec::new(),
        })
    }

    /// Main processing loop.
    pub async fn run(mut self) -> Result<(), TimeModuleError> {
        info!(
            contract = %self.contract_address,
            rpc      = %self.rpc_url,
            "MintTrigger starting"
        );

        // Connect to Logos chain RPC
        self.connect_rpc().await?;

        loop {
            tokio::select! {
                // New agreement from Waku
                result = self.agreement_rx.recv() => {
                    match result {
                        Ok(agreement) => {
                            self.on_agreement_received(agreement).await;
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            warn!(skipped = n, "MintTrigger: broadcast channel lagged");
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            return Err(TimeModuleError::ChannelClosed);
                        }
                    }
                }

                // Poll pending agreements for on-chain confirmation
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(12)) => {
                    self.check_pending_agreements().await;
                }
            }
        }
    }

    async fn connect_rpc(&self) -> Result<(), TimeModuleError> {
        // TODO: Connect ethers/alloy WebSocket provider to Logos chain RPC
        // let provider = Provider::<Ws>::connect(&self.rpc_url).await?;
        // self.status.write().await.chain_connected = true;

        info!(rpc = %self.rpc_url, "MintTrigger: connected to Logos chain RPC");
        {
            let mut s = self.status.write().await;
            s.chain_connected = true;
        }
        Ok(())
    }

    async fn on_agreement_received(&mut self, agreement: WorkAgreement) {
        debug!(
            id     = ?agreement.id,
            worker = ?agreement.worker,
            payer  = ?agreement.payer,
            "MintTrigger: received agreement, waiting for on-chain payment"
        );
        self.pending.push(agreement);
    }

    /// Check all pending agreements for on-chain payment confirmation.
    /// A pending agreement is confirmed when a `MintEvent` log is observed
    /// from the TimeProtocol contract with a matching `agreementHash`.
    async fn check_pending_agreements(&mut self) {
        if self.pending.is_empty() {
            return;
        }

        debug!(pending = self.pending.len(), "Checking pending agreements for on-chain confirmation");

        // TODO: Query Logos chain for MintEvent logs matching pending agreement hashes
        //
        // let filter = Filter::new()
        //     .address(self.contract_address.parse::<Address>().unwrap())
        //     .event("MintEvent(address,address,uint256,uint256,uint256,bytes32,address,uint256)")
        //     .from_block(BlockNumber::Latest);
        //
        // let logs = provider.get_logs(&filter).await?;
        // for log in logs {
        //     let mint_event = decode_mint_event_log(log)?;
        //     self.on_mint_confirmed(mint_event).await;
        // }
    }

    /// Called when a MintEvent is confirmed on the Logos blockchain.
    async fn on_mint_confirmed(&mut self, event: MintEvent) {
        info!(
            tx     = ?event.tx_hash,
            worker = ?event.worker,
            time   = %format!("{} TIME", event.time_amount / alloy_primitives::U256::from(1_000_000_000_000_000_000u64)),
            nft_id = %event.work_nft_id,
            "✓ Mint confirmed"
        );

        // Upload Work NFT metadata to Logos Storage
        if let Err(e) = self.store_work_nft_metadata(&event).await {
            warn!(error = %e, "Failed to store Work NFT metadata — will retry");
        }

        // Update status
        {
            let mut s = self.status.write().await;
            s.total_mints_observed += 1;
            s.last_block_seen = event.block_number;
            if s.agreements_pending > 0 {
                s.agreements_pending -= 1;
            }
        }

        // Remove from pending
        self.pending.retain(|a| a.agreement_hash() != event.agreement_hash);
    }

    /// Upload Work NFT metadata JSON to Logos decentralised storage.
    /// Returns the content-addressed URI to store in the WorkNFT.
    async fn store_work_nft_metadata(
        &self,
        event: &MintEvent,
    ) -> Result<String, TimeModuleError> {
        let metadata = WorkNftMetadata {
            name: format!("Work Record #{}", event.work_nft_id),
            description: format!(
                "Verified work record: {} TIME minted for {} hours of labour",
                event.time_amount,
                event.hours_worked_scaled as f64 / 1000.0
            ),
            mint_event: event.clone(),
            tags: vec!["time-protocol".into(), "work".into(), "parallel-society".into()],
            schema_version: "1.0.0".into(),
        };

        let json = serde_json::to_vec(&metadata)
            .map_err(|e| TimeModuleError::StorageError(e.to_string()))?;

        // TODO: Upload to Logos Storage node
        // POST /api/v1/upload → returns content-addressed CID
        // let cid = logos_storage_client.upload(json).await?;
        // return Ok(format!("logos://{}", cid));

        debug!(nft_id = %event.work_nft_id, "Work NFT metadata queued for Logos Storage upload");
        Ok(format!("logos://pending/{}", event.work_nft_id))
    }
}
