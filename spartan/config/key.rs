use std::{
    borrow::Borrow,
    collections::HashSet,
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};

/// Wildcard queue name. Used in `queues` to represent all available queues in node
const WILDCARD_QUEUE: &str = "*";

/// Single access key
///
/// Contains key value, and queues that are protected with it
///
/// For usage with HashMap/HashSet implements [`Borrow<str>`] and [`Hash`]
#[derive(Serialize, Deserialize, Eq, Clone)]
pub struct Key {
    /// Key value. Passed in request as Bearer token.
    pub key: Box<str>,

    /// Set of queues, that key has access to
    pub queues: HashSet<Box<str>>,
}

impl Key {
    /// Check if user of key has access to provided queue, or if key contains wildcard queue access.
    pub fn has_queue(&self, queue: &str) -> bool {
        self.queues.contains(WILDCARD_QUEUE) || self.queues.contains(queue)
    }
}

impl Borrow<str> for Key {
    fn borrow(&self) -> &str {
        &self.key
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
