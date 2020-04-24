pub mod simple;
pub mod status_aware;

pub use simple::SimpleDispatcher;
pub use status_aware::StatusAwareDispatcher;

#[cfg(test)]
mod tests {
    use super::{SimpleDispatcher, StatusAwareDispatcher};
    use crate::core::db::vec::VecDatabase;
    use crate::core::message::{builder::MessageBuilder, Message};
    use crate::core::payload::{Dispatchable, Status};
    use chrono::{TimeZone, Utc};
    use rand::random;
    use uuid::Uuid;

    fn generate_test_message() -> Message {
        MessageBuilder::default()
            .body(&random::<[u8; 16]>())
            .max_tries(3)
            .compose()
            .unwrap()
    }

    fn create_database() -> VecDatabase<Message> {
        VecDatabase::<Message>::default()
    }

    #[test]
    fn push_message() {
        let message = generate_test_message();
        let mut db = create_database();
        db.push(message).unwrap();
        assert_eq!(db.size().unwrap(), 1);
        assert!(db.peak().unwrap().reservable());
    }

    #[test]
    fn peak_message() {
        let message = generate_test_message();
        let mut db = create_database();
        db.push(message.clone()).unwrap();
        assert_eq!(db.peak().unwrap().id, message.id);
        assert!(db.peak().unwrap().reservable());
    }

    #[test]
    fn pop_message() {
        let message = generate_test_message();
        let mut db = create_database();
        db.push(message.clone()).unwrap();
        assert_eq!(db.pop().unwrap().id, message.id);
    }

    #[test]
    fn chain_pop_message() {
        let message1 = generate_test_message();
        let message2 = generate_test_message();
        let mut db = create_database();
        db.push(message1.clone()).unwrap();
        db.push(message2.clone()).unwrap();
        assert_eq!(db.pop().unwrap().id, message1.id);
        assert_eq!(db.pop().unwrap().id, message2.id);
        assert!(db.pop().is_err());
    }

    #[test]
    fn delayed_message() {
        let message = generate_test_message();
        let delayed_message = MessageBuilder::default()
            .body(&random::<[u8; 16]>())
            .max_tries(3)
            .delay(|_| Utc.ymd(2030, 1, 1).and_hms(01, 00, 00).timestamp())
            .compose()
            .unwrap();
        let mut db = create_database();

        db.push(delayed_message).unwrap();
        db.push(message.clone()).unwrap();

        assert_eq!(db.pop().unwrap().id, message.id);
        assert_eq!(db.pop().is_ok(), false);
    }

    #[test]
    fn delayed_and_ready_message() {
        let message = generate_test_message();
        let delayed_message = MessageBuilder::default()
            .body(&random::<[u8; 16]>())
            .max_tries(3)
            .delay(|_| Utc.ymd(1990, 1, 1).and_hms(01, 00, 00).timestamp())
            .compose()
            .unwrap();
        let mut db = create_database();

        db.push(message.clone()).unwrap();
        db.push(delayed_message.clone()).unwrap();

        assert_eq!(db.pop().unwrap().id, message.id);
        assert_eq!(db.pop().unwrap().id, delayed_message.id);
    }

    #[test]
    fn delete_message() {
        let message = generate_test_message();
        let mut db = create_database();
        db.push(message.clone()).unwrap();
        assert_eq!(db.size().unwrap(), 1);
        db.delete(message.id).unwrap();
    }

    #[test]
    #[should_panic]
    fn delete_nonexistent_message() {
        let mut db = create_database();
        db.delete(Uuid::new_v4()).unwrap();
    }

    #[test]
    fn requeue() {
        let message = generate_test_message();
        let mut db = create_database();
        db.push(message.clone()).unwrap();
        let pop = db.pop().unwrap();
        assert!(pop.requeueable());
        db.requeue(message.id).unwrap();
        assert!(db.peak().unwrap().obtainable());
    }

    #[test]
    fn gc() {
        let message = generate_test_message();
        let delayed_message = MessageBuilder::default()
            .body(&random::<[u8; 16]>())
            .max_tries(3)
            .delay(|_| Utc.ymd(2030, 1, 1).and_hms(01, 00, 00).timestamp())
            .compose()
            .unwrap();
        let useless_message = MessageBuilder::default()
            .body(&random::<[u8; 16]>())
            .max_tries(0)
            .compose()
            .unwrap();
        let mut db = VecDatabase::default();
        db.push(message.clone()).unwrap();
        db.push(useless_message).unwrap();
        db.push(delayed_message.clone()).unwrap();
        assert_eq!(db.size().unwrap(), 3);
        db.gc().unwrap();
        assert_eq!(db.size().unwrap(), 2);
        assert_eq!(db.pop().unwrap().id, message.id);
    }
}
