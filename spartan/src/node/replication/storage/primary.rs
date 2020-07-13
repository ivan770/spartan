use crate::node::replication::event::Event;
use maybe_owned::MaybeOwned;
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

    pub fn gc(&mut self) {
        let gc_threshold = self.gc_threshold;

        self.log
            .drain_filter(|index, _| *index <= gc_threshold)
            .for_each(drop);
    }

    pub fn slice(&self, start: u64) -> Box<[(MaybeOwned<'_, u64>, MaybeOwned<'_, Event>)]> {
        self.log
            .range(start..)
            .map(|(k, v)| (MaybeOwned::Borrowed(k), MaybeOwned::Borrowed(v)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::PrimaryStorage;
    use crate::node::replication::event::Event;

    #[test]
    fn test_gc() {
        let mut storage = PrimaryStorage::default();

        for _ in 0..6 {
            storage.push(Event::Pop);
        }

        storage.gc_threshold = 4;
        storage.gc();

        assert_eq!(storage.log.iter().map(|(k, _)| k).next(), Some(&5));
    }
}
