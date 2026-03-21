use std::collections::HashMap;
use std::sync::Arc;

use nostr_sdk::prelude::*;
use tokio::sync::RwLock;

#[derive(Default)]
pub enum SyncStatus {
    Syncing,
    #[default]
    Success,
    Failed,
}

impl SyncStatus {
    #[inline]
    pub fn is_syncing(&self) -> bool {
        matches!(self, SyncStatus::Syncing)
    }
}

#[derive(Default)]
pub struct PublicKeySyncData {
    pub status: SyncStatus,
    pub last_sync_attempt: u64,
    pub last_success_sync: u64,
}

#[derive(Clone)]
pub struct SharedState {
    // TODO: replace this with persistent database
    pub pubkeys: Arc<RwLock<HashMap<PublicKey, PublicKeySyncData>>>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            pubkeys: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
