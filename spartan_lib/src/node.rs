use crate::core::db::{Database, SerializableDatabase};
use crate::core::message::Message;
use bincode::{deserialize, serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;
#[derive(Error, Debug)]
pub enum NodeError {
    #[error("cannot lock mutex")]
    MutexError,
    #[error("unable to serialize database to bincode")]
    BincodeError,
}

#[derive(Default)]
pub struct Node<'a, DB>
where
    DB: Database<Message>,
{
    db: HashMap<&'a str, Arc<Mutex<DB>>>,
}

impl<'a, DB> Node<'a, DB>
where
    DB: Database<Message>,
{
    pub fn queue(&mut self, name: &'a str) -> &mut Arc<Mutex<DB>> {
        self.db
            .entry(name)
            .or_insert_with(|| Arc::new(Mutex::new(DB::default())))
    }
}

impl<'a, DB> Node<'a, DB>
where
    DB: SerializableDatabase<Message>,
{
    pub fn serialize(&self) -> Result<HashMap<&str, Vec<u8>>, NodeError> {
        let mut serialized_db = HashMap::new();
        for (key, value) in self.db.iter() {
            serialized_db.insert(
                *key,
                serialize(
                    value
                        .clone()
                        .lock()
                        .map_err(|_| NodeError::MutexError)?
                        .get_db(),
                )
                .map_err(|_| NodeError::BincodeError)?,
            );
        }
        Ok(serialized_db)
    }

    pub fn deserialize(db: HashMap<&'a str, Vec<u8>>) -> Result<Self, NodeError> {
        let mut node = Self::default();
        for (key, value) in db.iter() {
            node.queue(*key)
                .lock()
                .unwrap()
                .set_db(deserialize(&value).map_err(|_| NodeError::BincodeError)?);
        }
        Ok(node)
    }
}

#[cfg(test)]
mod tests {
    use super::{Message, Node};
    use crate::core::db::vec::VecDatabase;
    use crate::core::dispatcher::{SimpleDispatcher, StatusAwareDispatcher};
    use crate::core::message::builder::MessageBuilder;
    use crate::core::payload::Dispatchable;

    fn create_node<'a>() -> Node<'a, VecDatabase<Message>> {
        Node::<VecDatabase<Message>>::default()
    }

    fn create_message() -> Message {
        MessageBuilder::default()
            .body(b"Test message")
            .compose()
            .unwrap()
    }

    #[test]
    fn create_queue() {
        let mut node = create_node();
        let queue = node.queue("test").lock().unwrap();
        assert_eq!(queue.size().unwrap(), 0);
    }

    #[test]
    fn db() {
        let mut node = create_node();
        let mut queue = node.queue("test").lock().unwrap();
        let message = create_message();
        queue.push(message.clone()).unwrap();
        assert_eq!(queue.size().unwrap(), 1);
        queue.pop().unwrap();
        queue.delete(message.id()).unwrap();
        assert_eq!(queue.size().unwrap(), 0);
    }

    #[test]
    fn serialization() {
        let mut node = create_node();
        let message = create_message();
        let mut queue = node.queue("test").lock().unwrap();
        queue.push(message.clone()).unwrap();
        drop(queue);
        let serialized_db = node.serialize().unwrap();
        let mut deserialized_db = Node::<VecDatabase<Message>>::deserialize(serialized_db).unwrap();
        assert_eq!(
            deserialized_db
                .queue("test")
                .lock()
                .unwrap()
                .peak()
                .unwrap()
                .id(),
            message.id()
        );
    }
}
