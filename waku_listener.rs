use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tracing::{info, warn, debug, error};

use crate::{
    error::TimeModuleError,
    types::{WorkAgreement, ModuleStatus},
};

/// Subscribes to the TIME Protocol Waku content topic and forwards
/// countersigned WorkAgreements to the MintTrigger via broadcast channel.
///
/// Waku content topic: `/time/1/work-agreements/proto`
///
/// Message flow:
///   Off-chain: worker + payer co-sign WorkAgreement
///   → Payer publishes to Waku topic
///   → WakuListener receives, validates, forwards
///   → MintTrigger watches for on-chain payment confirmation
pub struct WakuListener {
    node_url:      String,
    content_topic: String,
    agreement_tx:  broadcast::Sender<WorkAgreement>,
    status:        Arc<RwLock<ModuleStatus>>,
}

impl WakuListener {
    pub async fn new(
        node_url:      String,
        content_topic: String,
        agreement_tx:  broadcast::Sender<WorkAgreement>,
        status:        Arc<RwLock<ModuleStatus>>,
    ) -> Result<Self, TimeModuleError> {
        Ok(Self { node_url, content_topic, agreement_tx, status })
    }

    /// Main loop: connect to Waku node and process incoming messages.
    pub async fn run(self) -> Result<(), TimeModuleError> {
        info!(
            node = %self.node_url,
            topic = %self.content_topic,
            "WakuListener connecting"
        );

        // In production: use waku-bindings crate to connect to a running Waku node.
        // The implementation below shows the logical flow; the exact waku-bindings
        // API will be wired in once the Logos testnet Waku node is accessible.
        //
        // Reference: https://github.com/waku-org/waku-rust-bindings

        loop {
            match self.connect_and_listen().await {
                Ok(()) => {
                    info!("WakuListener: clean disconnect, will reconnect");
                }
                Err(e) => {
                    error!(error = %e, "WakuListener error, reconnecting in 5s");
                    {
                        let mut s = self.status.write().await;
                        s.waku_connected = false;
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn connect_and_listen(&self) -> Result<(), TimeModuleError> {
        // TODO: Wire in waku-bindings once Logos testnet Waku node is live.
        // Pseudocode for the real implementation:
        //
        //   let node = WakuNode::new(&self.node_url).await?;
        //   node.subscribe(&self.content_topic).await?;
        //   self.status.write().await.waku_connected = true;
        //
        //   while let Some(msg) = node.recv().await {
        //       if let Ok(agreement) = self.decode_message(msg).await {
        //           self.handle_agreement(agreement).await?;
        //       }
        //   }

        {
            let mut s = self.status.write().await;
            s.waku_connected = true;
        }

        info!(
            topic = %self.content_topic,
            "WakuListener subscribed — waiting for work agreements"
        );

        // Simulation loop for testnet development
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            debug!("WakuListener: heartbeat — listening for agreements");
        }
    }

    /// Decode and validate a raw Waku message into a WorkAgreement.
    async fn decode_message(&self, payload: Vec<u8>) -> Result<WorkAgreement, TimeModuleError> {
        let agreement: WorkAgreement = serde_json::from_slice(&payload)
            .map_err(|e| TimeModuleError::MessageDecodeError(e.to_string()))?;

        // Validate: must have both signatures
        if !agreement.is_countersigned() {
            return Err(TimeModuleError::InvalidAgreement(
                "Agreement missing payer signature".into()
            ));
        }

        // Validate: not expired
        if agreement.expires_at > 0 {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            if now > agreement.expires_at {
                return Err(TimeModuleError::InvalidAgreement(
                    "Agreement has expired".into()
                ));
            }
        }

        // TODO: Verify EIP-712 signatures from both worker and payer
        // self.verify_signatures(&agreement)?;

        debug!(
            agreement_id = ?agreement.id,
            worker = ?agreement.worker,
            payer  = ?agreement.payer,
            hours  = agreement.hours_worked_scaled,
            "WorkAgreement received and validated"
        );

        Ok(agreement)
    }

    async fn handle_agreement(&self, agreement: WorkAgreement) -> Result<(), TimeModuleError> {
        {
            let mut s = self.status.write().await;
            s.agreements_pending += 1;
        }

        self.agreement_tx.send(agreement)
            .map_err(|_| TimeModuleError::ChannelClosed)?;

        Ok(())
    }
}
