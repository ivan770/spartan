use std::{borrow::Cow, path::Path};

use serde::{Deserialize, Serialize};

pub(super) fn default_persistence() -> Option<Persistence<'static>> {
    Some(Persistence::Snapshot(SnapshotConfig::default()))
}

/// Default database path
fn default_path() -> Cow<'static, Path> {
    Cow::Borrowed(Path::new("./db"))
}

/// Default amount of seconds between persistence jobs
const fn default_snapshot_timer() -> u64 {
    900
}

#[derive(Serialize, Deserialize)]
pub enum Persistence<'a> {
    Log(LogConfig<'a>),
    Snapshot(SnapshotConfig<'a>),
}

impl Persistence<'_> {
    pub fn path(&self) -> &Path {
        match self {
            Persistence::Log(config) => &config.path,
            Persistence::Snapshot(config) => &config.path,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SnapshotConfig<'a> {
    /// Database path
    #[serde(default = "default_path")]
    pub path: Cow<'a, Path>,

    /// Amount of seconds between snapshot creation
    #[serde(default = "default_snapshot_timer")]
    pub timer: u64,
}

impl Default for SnapshotConfig<'_> {
    fn default() -> Self {
        SnapshotConfig {
            path: default_path(),
            timer: default_snapshot_timer(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LogConfig<'a> {
    /// Database path
    #[serde(default = "default_path")]
    pub path: Cow<'a, Path>,
}

impl Default for LogConfig<'_> {
    fn default() -> Self {
        LogConfig {
            path: default_path(),
        }
    }
}
