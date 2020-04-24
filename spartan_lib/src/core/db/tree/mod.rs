use crate::core::{
    db::{Database, DatabaseError, Result},
    payload::{Identifiable, Sortable},
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

type MessageStore<M> = HashMap<Uuid, (u64, M)>;
type Tree<M> = BTreeMap<(<M as Sortable>::Sort, u64), Uuid>;

#[derive(Serialize, Deserialize)]
pub struct TreeDatabase<M>
where
    M: Identifiable + Sortable,
{
    last_insert_id: u64,
    objects: MessageStore<M>,
    #[serde(bound = "<M as Sortable>::Sort: Serialize + DeserializeOwned")]
    queue_tree: Tree<M>,
}

impl<M> Default for TreeDatabase<M>
where
    M: Identifiable + Sortable,
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
{
    type PositionKey = Uuid;

    fn push_raw(&mut self, message: M) -> Result<()> {
        let id = self.last_insert_id;
        self.last_insert_id += 1;

        self.queue_tree.insert((message.sort(), id), message.id());
        self.objects.insert(message.id(), (id, message));
        Ok(())
    }

    fn position<F>(&self, predicate: F) -> Result<Self::PositionKey>
    where
        F: Fn(&M) -> bool,
    {
        self.queue_tree
            .values()
            .find_map(|uuid| {
                let message = self.objects.get(uuid).unwrap();
                if predicate(&message.1) {
                    Some(message.1.id())
                } else {
                    None
                }
            })
            .ok_or_else(|| DatabaseError::MessageNotFound)
    }

    fn get(&self, position: Self::PositionKey) -> Result<&M> {
        Ok(&self
            .objects
            .get(&position)
            .ok_or_else(|| DatabaseError::MessageNotFound)?
            .1)
    }

    fn get_mut(&mut self, position: Self::PositionKey) -> Result<&mut M> {
        Ok(&mut self
            .objects
            .get_mut(&position)
            .ok_or_else(|| DatabaseError::MessageNotFound)?
            .1)
    }

    fn delete_pos(&mut self, position: Self::PositionKey) -> Result<()> {
        let message = self
            .objects
            .remove(&position)
            .ok_or_else(|| DatabaseError::MessageNotFound)?;
        self.queue_tree
            .remove(&(message.1.sort(), message.0))
            .unwrap();
        Ok(())
    }

    fn retain<F>(&mut self, predicate: F) -> Result<()>
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

        Ok(())
    }

    fn len(&self) -> Result<usize> {
        Ok(self.objects.len())
    }

    fn clear(&mut self) -> Result<()> {
        self.objects.clear();
        self.queue_tree.clear();
        Ok(())
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
        database.push_raw(message).unwrap();
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
        database.push_raw(message1.clone()).unwrap();
        database.push_raw(message2.clone()).unwrap();
        database.push_raw(message3.clone()).unwrap();
        database.push_raw(message4.clone()).unwrap();

        position!(database, message1);
        position!(database, message2);
        position!(database, message4);
        position!(database, message3);
    }

    #[test]
    fn test_get_mut() {
        let mut database = create_database();
        let message = create_message!();
        database.push_raw(message).unwrap();
        let pos = database.position(|_| true).unwrap();
        let message = database.get_mut(pos).unwrap();
        message.reserve();
    }

    #[test]
    fn test_delete() {
        let mut database = create_database();
        let message1 = create_message!();
        let message2 = create_message!(|_| Utc::today().and_hms(1, 0, 0).timestamp());
        database.push_raw(message1).unwrap();
        database.push_raw(message2.clone()).unwrap();
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
        database.push_raw(message1).unwrap();
        database.push_raw(message2.clone()).unwrap();
        database
            .retain(|message| message.id == message2.id)
            .unwrap();
        assert_eq!(database.objects.len(), 1);
        assert_eq!(database.queue_tree.len(), 1);
        let pos = database.position(|_| true).unwrap();
        assert_eq!(database.get(pos).unwrap().id, message2.id);
    }

    #[test]
    fn test_len_clear() {
        let mut database = create_database();
        assert_eq!(database.len().unwrap(), 0);
        database.push_raw(create_message!()).unwrap();
        database.push_raw(create_message!()).unwrap();
        database.push_raw(create_message!()).unwrap();
        database.push_raw(create_message!()).unwrap();
        assert_eq!(database.len().unwrap(), 4);
        database.clear().unwrap();
        assert_eq!(database.len().unwrap(), 0);
    }
}
