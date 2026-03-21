use std::time::Duration;

use nostr_gossip_sqlite::store::NostrGossipSqlite;
use nostr_sdk::prelude::*;
use nostr_sqlite::store::NostrSqlite;
use tokio::time;

use crate::config::Config;
use crate::error::Error;

pub async fn run(config: &Config) -> Result<(), Error> {
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
            .capabilities(RelayCapabilities::DISCOVERY)
            .await?;
    }

    tracing::info!("Starting syncer...");

    loop {
        // TODO: read data from the app database and periodically sync events

        time::sleep(Duration::from_secs(60)).await;
    }
}
