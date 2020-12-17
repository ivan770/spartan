use std::convert::TryFrom;

use serde::Deserialize;
use spartan_lib::core::message::{
    builder::{BuilderError, MessageBuilder},
    Message,
};

#[derive(Deserialize)]
#[cfg_attr(test, derive(Default, serde::Serialize))]
pub struct PushRequest {
    pub body: Box<str>,
    pub offset: Option<i32>,
    pub max_tries: Option<u32>,
    pub timeout: Option<u32>,
    pub delay: Option<u32>,
}

impl TryFrom<PushRequest> for Message {
    type Error = BuilderError;

    fn try_from(request: PushRequest) -> Result<Message, Self::Error> {
        let mut builder = MessageBuilder::default().body(request.body);

        if let Some(offset) = request.offset {
            builder = builder.offset(offset);
        };

        if let Some(max_tries) = request.max_tries {
            builder = builder.max_tries(max_tries);
        };

        if let Some(timeout) = request.timeout {
            builder = builder.timeout(timeout);
        };

        if let Some(delay) = request.delay {
            builder = builder.delay(delay);
        };

        builder.compose()
    }
}
