use std::{borrow::Cow, path::Path};

use serde::{Deserialize, Serialize};

pub(super) fn default_persistence() -> Option<Persistence<'static>> {
    Some(Persistence::Snapshot(PersistenceConfig::default()))
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
    Log(PersistenceConfig<'a>),
    Snapshot(PersistenceConfig<'a>),
}

impl Persistence<'_> {
    pub fn config(&self) -> &PersistenceConfig<'_> {
        match self {
            Persistence::Log(config) => &config,
            Persistence::Snapshot(config) => &config,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PersistenceConfig<'a> {
    /// Database path
    #[serde(default = "default_path")]
    pub path: Cow<'a, Path>,

    /// Amount of seconds between snapshot creation
    /// When using log config, this value defines interval for replication log snapshots
    #[serde(default = "default_snapshot_timer")]
    pub timer: u64,
}

impl Default for PersistenceConfig<'_> {
    fn default() -> Self {
        PersistenceConfig {
            path: default_path(),
            timer: default_snapshot_timer(),
        }
    }
}
