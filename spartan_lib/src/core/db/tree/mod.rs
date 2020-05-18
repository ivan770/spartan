use super::StatusAwareDatabase;
use crate::core::{
    db::Database,
    payload::{Identifiable, Sortable},
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

type MessageStore<M> = HashMap<<M as Identifiable>::Id, (u64, M)>;
type Tree<M> = BTreeMap<(<M as Sortable>::Sort, u64), <M as Identifiable>::Id>;

#[derive(Serialize, Deserialize)]
#[serde(bound = "M: Serialize + DeserializeOwned")]
pub struct TreeDatabase<M>
where
    M: Identifiable + Sortable,
    <M as Identifiable>::Id: Hash,
{
    last_insert_id: u64,
    #[serde(bound = "<M as Identifiable>::Id: Serialize + DeserializeOwned")]
    objects: MessageStore<M>,
    #[serde(bound = "<M as Sortable>::Sort: Serialize + DeserializeOwned")]
    queue_tree: Tree<M>,
}

impl<M> Default for TreeDatabase<M>
where
    M: Identifiable + Sortable,
    <M as Identifiable>::Id: Hash,
{
    fn default() -> Self {
        TreeDatabase {
            last_insert_id: 0,
            objects: HashMap::new(),
            queue_tree: BTreeMap::new(),
        }
    }
}

impl<M> Database<M> for TreeDatabase<M>
where
    M: Identifiable + Sortable,
    <M as Identifiable>::Id: Hash,
{
    type PositionKey = <M as Identifiable>::Id;

    fn push_raw(&mut self, message: M) {
        let id = self.last_insert_id;
        self.last_insert_id += 1;

        self.queue_tree.insert((message.sort(), id), message.id());
        self.objects.insert(message.id(), (id, message));
    }

    fn position<F>(&self, predicate: F) -> Option<Self::PositionKey>
    where
        F: Fn(&M) -> bool,
    {
        Some(self.queue_tree.values().find_map(|key| {
            let message = self.objects.get(key).unwrap();
            if predicate(&message.1) {
                Some(message.1.id())
            } else {
                None
            }
        })?)
    }

    fn get(&self, position: Self::PositionKey) -> Option<&M> {
        Some(&self.objects.get(&position)?.1)
    }

    fn get_mut(&mut self, position: Self::PositionKey) -> Option<&mut M> {
        Some(&mut self.objects.get_mut(&position)?.1)
    }

    fn delete_pos(&mut self, position: Self::PositionKey) -> Option<()> {
        let message = self.objects.remove(&position)?;
        self.queue_tree
            .remove(&(message.1.sort(), message.0))
            .unwrap();
        Some(())
    }

    fn retain<F>(&mut self, predicate: F)
    where
        F: Fn(&M) -> bool,
    {
        let tree = &mut self.queue_tree;

        self.objects.retain(|_, (id, message)| {
            let preserve = predicate(message);

            if !preserve {
                tree.remove(&(message.sort(), *id));
            }

            preserve
        });
    }

    fn len(&self) -> usize {
        self.objects.len()
    }

    fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    fn clear(&mut self) {
        self.objects.clear();
        self.objects.shrink_to_fit();
        self.queue_tree.clear();
    }
}

impl<M> StatusAwareDatabase<M> for TreeDatabase<M>
where
    M: Identifiable + Sortable,
    <M as Identifiable>::Id: Hash,
{
    type RequeueKey = <M as Identifiable>::Id;

    fn reserve(&mut self, position: Self::PositionKey) -> Option<&mut M> {
        let message = self.objects.get_mut(&position)?;
        self.queue_tree.remove(&(message.1.sort(), message.0));
        Some(&mut message.1)
    }

    fn requeue<F>(&mut self, position: Self::RequeueKey, predicate: F) -> Option<&mut M>
    where
        F: Fn(&M) -> bool,
    {
        let message = self.objects.get_mut(&position)?;

        if predicate(&message.1) {
            self.queue_tree
                .insert((message.1.sort(), message.0), position);
            Some(&mut message.1)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TreeDatabase;
    use crate::core::{
        db::Database,
        message::{builder::MessageBuilder, Message},
        payload::Status,
    };
    use chrono::Utc;

    fn create_database() -> TreeDatabase<Message> {
        TreeDatabase::default()
    }

    macro_rules! create_message {
        () => {
            MessageBuilder::default()
                .body(b"Hello world")
                .compose()
                .unwrap()
        };
        ($delay:expr) => {
            MessageBuilder::default()
                .body(b"Hello world")
                .delay($delay)
                .compose()
                .unwrap()
        };
    }

    macro_rules! position {
        ($database:expr, $message:expr) => {
            let pos = $database.position(|_| true).unwrap();
            assert_eq!($database.get(pos).unwrap().id, $message.id);
            $database.delete_pos(pos).unwrap();
        };
    }

    #[test]
    fn test_push() {
        let mut database = create_database();
        let message = create_message!();
        database.push_raw(message);
        assert_eq!(database.objects.len(), 1);
        assert_eq!(database.queue_tree.len(), 1);
    }

    #[test]
    fn test_position_get() {
        let mut database = create_database();
        let message1 = create_message!();
        let message2 = create_message!(|_| Utc::today().and_hms(1, 0, 0).timestamp());
        let message3 = create_message!(|_| Utc::today().and_hms(4, 0, 0).timestamp());
        let message4 = create_message!(|_| Utc::today().and_hms(3, 0, 0).timestamp());
        database.push_raw(message1.clone());
        database.push_raw(message2.clone());
        database.push_raw(message3.clone());
        database.push_raw(message4.clone());

        position!(database, message1);
        position!(database, message2);
        position!(database, message4);
        position!(database, message3);
    }

    #[test]
    fn test_get_mut() {
        let mut database = create_database();
        let message = create_message!();
        database.push_raw(message);
        let pos = database.position(|_| true).unwrap();
        let message = database.get_mut(pos).unwrap();
        message.reserve();
    }

    #[test]
    fn test_delete() {
        let mut database = create_database();
        let message1 = create_message!();
        let message2 = create_message!(|_| Utc::today().and_hms(1, 0, 0).timestamp());
        database.push_raw(message1);
        database.push_raw(message2.clone());
        assert_eq!(database.objects.len(), 2);
        assert_eq!(database.queue_tree.len(), 2);
        let pos = database.position(|_| true).unwrap();
        database.delete_pos(pos).unwrap();
        assert_eq!(database.objects.len(), 1);
        assert_eq!(database.queue_tree.len(), 1);
        let pos = database.position(|_| true).unwrap();
        assert_eq!(database.get(pos).unwrap().id, message2.id);
    }

    #[test]
    fn test_retain() {
        let mut database = create_database();
        let message1 = create_message!();
        let message2 = create_message!();
        database.push_raw(message1);
        database.push_raw(message2.clone());
        database.retain(|message| message.id == message2.id);
        assert_eq!(database.objects.len(), 1);
        assert_eq!(database.queue_tree.len(), 1);
        let pos = database.position(|_| true).unwrap();
        assert_eq!(database.get(pos).unwrap().id, message2.id);
    }

    #[test]
    fn test_len_clear() {
        let mut database = create_database();
        assert_eq!(database.len(), 0);
        database.push_raw(create_message!());
        database.push_raw(create_message!());
        database.push_raw(create_message!());
        database.push_raw(create_message!());
        assert_eq!(database.len(), 4);
        database.clear();
        assert_eq!(database.len(), 0);
    }

    #[test]
    fn test_is_empty() {
        let mut db = create_database();
        assert!(db.is_empty());
        db.push_raw(create_message!());
        assert!(!db.is_empty());
    }
}

#[cfg(test)]
mod dispatcher_tests {
    use super::TreeDatabase;
    use crate::core::dispatcher::simple::PositionBasedDelete;

    crate::test_dispatcher!(TreeDatabase);
    crate::test_status_dispatcher!(TreeDatabase);
}
