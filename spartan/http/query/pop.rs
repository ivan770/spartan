use chrono::{DateTime, FixedOffset};
use serde::Serialize;
use spartan_lib::core::{
    message::{Message, State},
    payload::{Dispatchable, Identifiable},
};

#[derive(Serialize)]
pub struct Timeout<'m> {
    max: &'m u32,
    obtained_at: &'m Option<DateTime<FixedOffset>>,
}

#[derive(Serialize)]
pub struct Time<'m> {
    dispatched_at: &'m DateTime<FixedOffset>,
    delay: &'m Option<DateTime<FixedOffset>>,
    timeout: Timeout<'m>,
}

#[derive(Serialize)]
pub struct PopResponse<'m> {
    id: <Message as Identifiable>::Id,
    body: &'m <Message as Dispatchable>::Body,
    state: &'m State,
    time: Time<'m>,
}

impl<'m> From<&'m Message> for PopResponse<'m> {
    fn from(message: &'m Message) -> Self {
        PopResponse {
            id: message.id(),
            body: message.body(),
            state: message.state(),
            time: Time {
                dispatched_at: message.time().dispatched_at(),
                delay: message.time().delay(),
                timeout: Timeout {
                    max: message.time().timeout().max(),
                    obtained_at: message.time().timeout().obtained_at(),
                },
            },
        }
    }
}

#[cfg(test)]
pub mod test_response {
    use serde::Deserialize;

    use super::*;

    #[derive(Deserialize)]
    pub struct TestPopResponse {
        pub id: <Message as Identifiable>::Id,
        pub body: Box<<Message as Dispatchable>::Body>,
    }
}
