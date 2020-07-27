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

    pub fn set_gc_threshold(&mut self, threshold: u64) {
        self.gc_threshold = threshold;
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

        let slice = storage.slice(1);
        let (index, _) = slice.first().unwrap();
        assert_eq!(**index, 1);

        storage.gc_threshold = 4;
        storage.gc();

        let slice = storage.slice(1);
        let (index, _) = slice.first().unwrap();
        assert_eq!(**index, 5);
    }

    #[test]
    fn test_empty_slice() {
        let storage = PrimaryStorage::default();

        let slice = storage.slice(1);
        assert!(slice.first().is_none());
    }

    #[test]
    fn test_push_slice() {
        let mut storage = PrimaryStorage::default();

        for _ in 0..6 {
            storage.push(Event::Pop);
        }

        let slice = storage.slice(1);
        assert_eq!(slice.len(), 6);

        let (index, event) = slice.first().unwrap();
        assert_eq!((**index, &**event), (1, &Event::Pop));
    }
}
