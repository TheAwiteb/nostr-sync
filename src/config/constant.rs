use std::collections::HashSet;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::LazyLock;

use nostr_sdk::prelude::*;

pub(super) const DEFAULT_WEB_LISTEN_ADDR: SocketAddr =
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 44552));
pub(super) static DEFAULT_NOSTR_DISCOVERY_RELAYS: LazyLock<HashSet<RelayUrl>> =
    LazyLock::new(|| {
        HashSet::from([
            RelayUrl::parse("wss://purplepag.es").unwrap(),
            RelayUrl::parse("wss://relay.damus.io").unwrap(),
            RelayUrl::parse("wss://relay.primal.net").unwrap(),
        ])
    });

pub(super) const ENV_WORKDIR: &str = "NOSTR_SYNC_WORKDIR";

pub(super) const ENV_WEB_LISTEN_ADDR: &str = "NOSTR_SYNC_WEB_LISTEN_ADDR";
pub(super) const ENV_NOSTR_DISCOVERY_RELAYS: &str = "NOSTR_SYNC_DISCOVERY_RELAYS";
