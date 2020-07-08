use crate::node::replication::event::Event;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize)]
pub struct PrimaryStorage {
    next_index: u64,
    gc_threshold: u64,
    log: BTreeMap<u64, Event>,
}

impl Default for PrimaryStorage {
    fn default() -> Self {
        PrimaryStorage {
            next_index: 1,
            gc_threshold: 0,
            log: BTreeMap::new(),
        }
    }
}

impl PrimaryStorage {
    pub fn push(&mut self, event: Event) {
        self.log.insert(self.next_index, event);
        self.next_index += 1;
    }
}
