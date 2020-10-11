use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub(super) fn default_persistence() -> Option<Persistence> {
    Some(Persistence::Snapshot(SnapshotConfig::default()))
}

/// Default database path
fn default_path() -> Box<Path> {
    PathBuf::from("./db").into_boxed_path()
}

/// Default amount of seconds between persistence jobs
const fn default_snapshot_timer() -> u64 {
    900
}

#[derive(Serialize, Deserialize)]
pub enum Persistence {
    Log(LogConfig),
    Snapshot(SnapshotConfig),
}

#[derive(Serialize, Deserialize)]
pub struct SnapshotConfig {
    /// Database path
    #[serde(default = "default_path")]
    pub path: Box<Path>,

    /// Amount of seconds between snapshot creation
    #[serde(default = "default_snapshot_timer")]
    pub timer: u64,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        SnapshotConfig {
            path: default_path(),
            timer: default_snapshot_timer(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LogConfig {
    /// Database path
    #[serde(default = "default_path")]
    pub path: Box<Path>,
}

impl Default for LogConfig {
    fn default() -> Self {
        LogConfig {
            path: default_path(),
        }
    }
}
