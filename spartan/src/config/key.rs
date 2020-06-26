use serde::{Deserialize, Serialize};
use std::{
    borrow::Borrow,
    collections::HashSet,
    hash::{Hash, Hasher},
};

const WILDCARD_QUEUE: &'static str = "*";

#[derive(Serialize, Deserialize, Eq)]
pub struct Key {
    key: String,
    queues: HashSet<String>,
}

impl Key {
    pub fn has_queue(&self, queue: &str) -> bool {
        self.queues.contains(WILDCARD_QUEUE) || self.queues.contains(queue)
    }
}

impl Borrow<str> for Key {
    fn borrow(&self) -> &str {
        self.key.as_str()
    }
}

impl Hash for Key {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state)
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
