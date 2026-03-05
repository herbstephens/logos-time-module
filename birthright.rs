use tracing::{info, warn, error, debug};
use crate::error::TimeModuleError;

/// The BirthrightClock runs on a 24-hour cadence, making the daily
/// 1 TIME birthright allocation available to verified humans.
///
/// Architecture note:
/// The clock does NOT push birthright tokens automatically. Instead it:
///   1. Advances the "current day" counter on the BirthrightClock.sol contract
///   2. Verified humans then self-claim via the contract (with their World ID proof)
///
/// This design preserves user autonomy — no one can receive TIME without
/// initiating the action themselves. The clock merely opens the window.
///
/// Future integration: The Logos AnonComms payment protocol will allow
/// gasless birthright claims via the offchain relay network.
pub struct BirthrightClock {
    rpc_url:          String,
    contract_address: String,
    world_id_address: String,
    interval_secs:    u64,
    signer_key_env:   String,
}

impl BirthrightClock {
    pub async fn new(
        rpc_url:          String,
        contract_address: String,
        world_id_address: String,
        interval_secs:    u64,
        signer_key_env:   String,
    ) -> Result<Self, TimeModuleError> {
        Ok(Self {
            rpc_url,
            contract_address,
            world_id_address,
            interval_secs,
            signer_key_env,
        })
    }

    /// Main loop: tick once per `interval_secs` (default 86400 = 24h).
    pub async fn run(self) -> Result<(), TimeModuleError> {
        info!(
            interval_hours = self.interval_secs / 3600,
            "BirthrightClock started — 1 TIME/day for every verified human"
        );

        loop {
            let next_tick = self.seconds_until_next_day();
            debug!(secs_until_tick = next_tick, "BirthrightClock: waiting for next day boundary");

            tokio::time::sleep(tokio::time::Duration::from_secs(next_tick)).await;

            match self.tick().await {
                Ok(day) => {
                    info!(
                        day_number = day,
                        "BirthrightClock: day {} opened — 1 TIME claimable by verified humans",
                        day
                    );
                }
                Err(e) => {
                    error!(error = %e, "BirthrightClock tick failed");
                    // Back off and retry — missing a tick is recoverable
                    tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
                }
            }
        }
    }

    /// Execute the daily tick.
    /// In practice this is a no-op on-chain — the BirthrightClock.sol
    /// uses `block.timestamp / 86400` as the day key, so the "tick" is
    /// implicit in block time. This method logs the day boundary and
    /// optionally emits a Waku notification to inform clients.
    async fn tick(&self) -> Result<u64, TimeModuleError> {
        let day_number = self.current_day();

        // TODO: Optionally publish a Waku notification on /time/1/birthright/proto
        // so that client wallets know a new day has opened and they can prompt
        // users to claim their birthright.
        //
        // let notification = BirthrightNotification {
        //     day_number,
        //     claim_window_closes: (day_number + 1) * 86400,
        // };
        // waku_client.publish("/time/1/birthright/proto", notification).await?;

        Ok(day_number)
    }

    /// Current day number (unix timestamp / 86400), matching BirthrightClock.sol.
    fn current_day(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            / 86_400
    }

    /// Seconds until the next UTC midnight (day boundary).
    fn seconds_until_next_day(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let elapsed_today = now % 86_400;
        86_400 - elapsed_today
    }
}
