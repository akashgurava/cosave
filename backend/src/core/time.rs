use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the current UTC timestamp as epoch seconds (seconds since Jan 1, 1970).
/// Authoritative Single Source of Truth (SSOT) for all backend timestamp generation.
pub(crate) fn now_epoch_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now_epoch_secs_returns_reasonable_timestamp() {
        let ts = now_epoch_secs();
        // Timestamp must be after 2024-01-01 (1704067200)
        assert!(ts > 1_704_067_200);
    }
}
