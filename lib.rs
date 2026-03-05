//! # logos-time
//!
//! TIME Protocol Logos Core module.
//!
//! This crate implements the `LogosCoreModule` trait, registering TIME Protocol
//! as a composable plugin in the Logos Core runtime.
//!
//! ## What this module does
//!
//! When loaded by a Logos node, this module:
//!   - Subscribes to the Waku work-agreement content topic
//!   - Detects confirmed on-chain payments via the Logos blockchain RPC
//!   - Triggers TIME token + WorkNFT mints through the TimeProtocol contract
//!   - Runs the birthright clock (1 TIME/day per verified human)
//!   - Stores Work NFT metadata to Logos decentralised storage
//!
//! ## Usage in logos-node config
//!
//! ```toml
//! [modules.logos-time]
//! enabled = true
//! world_id_verifier = "0x..."
//! waku_content_topic = "/time/1/work-agreements/proto"
//! lssa_contract_address = "0x..."
//! logos_rpc_url = "ws://localhost:8546"
//! birthright_interval_secs = 86400
//! ```

pub mod birthright;
pub mod mint_trigger;
pub mod service;
pub mod waku_listener;
pub mod config;
pub mod error;
pub mod types;

use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::{
    config::TimeModuleConfig,
    service::TimeService,
    error::TimeModuleError,
};

/// The TIME Protocol Logos Core module.
///
/// Implements the `LogosCoreModule` trait so the Logos Core runtime
/// can dynamically load, start, and stop this module alongside the
/// core messaging, storage, and blockchain components.
pub struct TimeModule {
    config:  TimeModuleConfig,
    service: Arc<RwLock<Option<TimeService>>>,
}

impl TimeModule {
    pub fn new(config: TimeModuleConfig) -> Self {
        Self {
            config,
            service: Arc::new(RwLock::new(None)),
        }
    }

    /// Module name — used by Logos Core for discovery and plugin registry listing.
    pub const NAME: &'static str = "logos-time";

    /// Semantic version of this module.
    pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
}

/// `LogosCoreModule` trait — the Logos Core plugin interface.
///
/// NOTE: Logos Core's final plugin API is being stabilised for the 2026 testnet.
/// This implementation follows the module lifecycle contract described in the
/// Logos Core architecture documentation and will be updated to match the
/// canonical trait signature when published.
#[async_trait]
pub trait LogosCoreModule: Send + Sync {
    /// Unique module identifier.
    fn name(&self) -> &'static str;

    /// Semantic version string.
    fn version(&self) -> &'static str;

    /// Called by Logos Core after all core modules have been initialised.
    /// The module should acquire resources but not yet start processing.
    async fn init(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Called by Logos Core to begin module operation.
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Called by Logos Core on graceful shutdown.
    async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Health check — Logos Core polls this to include module status in node health.
    async fn health_check(&self) -> ModuleHealth;
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModuleHealth {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}

#[async_trait]
impl LogosCoreModule for TimeModule {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn version(&self) -> &'static str {
        Self::VERSION
    }

    async fn init(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            module = Self::NAME,
            version = Self::VERSION,
            "Initialising TIME Protocol Logos Core module"
        );

        let service = TimeService::new(self.config.clone()).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        *self.service.write().await = Some(service);

        info!(
            waku_topic = %self.config.waku_content_topic,
            contract   = %self.config.lssa_contract_address,
            "TIME module initialised"
        );

        Ok(())
    }

    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut guard = self.service.write().await;
        if let Some(svc) = guard.as_mut() {
            svc.start().await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            info!(module = Self::NAME, "TIME module started");
        } else {
            warn!(module = Self::NAME, "start() called before init()");
        }
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut guard = self.service.write().await;
        if let Some(svc) = guard.as_mut() {
            svc.stop().await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            info!(module = Self::NAME, "TIME module stopped");
        }
        Ok(())
    }

    async fn health_check(&self) -> ModuleHealth {
        let guard = self.service.read().await;
        match guard.as_ref() {
            None => ModuleHealth::Unhealthy("Service not initialised".into()),
            Some(svc) => svc.health_check().await,
        }
    }
}

/// Logos Core dynamic module entry point.
///
/// This function is called by the Logos Core runtime when it dynamically
/// loads this plugin from the community module library.
///
/// # Safety
/// This is a C-ABI compatible function required for `cdylib` dynamic loading.
#[no_mangle]
pub extern "C" fn logos_module_create(config_json: *const std::ffi::c_char) -> *mut dyn LogosCoreModule {
    let config_str = unsafe {
        std::ffi::CStr::from_ptr(config_json)
            .to_str()
            .unwrap_or("{}")
    };

    let config: TimeModuleConfig = serde_json::from_str(config_str)
        .unwrap_or_default();

    let module = Box::new(TimeModule::new(config));
    Box::into_raw(module)
}
