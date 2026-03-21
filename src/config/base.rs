use std::collections::HashSet;
use std::net::SocketAddr;

use nostr_sdk::prelude::*;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct BaseWebConfig {
    pub listen_addr: Option<SocketAddr>,
}

#[derive(Debug, Default, Deserialize)]
pub struct BaseNostrConfig {
    pub discovery_relays: Option<HashSet<RelayUrl>>,
}

#[derive(Debug, Default, Deserialize)]
pub struct BaseConfig {
    pub web: BaseWebConfig,
    pub nostr: BaseNostrConfig,
}
