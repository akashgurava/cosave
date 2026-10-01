use sqlx::Error;

pub(crate) use crate::core::now_epoch_secs;

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
