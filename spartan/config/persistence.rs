use std::{borrow::Cow, path::Path};

use serde::{Deserialize, Serialize};

const fn default_persistence() -> Persistence {
    Persistence::Snapshot
}

/// Default database path
fn default_path() -> Cow<'static, Path> {
    Cow::Borrowed(Path::new("./db"))
}

/// Default amount of seconds between persistence jobs
const fn default_snapshot_timer() -> u64 {
    900
}

const fn default_compaction() -> bool {
    true
}

#[derive(Serialize, Deserialize)]
pub enum Persistence {
    Log,
    Snapshot,
}

#[derive(Serialize, Deserialize)]
pub struct PersistenceConfig<'a> {
    /// Persistence mode
    pub mode: Persistence,

    /// Database path
    #[serde(default = "default_path")]
    pub path: Cow<'a, Path>,

    /// Amount of seconds between snapshot creation
    /// When using log config, this value defines interval for replication log snapshots
    #[serde(default = "default_snapshot_timer")]
    pub timer: u64,

    /// Log compaction on queue restoring from FS
    #[serde(default = "default_compaction")]
    pub compaction: bool,
}

impl Default for PersistenceConfig<'_> {
    fn default() -> Self {
        PersistenceConfig {
            mode: default_persistence(),
            path: default_path(),
            timer: default_snapshot_timer(),
            compaction: default_compaction(),
        }
    }
}
