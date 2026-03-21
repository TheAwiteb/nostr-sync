use std::time::{SystemTime, UNIX_EPOCH};

#[inline]
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
