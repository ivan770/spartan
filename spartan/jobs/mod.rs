/// Ctrl-C handler
pub mod exit;

/// GC handler
pub mod gc;

/// Persistence handler
pub mod persistence;

#[cfg(feature = "replication")]
/// Replication job
pub mod replication;
