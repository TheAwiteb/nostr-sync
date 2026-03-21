use std::collections::HashSet;
use std::time::Duration;

use nostr_gossip_sqlite::store::NostrGossipSqlite;
use nostr_sdk::prelude::*;
use nostr_sqlite::store::NostrSqlite;
use tokio::time;

use crate::config::Config;
use crate::error::Error;
use crate::state::{SharedState, SyncStatus};
use crate::util;

const PUBLIC_KEY_SYNC_INTERVAL_ON_FAIL: Duration = Duration::from_secs(60 * 60);
const PUBLIC_KEY_SYNC_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

async fn find_pubkeys_to_sync(state: &SharedState) -> HashSet<PublicKey> {
    // Find public keys that need to be synced
    let public_keys = state.pubkeys.read().await;

    let now: u64 = util::current_timestamp();
    let mut to_sync = HashSet::new();

    for (pubkey, data) in public_keys.iter() {
        // Skip if already syncing
        if data.status.is_syncing() {
            continue;
        }

        let diff1 = now.saturating_sub(data.last_success_sync);
        let diff2 = now.saturating_sub(data.last_sync_attempt);

        if diff1 > PUBLIC_KEY_SYNC_INTERVAL.as_secs()
            || diff2 > PUBLIC_KEY_SYNC_INTERVAL_ON_FAIL.as_secs()
        {
            to_sync.insert(*pubkey);
        }
    }

    to_sync
}

pub async fn run(config: &Config, state: SharedState) -> Result<(), Error> {
    // Open nostr databases
    let events: NostrSqlite = NostrSqlite::builder()
        .in_file(&config.nostr.events_path)
        .build()
        .await?;
    let gossip: NostrGossipSqlite = NostrGossipSqlite::open(&config.nostr.gossip_path).await?;

    let client: Client = Client::builder()
        .database(events)
        .gossip(gossip)
        .sleep_when_idle(SleepWhenIdle::Enabled {
            timeout: Duration::from_secs(300),
        })
        .build();

    for relay in &config.nostr.discovery_relays {
        client
            .add_relay(relay)
            .and_connect()
            .capabilities(RelayCapabilities::DISCOVERY)
            .await?;
    }

    tracing::info!("Starting syncer...");

    loop {
        let pks_to_sync: HashSet<PublicKey> = find_pubkeys_to_sync(&state).await;

        for pubkey in pks_to_sync {
            let state = state.clone();
            let client = client.clone();
            tokio::spawn(async move {
                // Set syncing
                {
                    let mut pks = state.pubkeys.write().await;
                    pks.entry(pubkey).and_modify(|d| {
                        d.status = SyncStatus::Syncing;
                    });
                }

                let now: u64 = util::current_timestamp();

                let filter = Filter::new().author(pubkey);

                // Sync down the events
                let opts = SyncOptions::new().direction(SyncDirection::Down);
                let down_success: bool = match client.sync(filter.clone()).opts(opts).await {
                    Ok(output) => {
                        tracing::info!(success = ?output.success, failed = ?output.failed, "Down sync completed for '{pubkey}'");

                        // Consider success if there is at least one success
                        !output.success.is_empty()
                    }
                    Err(e) => {
                        tracing::error!("Failed to start down sync for '{pubkey}': {e}");
                        false
                    }
                };

                // Sync up the events
                let opts = SyncOptions::new().direction(SyncDirection::Up);
                let up_success: bool = match client.sync(filter.clone()).opts(opts).await {
                    Ok(output) => {
                        tracing::info!(success = ?output.success, failed = ?output.failed, "Up sync completed for '{pubkey}'");

                        // Consider success if there is at least one success
                        !output.success.is_empty()
                    }
                    Err(e) => {
                        tracing::error!("Failed to start up sync for '{pubkey}': {e}");
                        false
                    }
                };

                let mut pks = state.pubkeys.write().await;

                if down_success && up_success {
                    pks.entry(pubkey).and_modify(|d| {
                        d.status = SyncStatus::Success;
                        d.last_sync_attempt = now;
                        d.last_success_sync = now;
                    });
                } else {
                    pks.entry(pubkey).and_modify(|d| {
                        d.status = SyncStatus::Failed;
                        d.last_sync_attempt = now;
                    });
                }
            });
        }

        time::sleep(Duration::from_secs(60)).await;
    }
}
