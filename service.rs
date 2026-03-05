use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tokio::task::JoinHandle;
use tracing::{info, error, warn};

use crate::{
    ModuleHealth,
    config::TimeModuleConfig,
    error::TimeModuleError,
    mint_trigger::MintTrigger,
    waku_listener::WakuListener,
    birthright::BirthrightClock,
    types::{WorkAgreement, MintEvent, ModuleStatus},
};

/// The central TIME service.
///
/// Owns and coordinates:
///   - `WakuListener`    — subscribes to work agreement messages
///   - `MintTrigger`     — watches chain + fires mints on confirmed payment
///   - `BirthrightClock` — issues 1 TIME/day to verified humans
pub struct TimeService {
    config:         TimeModuleConfig,
    status:         Arc<RwLock<ModuleStatus>>,

    // Internal broadcast channel: WakuListener → MintTrigger
    agreement_tx:   broadcast::Sender<WorkAgreement>,

    // Task handles (set on start)
    waku_handle:        Option<JoinHandle<()>>,
    mint_handle:        Option<JoinHandle<()>>,
    birthright_handle:  Option<JoinHandle<()>>,
}

impl TimeService {
    pub async fn new(config: TimeModuleConfig) -> Result<Self, TimeModuleError> {
        let (agreement_tx, _) = broadcast::channel(256);

        let status = Arc::new(RwLock::new(ModuleStatus {
            waku_connected:       false,
            chain_connected:      false,
            storage_connected:    false,
            agreements_pending:   0,
            total_mints_observed: 0,
            last_block_seen:      0,
        }));

        Ok(Self {
            config,
            status,
            agreement_tx,
            waku_handle:       None,
            mint_handle:       None,
            birthright_handle: None,
        })
    }

    pub async fn start(&mut self) -> Result<(), TimeModuleError> {
        info!("Starting TIME service components");

        // ── 1. Start Waku listener ────────────────────────────────────────────
        let waku_listener = WakuListener::new(
            self.config.waku_node_url.clone(),
            self.config.waku_content_topic.clone(),
            self.agreement_tx.clone(),
            Arc::clone(&self.status),
        ).await?;

        self.waku_handle = Some(tokio::spawn(async move {
            if let Err(e) = waku_listener.run().await {
                error!(error = %e, "Waku listener failed");
            }
        }));

        // ── 2. Start mint trigger ─────────────────────────────────────────────
        let agreement_rx = self.agreement_tx.subscribe();
        let mint_trigger = MintTrigger::new(
            self.config.logos_rpc_url.clone(),
            self.config.lssa_contract_address.clone(),
            self.config.logos_storage_url.clone(),
            self.config.signer_key_env.clone(),
            agreement_rx,
            Arc::clone(&self.status),
        ).await?;

        self.mint_handle = Some(tokio::spawn(async move {
            if let Err(e) = mint_trigger.run().await {
                error!(error = %e, "Mint trigger failed");
            }
        }));

        // ── 3. Start birthright clock ─────────────────────────────────────────
        let birthright_clock = BirthrightClock::new(
            self.config.logos_rpc_url.clone(),
            self.config.lssa_contract_address.clone(),
            self.config.world_id_verifier.clone(),
            self.config.birthright_interval_secs,
            self.config.signer_key_env.clone(),
        ).await?;

        self.birthright_handle = Some(tokio::spawn(async move {
            if let Err(e) = birthright_clock.run().await {
                error!(error = %e, "Birthright clock failed");
            }
        }));

        info!("TIME service started — all components running");
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), TimeModuleError> {
        info!("Stopping TIME service");

        for handle in [
            self.waku_handle.take(),
            self.mint_handle.take(),
            self.birthright_handle.take(),
        ].into_iter().flatten() {
            handle.abort();
        }

        info!("TIME service stopped");
        Ok(())
    }

    pub async fn health_check(&self) -> ModuleHealth {
        let status = self.status.read().await;

        if !status.chain_connected {
            return ModuleHealth::Unhealthy("Not connected to Logos chain RPC".into());
        }
        if !status.waku_connected {
            return ModuleHealth::Degraded("Waku connection unavailable — work agreements paused".into());
        }
        if !status.storage_connected {
            return ModuleHealth::Degraded("Logos Storage unavailable — Work NFT metadata will queue".into());
        }

        ModuleHealth::Healthy
    }
}
