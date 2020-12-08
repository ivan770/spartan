/// Message builder
pub mod builder;

/// Message time manager
mod time;

/// Message internal state
mod state;

use serde::{Deserialize, Serialize};
pub use state::{State, Status};
pub use time::{Offset, Time, Timeout};
use uuid::Uuid;

use crate::core::payload::{Dispatchable, Identifiable, Sortable, Status as StatusPayload};

/// Default message implementation, with support of all [`payload`] traits
///
/// [`Sortable`] implementation is compatible with [`TreeDatabase`]
///
/// [`payload`]: crate::core::payload
/// [`TreeDatabase`]: crate::core::db::TreeDatabase
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    id: Uuid,
    body: Box<str>,
    state: State,
    time: Time,
}

impl Message {
    fn new(
        body: Box<str>,
        delay: Option<u32>,
        offset: Offset,
        max_tries: u32,
        timeout: u32,
    ) -> Self {
        Message {
            id: Message::generate_id(),
            body,
            state: State::new(max_tries),
            time: Time::new(offset, delay, timeout),
        }
    }

    fn generate_id() -> Uuid {
        Uuid::new_v4()
    }

    /// Get current message [`State`]
    ///
    /// [`State`]: state::State
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Get message [`Time`]
    ///
    /// [`Time`]: time::Time
    pub fn time(&self) -> &Time {
        &self.time
    }
}

impl Identifiable for Message {
    type Id = Uuid;

    fn id(&self) -> Uuid {
        self.id
    }
}

impl Dispatchable for Message {
    type Body = str;

    fn obtainable(&self) -> bool {
        self.time.check_delay() && !self.time.expired()
    }

    fn body(&self) -> &Self::Body {
        &self.body
    }

    fn gc(&self) -> bool {
        self.state.requires_gc() || (self.state.requeueable() && self.time.expired())
    }
}

impl StatusPayload for Message {
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

    macro_rules! delayed_message {
        ($time:expr) => {
            MessageBuilder::default()
                .delay($time)
                .body("Hello world")
                .compose()
                .unwrap()
        };
    }

    #[test]
    fn test_sort() {
        let message1 = delayed_message!(2);
        let message2 = delayed_message!(3);
        let message3 = delayed_message!(1);
        let mut vec = vec![message1.clone(), message2.clone(), message3.clone()];
        vec.sort_by_key(|msg| msg.sort());
        assert_eq!(vec.pop().unwrap().id, message2.id);
        assert_eq!(vec.pop().unwrap().id, message1.id);
        assert_eq!(vec.pop().unwrap().id, message3.id);
    }
}
