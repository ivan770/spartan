/// Queue access key
pub mod key;

/// Replication config
pub mod replication;

/// Persistence config
pub mod persistence;

use key::Key;
use replication::Replication;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashSet;

use persistence::PersistenceConfig;

/// Default amount of seconds between GC jobs
const fn default_gc_timer() -> u64 {
    300
}

fn default_persistence() -> Option<PersistenceConfig<'static>> {
    Some(PersistenceConfig::default())
}

fn serialize_persistence<'a, S>(value: &Option<PersistenceConfig<'a>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer
{
    if let Some(config) = value.as_ref() {
        config.serialize(serializer)
    } else {
        PersistenceConfig::default().serialize(serializer)
    }
}

/// Server configuration
#[derive(Serialize, Deserialize)]
pub struct Config<'a> {
    /// Max body size in bytes
    /// Default value is defined in Actix source code
    pub body_size: Option<usize>,

    /// Amount of seconds between GC jobs
    #[serde(default = "default_gc_timer")]
    #[serde(skip_serializing)]
    pub gc_timer: u64,

    /// Array of queues
    pub queues: Box<[Box<str>]>,

    /// Persistence encryption key
    pub encryption_key: Option<Box<str>>,

    /// Queue access keys
    pub access_keys: Option<HashSet<Key>>,

    /// Replication config
    pub replication: Option<Replication>,

    /// Persistence config
    #[serde(serialize_with = "serialize_persistence")]
    pub persistence: Option<PersistenceConfig<'a>>,
}

#[cfg(not(test))]
impl Default for Config<'_> {
    fn default() -> Self {
        Config {
            body_size: None,
            gc_timer: default_gc_timer(),
            queues: Box::new([]),
            encryption_key: None,
            access_keys: None,
            replication: None,
            persistence: default_persistence(),
        }
    }
}

#[cfg(test)]
impl Default for Config<'_> {
    fn default() -> Self {
        Config {
            body_size: None,
            gc_timer: 10,
            queues: Box::new([
                String::from("test").into_boxed_str(),
                String::from("test_2").into_boxed_str(),
            ]),
            encryption_key: None,
            access_keys: None,
            replication: None,
            persistence: default_persistence(),
        }
    }
}
