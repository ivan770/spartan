use crate::{
    node::QueueExtractor,
    query::{pop::PopResponse, Error, Response},
    respond, Request,
};
use spartan_lib::core::dispatcher::StatusAwareDispatcher;
use tide::{Result, StatusCode};

/// Pop message from queue.
///
/// Doesn't require any input, returns reserved message.
/// After reserving message, you either need to return it to queue, or delete it.
/// Messages that are not returned after timeout are deleted by GC.
pub async fn pop(request: Request) -> Result {
    let mut queue = respond!(QueueExtractor::new(&request).extract().await);
    let message = respond!(queue
        .pop()
        .ok_or_else(|| Error::new(StatusCode::NotFound, "No message available")));
    Ok(PopResponse::new(message).respond())
}
