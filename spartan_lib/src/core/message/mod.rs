pub mod builder;
mod state;
mod time;

use crate::core::payload::{Dispatchable, Status};
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

impl Dispatchable for Message {
    fn id(&self) -> &Uuid {
        &self.id
    }

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
