use std::time::{SystemTime, UNIX_EPOCH};

use rand::RngCore;
use sqlx::Error;

pub(crate) fn now_epoch_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub(crate) fn generate_token() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    let mut hex = String::with_capacity(32);
    for b in bytes {
        hex.push_str(&format!("{b:02x}"));
    }
    hex
}

pub(crate) fn is_unique_violation(err: &Error) -> bool {
    if let Error::Database(db_err) = err {
        if db_err.is_unique_violation() {
            return true;
        }
        let msg = db_err.message().to_lowercase();
        if msg.contains("unique") || msg.contains("primary key") {
            return true;
        }
    }
    false
}
