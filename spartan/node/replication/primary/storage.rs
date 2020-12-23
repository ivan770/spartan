use std::collections::BTreeMap;

use maybe_owned::MaybeOwned;
use serde::{Deserialize, Serialize};

use crate::node::event::Event;

#[derive(Serialize, Deserialize)]
pub struct PrimaryStorage {
    next_index: u64,
    gc_threshold: u64,
    log: BTreeMap<u64, Event<'static>>,
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
    pub fn push(&mut self, event: Event<'static>) {
        self.log.insert(self.next_index, event);
        self.next_index += 1;
    }

    pub fn gc(&mut self) {
        let gc_threshold = self.gc_threshold;

        self.log
            .drain_filter(|index, _| *index <= gc_threshold)
            .for_each(drop);
    }

    /// Get event log slice
    ///
    /// [`None`], if `start <= gc_threshold`
    pub fn slice(
        &self,
        start: u64,
    ) -> Option<Box<[(MaybeOwned<'_, u64>, MaybeOwned<'_, Event<'_>>)]>> {
        debug!("Obtaining event log slice starting from ID {}", start);

        if start > self.gc_threshold {
            Some(
                self.log
                    .range(start..)
                    .map(|(k, v)| (MaybeOwned::Borrowed(k), MaybeOwned::Borrowed(v)))
                    .collect(),
            )
        } else {
            None
        }
    }

    pub fn set_gc_threshold(&mut self, threshold: u64) {
        if self.log.get(&threshold).is_some() {
            self.gc_threshold = threshold;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Event, PrimaryStorage};

    #[test]
    fn test_gc() {
        let mut storage = PrimaryStorage::default();

        for _ in 0..6 {
            storage.push(Event::Pop);
        }

        let slice = storage.slice(1).unwrap();
        let (index, _) = slice.first().unwrap();
        assert_eq!(**index, 1);

        storage.gc_threshold = 4;
        storage.gc();

        let slice = storage.slice(5).unwrap();
        let (index, _) = slice.first().unwrap();
        assert_eq!(**index, 5);
    }

    #[test]
    fn test_index_mismatch() {
        let mut storage = PrimaryStorage::default();

        storage.slice(1).unwrap();

        storage.gc_threshold = 1;
        storage.gc();

        assert!(storage.slice(1).is_none());
    }

    #[test]
    fn test_empty_slice() {
        let storage = PrimaryStorage::default();

        let slice = storage.slice(1).unwrap();
        assert!(slice.first().is_none());
    }

    #[test]
    fn test_push_slice() {
        let mut storage = PrimaryStorage::default();

        for _ in 0..6 {
            storage.push(Event::Pop);
        }

        let slice = storage.slice(1).unwrap();
        assert_eq!(slice.len(), 6);

        let (index, event) = slice.first().unwrap();
        assert_eq!((**index, &**event), (1, &Event::Pop));
    }
}
