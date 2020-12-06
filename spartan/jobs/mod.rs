/// Ctrl-C handler
pub mod exit;

/// GC handler
pub mod gc;

/// Persistence handler
pub mod persistence;

#[cfg(feature = "replication")]
/// Replication job
pub mod replication;

#[macro_export]
macro_rules! dispatch_jobs {
    ( $manager:ident, $job:expr ) => {
        let cloned_manager = $manager.clone();
        ::tokio::spawn(async move { $job(&cloned_manager).await });
    };

    ( $manager:ident, $job:expr, $($jobs:expr),+ ) => {
        dispatch_jobs!($manager, $job);
        dispatch_jobs!($manager, $($jobs),+);
    }
}
