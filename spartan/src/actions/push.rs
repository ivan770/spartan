use crate::{
    node::QueueExtractor,
    query::{
        push::{PushRequest, PushResponse},
        Error, Response,
    },
    respond, Request,
};
use spartan_lib::{
    core::{
        dispatcher::SimpleDispatcher,
        message::{builder::MessageBuilder, Message},
    },
};
use tide::Result;

/// Push message to queue.
///
/// Requires message body. Offset, max tries, timeout, delay are optional.
/// Returns empty response.
pub async fn push(mut request: Request) -> Result {
    let json: PushRequest = respond!(request.body_json().await.map_err(|_| Error::bad_request()));
    let mut queue = respond!(QueueExtractor::new(&request).extract().await);
    queue.push(apply_builder(&json));
    Ok(PushResponse::new().respond())
}

/// Compose message from push request.
pub fn apply_builder(request: &PushRequest) -> Message {
    let mut builder = MessageBuilder::default().body(request.body.clone());

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
        builder = builder
            .delay(delay);
    };

    builder.compose().expect("No message body provided")
}
