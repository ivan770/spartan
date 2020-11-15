pub mod simple;
pub mod status_aware;

pub use simple::SimpleDispatcher;
pub use status_aware::StatusAwareDispatcher;

#[cfg(test)]
mod tests {
    #[macro_export]
    macro_rules! test_dispatcher {
        ($db:tt) => {
            use crate::core::dispatcher::SimpleDispatcher;
            use crate::core::message::{builder::MessageBuilder, Message};
            use crate::core::payload::{Identifiable, Status};
            use uuid::Uuid;

            fn generate_test_message() -> Message {
                MessageBuilder::default()
                    .body("Hello, world")
                    .max_tries(3)
                    .compose()
                    .unwrap()
            }

            fn create_database() -> $db<Message> {
                $db::<Message>::default()
            }

            #[test]
            fn push_message() {
                let message = generate_test_message();
                let mut db = create_database();
                db.push(message);
                assert_eq!(db.size(), 1);
                assert!(db.peek().unwrap().reservable());
            }

            #[test]
            fn peek_message() {
                let message = generate_test_message();
                let mut db = create_database();
                db.push(message.clone());
                assert_eq!(db.peek().unwrap().id(), message.id());
                assert!(db.peek().unwrap().reservable());
            }

            #[test]
            fn delete_message() {
                let message = generate_test_message();
                let mut db = create_database();
                db.push(message.clone());
                assert_eq!(db.size(), 1);
                db.delete(message.id()).unwrap();
            }

            #[test]
            #[should_panic]
            fn delete_nonexistent_message() {
                let mut db = create_database();
                db.delete(Uuid::new_v4()).unwrap();
            }

            #[test]
            fn gc() {
                let message = generate_test_message();
                let delayed_message = MessageBuilder::default()
                    .body("Hello, world")
                    .max_tries(3)
                    .delay(900)
                    .compose()
                    .unwrap();
                let useless_message = MessageBuilder::default()
                    .body("Hello, world")
                    .max_tries(0)
                    .compose()
                    .unwrap();
                let mut db = $db::default();
                db.push(message.clone());
                db.push(useless_message);
                db.push(delayed_message.clone());
                assert_eq!(db.size(), 3);
                db.gc();
                assert_eq!(db.size(), 2);
                assert_eq!(db.peek().unwrap().id(), message.id());
            }
        };
    }

    #[macro_export]
    macro_rules! test_status_dispatcher {
        ($db:tt) => {
            use crate::core::dispatcher::StatusAwareDispatcher;

            #[test]
            fn pop_message() {
                let message = generate_test_message();
                let mut db = create_database();
                db.push(message.clone());
                assert_eq!(db.pop().unwrap().id(), message.id());
            }

            #[test]
            fn chain_pop_message() {
                let message1 = generate_test_message();
                let message2 = generate_test_message();
                let mut db = create_database();
                db.push(message1.clone());
                db.push(message2.clone());
                assert_eq!(db.pop().unwrap().id(), message1.id());
                assert_eq!(db.pop().unwrap().id(), message2.id());
                assert!(db.pop().is_none());
            }

            #[test]
            fn delayed_message() {
                let message = generate_test_message();
                let delayed_message = MessageBuilder::default()
                    .body("Hello, world")
                    .max_tries(3)
                    .delay(900)
                    .compose()
                    .unwrap();
                let mut db = create_database();

                db.push(delayed_message);
                db.push(message.clone());

                assert_eq!(db.pop().unwrap().id(), message.id());
                assert_eq!(db.pop().is_some(), false);
            }

            #[test]
            fn delayed_and_ready_message() {
                let message = generate_test_message();
                let delayed_message = MessageBuilder::default()
                    .body("Hello, world")
                    .max_tries(3)
                    .delay(0)
                    .compose()
                    .unwrap();
                let mut db = create_database();

                db.push(message.clone());
                db.push(delayed_message.clone());

                assert_eq!(db.pop().unwrap().id(), message.id());
                assert_eq!(db.pop().unwrap().id(), delayed_message.id());
            }
        };
    }
}
