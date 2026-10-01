//! Database error classification utilities for category persistence operations.
//!
//! Inspects underlying SQLite engine errors to detect unique constraint and primary key conflicts.
//! Identifying duplicate entries allows persistence workflows to map raw database failures
//! into strongly typed domain conflict errors with actionable messages for the user.

use sqlx::Error;

/// Inspects a [`sqlx::Error`] to determine whether it was caused by a unique constraint violation.
///
/// Checks both the native [`sqlx::error::DatabaseError::is_unique_violation`] predicate and performs
/// fallback string matching on SQLite error messages for compound `UNIQUE` or `PRIMARY KEY` conflicts.
///
/// # Ingress
/// - `err`: Reference to the underlying [`sqlx::Error`] returned by query execution.
///
/// # Returns
/// - `true` if the error originates from a duplicate key or unique constraint violation.
/// - `false` otherwise.
pub(super) fn is_unique_violation(err: &Error) -> bool {
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
