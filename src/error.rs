use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    #[error(transparent)]
    NostrSqlite(#[from] nostr_sqlite::error::Error),
    #[error(transparent)]
    NostrGossipSqlite(#[from] nostr_gossip_sqlite::error::Error),
    #[error(transparent)]
    NostrClient(#[from] nostr_sdk::client::Error),
    #[error("the specified path is not a directory")]
    NotADirectory,
}
