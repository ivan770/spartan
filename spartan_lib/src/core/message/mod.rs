pub mod builder;
mod state;
mod time;

use super::payload::Identifiable;
use crate::core::payload::{Dispatchable, Sortable, Status};
use serde::{Deserialize, Serialize};
use state::State;
use time::Time;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub id: Uuid,
    state: State,
    time: Time,
    body: Box<[u8]>,
}

impl Message {
    fn new(body: &[u8], delay: Option<i64>, offset: i32, max_tries: u32, timeout: u32) -> Self {
        Message {
            id: Message::generate_id(),
            state: State::new(max_tries),
            time: Time::new(offset, delay, timeout),
            body: body.into(),
        }
    }

    fn generate_id() -> Uuid {
        Uuid::new_v4()
    }
}

impl Identifiable for Message {
    type Id = Uuid;

    fn id(&self) -> Uuid {
        self.id
    }
}

impl Dispatchable for Message {
    fn obtainable(&self) -> bool {
        self.time.check_delay() && !self.time.expired()
    }

    fn gc(&self) -> bool {
        self.state.gc() || (self.state.requeueable() && self.time.expired())
    }
}

impl Status for Message {
    fn requeue(&mut self) {
        self.state.requeue();
    }

    fn reserve(&mut self) {
        self.state.reserve();
        self.time.obtain();
    }

    fn requeueable(&self) -> bool {
        self.state.requeueable()
    }

    fn reservable(&self) -> bool {
        self.state.reservable()
    }
}

impl Sortable for Message {
    type Sort = Option<i64>;

    fn sort(&self) -> Self::Sort {
        self.time.get_raw_delay()
    }
}

#[cfg(test)]
mod tests {
    use super::builder::MessageBuilder;
    use crate::core::payload::Sortable;
    use chrono::Utc;

    macro_rules! delayed_message {
        ($time:expr) => {
            MessageBuilder::default()
                .delay($time)
                .body(b"Hello world")
                .compose()
                .unwrap()
        };
    }

    #[test]
    fn test_sort() {
        let message1 = delayed_message!(|_| Utc::today().and_hms(01, 00, 00).timestamp());
        let message2 = delayed_message!(|_| Utc::today().and_hms(02, 00, 00).timestamp());
        let message3 = delayed_message!(|_| Utc::today().and_hms(00, 00, 00).timestamp());
        let mut vec = vec![message1.clone(), message2.clone(), message3.clone()];
        vec.sort_by_key(|msg| msg.sort());
        assert_eq!(vec.pop().unwrap().id, message2.id);
        assert_eq!(vec.pop().unwrap().id, message1.id);
        assert_eq!(vec.pop().unwrap().id, message3.id);
    }
}
